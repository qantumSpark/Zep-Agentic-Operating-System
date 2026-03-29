use crate::events::CliEvent;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::sync::broadcast;

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CLI not found in PATH")]
    CliNotFound,

    #[error("Session not spawned")]
    NotSpawned,

    #[error("Parse error: {0}")]
    ParseError(String),
}
pub type Result<T> = std::result::Result<T, SessionError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub project_dir: PathBuf,
}

/// Manages the lifecycle of a long-lived Claude Code CLI subprocess.
/// The CLI is spawned once via `start_session`, then user messages and
/// permission responses are sent through stdin as stream-json JSONL.
pub struct SessionManager {
    session_id: Option<String>,
    project_dir: PathBuf,
    is_running: bool,
    child: Option<tokio::process::Child>,
    stdin: Option<tokio::process::ChildStdin>,
}

impl SessionManager {
    pub fn new(project_dir: PathBuf) -> Self {
        SessionManager {
            session_id: None,
            project_dir,
            is_running: false,
            child: None,
            stdin: None,
        }
    }

    /// Check if Claude CLI is available and authenticated
    pub async fn check_cli_auth() -> Result<String> {
        let output = Command::new("claude")
            .arg("--version")
            .output()
            .await
            .map_err(|_| SessionError::CliNotFound)?;

        if !output.status.success() {
            return Err(SessionError::CliNotFound);
        }

        let version = String::from_utf8_lossy(&output.stdout);
        tracing::info!("Claude CLI version: {}", version);
        Ok(version.to_string())
    }

    /// Set the session ID (called when we parse a system init event)
    pub fn set_session_id(&mut self, id: String) {
        self.session_id = Some(id);
    }

    /// Get current session ID
    pub fn get_session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Spawn the CLI once as a long-lived process.
    /// Returns a broadcast::Receiver to listen for parsed events.
    /// User messages and permission responses are sent via stdin afterwards.
    pub async fn start_session(&mut self) -> Result<broadcast::Receiver<CliEvent>> {
        let (tx, rx) = broadcast::channel(512);

        let mut cmd = Command::new("claude");
        cmd.arg("-p")
            .arg("--input-format")
            .arg("stream-json")
            .arg("--output-format")
            .arg("stream-json")
            .arg("--verbose")
            .arg("--include-partial-messages")
            .arg("--permission-prompt-tool")
            .arg("stdio");

        cmd.current_dir(&self.project_dir);
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            tracing::error!("Failed to spawn CLI: {}", e);
            SessionError::Io(e)
        })?;
        tracing::info!("Claude CLI session started (long-lived)");
        self.is_running = true;

        // Take stdin handle for sending messages
        let stdin = child.stdin.take().ok_or(SessionError::Io(
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to pipe stdin"),
        ))?;
        self.stdin = Some(stdin);

        // Take stdout for event parsing
        let stdout = child.stdout.take().ok_or(SessionError::Io(
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to pipe stdout"),
        ))?;

        // Spawn stderr reader
        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                let reader = tokio::io::BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::warn!("[CLI stderr] {}", line);
                }
            });
        }

        // Store child for lifecycle management
        self.child = Some(child);

        // Spawn JSONL parser task
        tokio::spawn(async move {
            if let Err(e) = crate::events::parse_stream(stdout, tx).await {
                tracing::error!("Event parser error: {}", e);
            }
        });

        Ok(rx)
    }

    /// Send a user message to the long-lived CLI process via stdin.
    pub async fn send_message(&mut self, prompt: &str) -> Result<()> {
        let stdin = self.stdin.as_mut().ok_or(SessionError::NotSpawned)?;

        let mut msg = serde_json::json!({
            "type": "user",
            "message": {
                "role": "user",
                "content": prompt
            },
            "parent_tool_use_id": null
        });

        // Include session_id for resume if we have one
        if let Some(ref sid) = self.session_id {
            msg["session_id"] = serde_json::Value::String(sid.clone());
        }

        let mut line = serde_json::to_string(&msg).map_err(|e| {
            SessionError::ParseError(format!("Failed to serialize message: {}", e))
        })?;
        line.push('\n');

        stdin
            .write_all(line.as_bytes())
            .await
            .map_err(SessionError::Io)?;
        stdin.flush().await.map_err(SessionError::Io)?;

        tracing::info!("Sent user message to CLI");
        Ok(())
    }

    /// Send a permission response (allow/deny) to the CLI for a pending tool approval.
    /// For allow, `tool_input` must contain the original (or modified) tool input.
    pub async fn send_permission_response(
        &mut self,
        id: &str,
        allow: bool,
        tool_input: Option<serde_json::Value>,
    ) -> Result<()> {
        let stdin = self.stdin.as_mut().ok_or(SessionError::NotSpawned)?;

        let result_json = if allow {
            serde_json::json!({
                "behavior": "allow",
                "updatedInput": tool_input.unwrap_or(serde_json::json!({}))
            })
        } else {
            serde_json::json!({
                "behavior": "deny",
                "message": "User denied this action"
            })
        };

        let response = serde_json::json!({
            "type": "control_response",
            "response": {
                "subtype": "success",
                "request_id": id,
                "response": result_json
            }
        });

        let mut line = serde_json::to_string(&response).map_err(|e| {
            SessionError::ParseError(format!("Failed to serialize response: {}", e))
        })?;
        line.push('\n');

        stdin
            .write_all(line.as_bytes())
            .await
            .map_err(SessionError::Io)?;
        stdin.flush().await.map_err(SessionError::Io)?;

        tracing::info!("Sent permission response: id={}, allow={}, json={}", id, allow, line.trim());
        Ok(())
    }

    pub fn set_running(&mut self, running: bool) {
        self.is_running = running;
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Whether a long-lived CLI session has been started.
    pub fn is_session_started(&self) -> bool {
        self.child.is_some()
    }

    /// Interrupt the running CLI process
    pub async fn interrupt(&mut self) -> Result<()> {
        if let Some(ref child) = self.child {
            if let Some(pid) = child.id() {
                tracing::info!("Interrupting CLI process with PID: {}", pid);
                #[cfg(target_os = "windows")]
                {
                    let _ = Command::new("taskkill")
                        .args(["/PID", &pid.to_string(), "/T", "/F"])
                        .output()
                        .await;
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let pid_i32 = i32::try_from(pid).map_err(|_| {
                        SessionError::ParseError(format!("PID {} exceeds i32::MAX", pid))
                    })?;
                    // SAFETY: pid is a valid process ID obtained from Child::id().
                    let ret = unsafe { libc::kill(pid_i32, libc::SIGTERM) };
                    if ret != 0 {
                        tracing::warn!(
                            "libc::kill returned {}, process may have already exited",
                            ret
                        );
                    }
                }
            }
        }
        self.child = None;
        self.stdin = None;
        self.is_running = false;
        Ok(())
    }

    /// List available sessions (placeholder)
    pub async fn list_sessions(&self) -> Result<Vec<String>> {
        tracing::debug!("list_sessions called");
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_manager_creation() {
        let manager = SessionManager::new(PathBuf::from("/tmp"));
        assert!(!manager.is_running());
        assert!(!manager.is_session_started());
    }
}
