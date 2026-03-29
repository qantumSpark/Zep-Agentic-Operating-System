use crate::events::CliEvent;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};
use tokio::sync::broadcast;

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CLI not found in PATH")]
    CliNotFound,

    #[error("Session not spawned")]
    NotSpawned,

    #[error("Write failed")]
    WriteFailed,

    #[error("Parse error: {0}")]
    ParseError(String),
}

pub type Result<T> = std::result::Result<T, SessionError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub project_dir: PathBuf,
    pub cwd: String,
    pub model: String,
    pub tokens_input: u64,
    pub tokens_output: u64,
    pub duration_ms: u64,
}

/// Manages the lifecycle of a Claude Code CLI subprocess
/// Spawns the CLI with --output-format stream-json, handles stdin/stdout communication
pub struct SessionManager {
    child: Option<Child>,
    session_id: Option<String>,
    tx: broadcast::Sender<CliEvent>,
    project_dir: PathBuf,
}

impl SessionManager {
    pub fn new(project_dir: PathBuf) -> Self {
        let (tx, _) = broadcast::channel(256);
        SessionManager {
            child: None,
            session_id: None,
            tx,
            project_dir,
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

    /// Spawn the Claude Code CLI with stream-json output format
    pub async fn spawn_cli(
        &mut self,
        project_dir: Option<PathBuf>,
    ) -> Result<broadcast::Receiver<CliEvent>> {
        // Determine project directory
        let proj_dir = project_dir.unwrap_or_else(|| self.project_dir.clone());

        // Build CLI command
        let mut cmd = Command::new("claude");
        cmd.arg("--output-format")
            .arg("stream-json")
            .arg("--verbose")
            .arg("--include-partial-messages")
            .arg("--project-dir")
            .arg(&proj_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| {
                tracing::error!("Failed to spawn CLI: {}", e);
                SessionError::Io(e)
            })?;

        tracing::info!("Claude CLI spawned");

        // Extract stdout for event parsing
        let stdout = child
            .stdout
            .take()
            .ok_or(SessionError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to pipe stdout",
            )))?;

        // Spawn event parser task
        let tx = self.tx.clone();
        tokio::spawn(async move {
            if let Err(e) = crate::events::parse_stream(stdout, tx).await {
                tracing::error!("Event parser error: {}", e);
            }
        });

        self.child = Some(child);

        // Return broadcast receiver for UI to listen on
        Ok(self.tx.subscribe())
    }

    /// Send a prompt to the CLI via stdin
    pub async fn send_prompt(&mut self, text: &str) -> Result<()> {
        let child = self.child.as_mut().ok_or(SessionError::NotSpawned)?;

        let stdin = child
            .stdin
            .as_mut()
            .ok_or(SessionError::WriteFailed)?;

        // Write prompt (newline-terminated)
        stdin
            .write_all(format!("{}\n", text).as_bytes())
            .await
            .map_err(|e| {
                tracing::error!("Failed to write prompt: {}", e);
                SessionError::WriteFailed
            })?;

        stdin.flush().await.map_err(|e| {
            tracing::error!("Failed to flush stdin: {}", e);
            SessionError::WriteFailed
        })?;

        tracing::debug!("Prompt sent to CLI");
        Ok(())
    }

    /// Resume an existing session
    pub async fn send_resume(&mut self, session_id: &str, text: &str) -> Result<()> {
        // Build CLI command with --resume flag
        let mut cmd = Command::new("claude");
        cmd.arg("--output-format")
            .arg("stream-json")
            .arg("--verbose")
            .arg("--include-partial-messages")
            .arg("--resume")
            .arg(session_id)
            .arg("--project-dir")
            .arg(&self.project_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            tracing::error!("Failed to spawn CLI for resume: {}", e);
            SessionError::Io(e)
        })?;

        let stdout = child.stdout.take().ok_or(SessionError::Io(
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to pipe stdout"),
        ))?;

        let tx = self.tx.clone();
        tokio::spawn(async move {
            if let Err(e) = crate::events::parse_stream(stdout, tx).await {
                tracing::error!("Event parser error: {}", e);
            }
        });

        self.child = Some(child);
        self.session_id = Some(session_id.to_string());

        // Send initial prompt
        self.send_prompt(text).await?;

        Ok(())
    }

    /// Interrupt the CLI subprocess (send SIGINT)
    pub async fn interrupt(&mut self) -> Result<()> {
        let child = self.child.as_mut().ok_or(SessionError::NotSpawned)?;

        child.kill().await.map_err(|e| {
            tracing::error!("Failed to kill CLI process: {}", e);
            SessionError::Io(e)
        })?;

        tracing::info!("CLI process interrupted");
        Ok(())
    }

    /// List available sessions from ~/.claude/projects/<encoded-cwd>/
    pub async fn list_sessions(&self) -> Result<Vec<String>> {
        // This would read from ~/.claude/projects/<encoded-cwd>/*.jsonl
        // For now, return empty list (can be enhanced later)
        tracing::debug!("list_sessions called");
        Ok(Vec::new())
    }

    /// Get the broadcast sender for the current session
    pub fn get_event_sender(&self) -> broadcast::Sender<CliEvent> {
        self.tx.clone()
    }

    /// Check if session is still running
    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut child) = self.child {
            matches!(child.try_wait(), Ok(None))
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_manager_creation() {
        let manager = SessionManager::new(PathBuf::from("/tmp"));
        assert!(!manager.is_running());
    }
}
