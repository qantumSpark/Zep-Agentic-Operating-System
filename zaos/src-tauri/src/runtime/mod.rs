pub mod claude;
pub mod paths;

pub use paths::{RuntimeKind, RuntimePaths};

use crate::events::zaos_events::ZaosEvent;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::broadcast;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Runtime CLI not found")]
    CliNotFound,

    #[error("CLI found but not authenticated")]
    NotAuthenticated,

    #[error("Session not started")]
    NotStarted,

    #[error("Protocol error: {0}")]
    Protocol(String),
}

pub type Result<T> = std::result::Result<T, RuntimeError>;

// ---------------------------------------------------------------------------
// Session record (runtime-agnostic)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub session_id: String,
    pub first_prompt: Option<String>,
    pub last_prompt: Option<String>,
    pub timestamp: Option<String>,
    pub last_modified: Option<String>,
}

// ---------------------------------------------------------------------------
// AgentRuntime trait — the abstraction seam
// ---------------------------------------------------------------------------

/// Trait representing an AI coding agent runtime (Claude, Codex, etc.).
///
/// Each implementation encapsulates:
/// - How to spawn / connect to the agent process
/// - The wire protocol for sending messages and receiving events
/// - Permission handling specific to that runtime
/// - Session storage format and location
///
/// The event channel carries [`ZaosEvent`] — the runtime-agnostic event type.
/// Each runtime implementation translates its native events into this format.
#[async_trait]
pub trait AgentRuntime: Send + Sync {
    /// Human-readable name for this runtime (e.g. "Claude Code CLI")
    fn name(&self) -> &str;

    /// Check whether the runtime CLI binary is installed (present on PATH).
    /// Returns the version string on success, or `CliNotFound` if absent.
    async fn check_installed(&self) -> Result<String>;

    /// Check whether the runtime is installed **and** authenticated.
    /// Returns a version string on success, `NotAuthenticated` if the CLI
    /// is present but the user is not logged in, or `CliNotFound` if absent.
    async fn check_auth(&self) -> Result<String>;

    /// Start a new session. Returns a broadcast receiver for parsed events.
    async fn start_session(&mut self) -> Result<broadcast::Receiver<ZaosEvent>>;

    /// Send a user message to the active session.
    async fn send_message(&mut self, message: &str) -> Result<()>;

    /// Respond to a pending permission / tool-approval request.
    async fn send_permission_response(
        &mut self,
        request_id: &str,
        allow: bool,
        tool_input: Option<serde_json::Value>,
    ) -> Result<()>;

    /// Interrupt / kill the active session.
    async fn interrupt(&mut self) -> Result<()>;

    /// Whether a session is currently running.
    fn is_session_active(&self) -> bool;

    /// Record the session ID (typically parsed from the first system event).
    fn set_session_id(&mut self, id: String);

    /// Get the current session ID, if any.
    fn get_session_id(&self) -> Option<&str>;

    /// List past sessions from this runtime's storage.
    async fn list_sessions(&self) -> Result<Vec<SessionRecord>>;
}

/// Create a runtime instance for the given kind.
///
/// This is the single entry-point for runtime construction. Adding a new
/// runtime means adding a branch here (and implementing `AgentRuntime`).
pub fn create_runtime(kind: RuntimeKind, project_dir: std::path::PathBuf) -> Box<dyn AgentRuntime> {
    match kind {
        RuntimeKind::Claude => Box::new(claude::ClaudeRuntime::new(project_dir)),
    }
}
