use serde::{Deserialize, Serialize};
use chrono::Utc;

use super::product_phase::{derive_product_phase, ProductPhase};
use super::engine::peek_next_phase;

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
    #[serde(default)]
    pub gate_ready: bool,
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
            gate_ready: false,
            last_updated: Utc::now().to_rfc3339(),
            history: Vec::new(),
            session: None,
            permission_mode: default_permission_mode(),
        }
    }
}

#[allow(dead_code)]
impl WorkflowState {
    pub fn new(phase: String, epic: String, task: String) -> Self {
        WorkflowState {
            phase,
            epic,
            task,
            mode: WorkflowMode::default(),
            gate_validated: false,
            gate_ready: false,
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

/// DTO sent to the frontend — enriches WorkflowState with derived product phase.
/// Never persisted, always computed on-the-fly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStateDto {
    pub phase: String,
    pub epic: String,
    pub task: String,
    pub mode: WorkflowMode,
    pub gate_validated: bool,
    pub gate_ready: bool,
    pub permission_mode: String,
    pub last_updated: String,
    pub history: Vec<PhaseTransition>,
    pub session: Option<SessionMetadata>,
    pub product_phase: ProductPhase,
    pub next_product_phase: Option<ProductPhase>,
}

impl WorkflowStateDto {
    /// Build a DTO from a WorkflowState, deriving product phase fields.
    pub fn from_state(state: &WorkflowState) -> Self {
        let product_phase = derive_product_phase(state);

        // Derive next product phase from the next technical phase
        let next_product_phase = peek_next_phase(&state.phase).map(|next_tech| {
            // Build a minimal temporary state to derive the product phase
            // for the next technical phase. We need epic and empty history
            // because the "next" state hasn't transitioned yet.
            let mut next_state = WorkflowState::default();
            next_state.phase = next_tech.to_string();
            next_state.epic = state.epic.clone();
            // When transitioning from closure to idle, simulate the closure→idle
            // history entry so derive_product_phase correctly returns Learn.
            if state.phase == "closure" && next_tech == "idle" {
                next_state.history.push(PhaseTransition {
                    from_phase: "closure".to_string(),
                    to_phase: "idle".to_string(),
                    timestamp: String::new(),
                    reason: None,
                });
            }
            derive_product_phase(&next_state)
        });

        WorkflowStateDto {
            phase: state.phase.clone(),
            epic: state.epic.clone(),
            task: state.task.clone(),
            mode: state.mode,
            gate_validated: state.gate_validated,
            gate_ready: state.gate_ready,
            permission_mode: state.permission_mode.clone(),
            last_updated: state.last_updated.clone(),
            history: state.history.clone(),
            session: state.session.clone(),
            product_phase,
            next_product_phase,
        }
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

    // --- DTO integration tests ---

    #[test]
    fn test_dto_implementation_phase() {
        let mut state = WorkflowState::default();
        state.phase = "implementation".to_string();
        state.epic = "MyEpic".to_string();

        let dto = WorkflowStateDto::from_state(&state);

        assert_eq!(dto.product_phase, ProductPhase::Build);
        assert_eq!(dto.next_product_phase, Some(ProductPhase::Verify));
    }

    #[test]
    fn test_dto_closure_phase() {
        // With epic: next is idle with closure→idle history => Learn
        let mut state = WorkflowState::default();
        state.phase = "closure".to_string();
        state.epic = "MyEpic".to_string();

        let dto = WorkflowStateDto::from_state(&state);

        assert_eq!(dto.product_phase, ProductPhase::Release);
        assert_eq!(dto.next_product_phase, Some(ProductPhase::Learn));

        // Without epic: next is idle with closure→idle history => Learn
        let mut state_empty = WorkflowState::default();
        state_empty.phase = "closure".to_string();

        let dto_empty = WorkflowStateDto::from_state(&state_empty);

        assert_eq!(dto_empty.product_phase, ProductPhase::Release);
        assert_eq!(dto_empty.next_product_phase, Some(ProductPhase::Learn));
    }

    #[test]
    fn test_dto_idle_post_closure_learn() {
        let mut state = WorkflowState::default();
        state.epic = "MyEpic".to_string();
        state.history.push(PhaseTransition {
            from_phase: "closure".to_string(),
            to_phase: "idle".to_string(),
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            reason: None,
        });

        let dto = WorkflowStateDto::from_state(&state);

        assert_eq!(dto.product_phase, ProductPhase::Learn);
        // next is comprehension with epic => Imagine
        assert_eq!(dto.next_product_phase, Some(ProductPhase::Imagine));
    }

    #[test]
    fn test_dto_review_phase() {
        let mut state = WorkflowState::default();
        state.phase = "review".to_string();
        state.epic = "MyEpic".to_string();

        let dto = WorkflowStateDto::from_state(&state);

        assert_eq!(dto.product_phase, ProductPhase::Verify);
        // next phase is "test" which also maps to Verify
        assert_eq!(dto.next_product_phase, Some(ProductPhase::Verify));
    }

    #[test]
    fn test_dto_closure_next_is_learn() {
        let mut state = WorkflowState::default();
        state.phase = "closure".to_string();
        state.epic = "MyEpic".to_string();
        let dto = WorkflowStateDto::from_state(&state);
        assert_eq!(dto.product_phase, ProductPhase::Release);
        assert_eq!(dto.next_product_phase, Some(ProductPhase::Learn));
    }

    #[test]
    fn test_dto_closure_no_epic_next_is_learn() {
        let mut state = WorkflowState::default();
        state.phase = "closure".to_string();
        state.epic = "".to_string();
        let dto = WorkflowStateDto::from_state(&state);
        assert_eq!(dto.product_phase, ProductPhase::Release);
        // Even without epic, closure→idle should produce Learn
        assert_eq!(dto.next_product_phase, Some(ProductPhase::Learn));
    }

    #[test]
    fn test_dto_preserves_all_fields() {
        let mut state = WorkflowState::default();
        state.phase = "architecture".to_string();
        state.epic = "SomeEpic".to_string();
        state.task = "task-42".to_string();
        state.mode = WorkflowMode::Free;
        state.gate_validated = true;
        state.gate_ready = true;
        state.permission_mode = "accept-edits".to_string();

        let dto = WorkflowStateDto::from_state(&state);

        assert_eq!(dto.phase, "architecture");
        assert_eq!(dto.epic, "SomeEpic");
        assert_eq!(dto.task, "task-42");
        assert_eq!(dto.mode, WorkflowMode::Free);
        assert!(dto.gate_validated);
        assert!(dto.gate_ready);
        assert_eq!(dto.permission_mode, "accept-edits");
    }
}
