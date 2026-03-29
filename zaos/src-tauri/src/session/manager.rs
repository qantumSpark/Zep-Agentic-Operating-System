use crate::events::CliEvent;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use thiserror::Error;
use tokio::io::AsyncBufReadExt;
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

/// Manages the lifecycle of Claude Code CLI subprocesses.
/// Each user message spawns a new CLI process with -p "prompt".
/// Session continuity is maintained via --resume <session_id>.
pub struct SessionManager {
    session_id: Option<String>,
    project_dir: PathBuf,
    is_running: bool,
    child_pid: Option<u32>,
}

impl SessionManager {
    pub fn new(project_dir: PathBuf) -> Self {
        SessionManager {
            session_id: None,
            project_dir,
            is_running: false,
            child_pid: None,
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
    /// Spawn a CLI process for a single prompt.
    /// Returns a broadcast::Receiver to listen for parsed events.
    /// If we have a session_id, uses --resume for continuity.
    pub async fn spawn_for_prompt(
        &mut self,
        prompt: &str,
    ) -> Result<(tokio::process::Child, broadcast::Receiver<CliEvent>)> {
        let (tx, rx) = broadcast::channel(512);

        let mut cmd = Command::new("claude");
        cmd.arg("-p")
            .arg(prompt)
            .arg("--output-format")
            .arg("stream-json")
            .arg("--verbose")
            .arg("--include-partial-messages")
            .arg("--dangerously-skip-permissions");

        // Add resume flag if we have a previous session
        if let Some(ref sid) = self.session_id {
            cmd.arg("--resume").arg(sid);
            tracing::info!("Resuming session: {}", sid);
        }

        // Set working directory instead of --project-dir
        cmd.current_dir(&self.project_dir);
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            tracing::error!("Failed to spawn CLI: {}", e);
            SessionError::Io(e)
        })?;
        tracing::info!("Claude CLI spawned for prompt");
        self.is_running = true;
        self.child_pid = child.id();
        if self.child_pid.is_none() {
            tracing::warn!("child.id() returned None — interrupt will not work for this process");
        }

        // Take stdout for event parsing
        let stdout = child
            .stdout
            .take()
            .ok_or(SessionError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to pipe stdout",
            )))?;

        // Spawn stderr reader for debugging
        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                let reader = tokio::io::BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::warn!("[CLI stderr] {}", line);
                }
            });
        }

        // Spawn the JSONL parser task
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            if let Err(e) = crate::events::parse_stream(stdout, tx_clone).await {
                tracing::error!("Event parser error: {}", e);
            }
        });

        Ok((child, rx))
    }

    pub fn set_running(&mut self, running: bool) {
        self.is_running = running;
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Interrupt the running CLI process
    pub async fn interrupt(&mut self) -> Result<()> {
        if let Some(pid) = self.child_pid.take() {
            tracing::info!("Interrupting CLI process with PID: {}", pid);
            // On Windows, use taskkill; on Unix, send SIGTERM
            #[cfg(target_os = "windows")]
            {
                let _ = Command::new("taskkill")
                    .args(["/PID", &pid.to_string(), "/T", "/F"])
                    .output()
                    .await;
            }
            #[cfg(not(target_os = "windows"))]
            {
                // SAFETY: pid is a valid process ID obtained from Child::id().
                // We guard against u32 > i32::MAX overflow before casting.
                let pid_i32 = i32::try_from(pid).map_err(|_| {
                    SessionError::ParseError(format!("PID {} exceeds i32::MAX", pid))
                })?;
                let ret = unsafe { libc::kill(pid_i32, libc::SIGTERM) };
                if ret != 0 {
                    tracing::warn!("libc::kill returned {}, errno may indicate process already exited", ret);
                }
            }
            self.is_running = false;
            Ok(())
        } else {
            tracing::warn!("No running CLI process to interrupt");
            Ok(())
        }
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
    }
}