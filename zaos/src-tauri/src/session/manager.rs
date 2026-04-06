use crate::events::zaos_events::ZaosEvent;
use crate::runtime::{AgentRuntime, RuntimeError, RuntimeKind, SessionRecord};
use crate::runtime::create_runtime;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use tokio::sync::broadcast;

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Runtime error: {0}")]
    Runtime(#[from] RuntimeError),
}
pub type Result<T> = std::result::Result<T, SessionError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SessionInfo {
    pub session_id: String,
    pub project_dir: PathBuf,
}

/// Re-export SessionRecord as CliSession for backward compatibility.
/// Commands and frontend still reference this name.
pub type CliSession = SessionRecord;

/// Manages the lifecycle of an agent session.
///
/// Wraps a [`AgentRuntime`] implementation and adds ZAOS-level orchestration
/// (e.g. deployer sync before session start). The public API remains identical
/// so that `commands.rs` requires minimal changes.
pub struct SessionManager {
    runtime: Box<dyn AgentRuntime>,
    project_dir: PathBuf,
}

impl SessionManager {
    /// Create a new SessionManager with the specified runtime kind.
    pub fn new(project_dir: PathBuf, runtime_kind: RuntimeKind) -> Self {
        let runtime = create_runtime(runtime_kind, project_dir.clone());
        SessionManager {
            runtime,
            project_dir,
        }
    }

    /// Create a SessionManager with a custom runtime (for future use / testing).
    pub fn with_runtime(project_dir: PathBuf, runtime: Box<dyn AgentRuntime>) -> Self {
        SessionManager {
            runtime,
            project_dir,
        }
    }

    /// Check if the runtime CLI is available and authenticated.
    pub async fn check_cli_auth(&self) -> Result<String> {
        self.runtime.check_auth().await.map_err(SessionError::Runtime)
    }

    /// Set the session ID (called when we parse a system init event).
    pub fn set_session_id(&mut self, id: String) {
        self.runtime.set_session_id(id);
    }

    /// Get current session ID.
    pub fn get_session_id(&self) -> Option<&str> {
        self.runtime.get_session_id()
    }

    /// Start a session via the runtime.
    ///
    /// Runs the ZAOS pre-session hook (deployer sync) before delegating
    /// to the runtime's `start_session`.
    pub async fn start_session(&mut self) -> Result<broadcast::Receiver<ZaosEvent>> {
        // Pre-session hook: auto-sync workflow kit
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

        self.runtime.start_session().await.map_err(SessionError::Runtime)
    }

    /// Send a user message to the active session.
    pub async fn send_message(&mut self, prompt: &str) -> Result<()> {
        self.runtime.send_message(prompt).await.map_err(SessionError::Runtime)
    }

    /// Send a permission response (allow/deny) for a pending tool approval.
    pub async fn send_permission_response(
        &mut self,
        id: &str,
        allow: bool,
        tool_input: Option<serde_json::Value>,
    ) -> Result<()> {
        self.runtime
            .send_permission_response(id, allow, tool_input)
            .await
            .map_err(SessionError::Runtime)
    }

    /// Whether a session is currently running.
    pub fn is_session_started(&self) -> bool {
        self.runtime.is_session_active()
    }

    /// Interrupt the running session.
    pub async fn interrupt(&mut self) -> Result<()> {
        self.runtime.interrupt().await.map_err(SessionError::Runtime)
    }

    /// List available sessions from the runtime's storage.
    pub async fn list_sessions(&self) -> Result<Vec<CliSession>> {
        self.runtime.list_sessions().await.map_err(SessionError::Runtime)
    }

    /// Get the runtime's human-readable name.
    #[allow(dead_code)]
    pub fn runtime_name(&self) -> &str {
        self.runtime.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_manager_creation() {
        let manager = SessionManager::new(PathBuf::from("/tmp"), RuntimeKind::default());
        assert!(!manager.is_session_started());
    }

    #[test]
    fn test_runtime_name() {
        let manager = SessionManager::new(PathBuf::from("/tmp"), RuntimeKind::default());
        assert_eq!(manager.runtime_name(), "Claude Code CLI");
    }
}
