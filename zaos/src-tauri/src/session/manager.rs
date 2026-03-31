use crate::events::CliEvent;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::sync::broadcast;

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len])
    } else {
        s.to_string()
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliSession {
    pub session_id: String,
    pub first_prompt: Option<String>,
    pub last_prompt: Option<String>,
    pub timestamp: Option<String>,
    pub last_modified: Option<String>,
}

fn encode_project_path(path: &std::path::Path) -> String {
    let s = path.to_string_lossy();
    s.replace('\\', "-")
        .replace('/', "-")
        .replace(':', "-")
        .replace(' ', "-")
}

/// Manages the lifecycle of a long-lived Claude Code CLI subprocess.
/// The CLI is spawned once via `start_session`, then user messages and
/// permission responses are sent through stdin as stream-json JSONL.
pub struct SessionManager {
    session_id: Option<String>,
    project_dir: PathBuf,
    child: Option<tokio::process::Child>,
    stdin: Option<tokio::process::ChildStdin>,
}

impl SessionManager {
    pub fn new(project_dir: PathBuf) -> Self {
        SessionManager {
            session_id: None,
            project_dir,
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

        // Auto-sync workflow kit before spawning CLI
        let config = crate::deployer::config::WorkflowKitConfig::load(&self.project_dir);
        if config.auto_sync {
            match crate::deployer::deploy(&self.project_dir) {
                Ok(report) => {
                    if !report.created.is_empty() || !report.updated.is_empty() {
                        tracing::info!("Workflow kit synced before session start");
                    }
                }
                Err(e) => {
                    tracing::warn!("Workflow kit sync failed (non-fatal): {}", e);
                }
            }
        }

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

    /// Write a JSON value as a single JSONL line to the CLI stdin.
    async fn write_jsonl(&mut self, value: serde_json::Value) -> Result<()> {
        let stdin = self.stdin.as_mut().ok_or(SessionError::NotSpawned)?;
        let mut line = serde_json::to_string(&value).map_err(|e| {
            SessionError::ParseError(format!("Failed to serialize: {}", e))
        })?;
        line.push('\n');
        stdin.write_all(line.as_bytes()).await.map_err(SessionError::Io)?;
        stdin.flush().await.map_err(SessionError::Io)?;
        Ok(())
    }

    /// Send a user message to the long-lived CLI process via stdin.
    pub async fn send_message(&mut self, prompt: &str) -> Result<()> {
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

        self.write_jsonl(msg).await?;
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

        tracing::info!("Sent permission response: id={}, allow={}", id, allow);
        self.write_jsonl(response).await?;
        Ok(())
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
        Ok(())
    }

    /// List available sessions from Claude CLI session files on disk
    pub async fn list_sessions(&self) -> Result<Vec<CliSession>> {
        let home = dirs::home_dir().ok_or_else(|| {
            SessionError::ParseError("Could not find home directory".to_string())
        })?;

        let encoded = encode_project_path(&self.project_dir);
        let sessions_dir = home.join(".claude").join("projects").join(&encoded);

        let mut read_dir = match tokio::fs::read_dir(&sessions_dir).await {
            Ok(rd) => rd,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::debug!("Sessions dir does not exist: {:?}", sessions_dir);
                return Ok(Vec::new());
            }
            Err(e) => {
                tracing::warn!("Failed to read sessions dir: {}", e);
                return Err(SessionError::Io(e));
            }
        };

        let mut sessions = Vec::new();

        while let Ok(Some(entry)) = read_dir.next_entry().await {
            let path = entry.path();

            // Only process .jsonl files (not directories)
            if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }

            let session_id = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };

            // Read first and last lines using BufReader (avoid loading entire file)
            let file = match tokio::fs::File::open(&path).await {
                Ok(f) => f,
                Err(e) => {
                    tracing::warn!("Failed to open session file {:?}: {}", path, e);
                    continue;
                }
            };

            let reader = tokio::io::BufReader::new(file);
            let mut lines = reader.lines();

            let first_line = match lines.next_line().await {
                Ok(Some(line)) => line,
                _ => continue,
            };

            // Read remaining lines, keeping only the last
            let mut last_line = first_line.clone();
            while let Ok(Some(line)) = lines.next_line().await {
                last_line = line;
            }

            let mut first_prompt = None;
            let mut timestamp = None;
            let mut last_prompt = None;

            // Parse first line for timestamp and first prompt
            if let Ok(first) = serde_json::from_str::<serde_json::Value>(&first_line) {
                if first.get("type").and_then(|t| t.as_str()) == Some("queue-operation") {
                    timestamp = first.get("timestamp").and_then(|t| t.as_str()).map(|s| s.to_string());
                    first_prompt = first.get("content").and_then(|c| c.as_str()).map(|s| truncate_str(s, 100));
                }
            }

            // Parse last line for last prompt
            if let Ok(last) = serde_json::from_str::<serde_json::Value>(&last_line) {
                if last.get("type").and_then(|t| t.as_str()) == Some("last-prompt") {
                    last_prompt = last.get("lastPrompt").and_then(|c| c.as_str()).map(|s| truncate_str(s, 100));
                }
            }

            // Get file modification time
            let last_modified = tokio::fs::metadata(&path)
                .await
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    let datetime: chrono::DateTime<chrono::Utc> = t.into();
                    datetime.to_rfc3339()
                });

            sessions.push(CliSession {
                session_id,
                first_prompt,
                last_prompt,
                timestamp,
                last_modified,
            });
        }

        // Sort by last_modified descending (most recent first)
        sessions.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

        tracing::info!("Found {} sessions", sessions.len());
        Ok(sessions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_manager_creation() {
        let manager = SessionManager::new(PathBuf::from("/tmp"));
        assert!(!manager.is_session_started());
    }
}
