use serde::{Deserialize, Serialize};
use chrono::Utc;

fn default_permission_mode() -> String {
    "strict".to_string()
}

/// WorkflowState mirrors the schema in .workflow/state.json
/// Extended with ZAOS-specific fields: history and session info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    // Core fields
    pub phase: String,
    pub epic: String,
    pub task: String,
    pub mode: WorkflowMode,
    pub gate_validated: bool,
    pub last_updated: String,

    // ZAOS extensions (optional)
    #[serde(default)]
    pub history: Vec<PhaseTransition>,

    #[serde(default)]
    pub session: Option<SessionMetadata>,

    #[serde(default = "default_permission_mode")]
    pub permission_mode: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowMode {
    Free,
    Pipeline,
}

impl Default for WorkflowMode {
    fn default() -> Self {
        WorkflowMode::Pipeline
    }
}

/// Record of a phase transition for history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseTransition {
    pub from_phase: String,
    pub to_phase: String,
    pub timestamp: String,
    pub reason: Option<String>,
}

/// Session metadata attached to workflow state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub session_id: String,
    pub started_at: String,
    pub updated_at: String,
    pub tokens_used: TokenUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_creation: u64,
}

impl Default for WorkflowState {
    fn default() -> Self {
        WorkflowState {
            phase: "idle".to_string(),
            epic: "".to_string(),
            task: "".to_string(),
            mode: WorkflowMode::default(),
            gate_validated: false,
            last_updated: Utc::now().to_rfc3339(),
            history: Vec::new(),
            session: None,
            permission_mode: default_permission_mode(),
        }
    }
}

impl WorkflowState {
    pub fn new(phase: String, epic: String, task: String) -> Self {
        WorkflowState {
            phase,
            epic,
            task,
            mode: WorkflowMode::default(),
            gate_validated: false,
            last_updated: Utc::now().to_rfc3339(),
            history: Vec::new(),
            session: None,
            permission_mode: default_permission_mode(),
        }
    }

    /// Record a phase transition in history
    pub fn record_transition(&mut self, from: String, to: String, reason: Option<String>) {
        self.history.push(PhaseTransition {
            from_phase: from,
            to_phase: to,
            timestamp: Utc::now().to_rfc3339(),
            reason,
        });
        self.last_updated = Utc::now().to_rfc3339();
    }

    /// Update session metadata
    pub fn set_session(&mut self, session_id: String) {
        let now = Utc::now().to_rfc3339();
        self.session = Some(SessionMetadata {
            session_id,
            started_at: now.clone(),
            updated_at: now,
            tokens_used: TokenUsage::default(),
        });
    }

    /// Update token usage in session
    pub fn update_tokens(&mut self, input: u64, output: u64, cache_read: u64, cache_creation: u64) {
        if let Some(ref mut session) = self.session {
            session.tokens_used = TokenUsage {
                input,
                output,
                cache_read,
                cache_creation,
            };
            session.updated_at = Utc::now().to_rfc3339();
        }
        self.last_updated = Utc::now().to_rfc3339();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_workflow_state() {
        let state = WorkflowState::default();
        assert_eq!(state.phase, "idle");
        assert_eq!(state.mode, WorkflowMode::Pipeline);
        assert!(!state.gate_validated);
    }

    #[test]
    fn test_record_transition() {
        let mut state = WorkflowState::new(
            "idle".to_string(),
            "combat".to_string(),
            "create hitbox".to_string(),
        );
        state.record_transition(
            "idle".to_string(),
            "comprehension".to_string(),
            Some("started task".to_string()),
        );
        assert_eq!(state.history.len(), 1);
    }
}
