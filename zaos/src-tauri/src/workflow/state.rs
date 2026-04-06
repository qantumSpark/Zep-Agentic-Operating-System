use serde::{Deserialize, Serialize};
use chrono::Utc;

use super::product_phase::{derive_product_phase, ProductPhase};
use super::engine::peek_next_phase;

fn default_permission_mode() -> String {
    "strict".to_string()
}

fn default_policy_profile() -> String {
    "guided-build".to_string()
}

/// WorkflowState mirrors the schema in .workflow/state.json
/// Extended with ZAOS-specific fields: history, permissions, policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    // Core fields
    pub phase: String,
    pub epic: String,
    /// @deprecated — la source de verite pour la task courante est .memory/current-epic.md.
    /// Ce champ reste pour backward compat serde.
    pub task: String,
    pub mode: WorkflowMode,
    pub gate_validated: bool,
    #[serde(default)]
    pub gate_ready: bool,
    pub last_updated: String,

    // ZAOS extensions (optional)
    #[serde(default)]
    pub history: Vec<PhaseTransition>,

    #[serde(default = "default_permission_mode")]
    pub permission_mode: String,

    #[serde(default = "default_policy_profile")]
    pub policy_profile: String,
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
            permission_mode: default_permission_mode(),
            policy_profile: default_policy_profile(),
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
            permission_mode: default_permission_mode(),
            policy_profile: default_policy_profile(),
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
    pub policy_profile: String,
    pub last_updated: String,
    pub history: Vec<PhaseTransition>,
    pub product_phase: ProductPhase,
    pub next_product_phase: Option<ProductPhase>,
    pub next_tech_phase: Option<String>,

    // Epic enrichment from .memory/current-epic.md
    /// Epic status (e.g. "EN COURS", "TERMINE")
    pub epic_status: Option<String>,
    /// Epic objective text
    pub epic_objective: Option<String>,
    /// Total number of tasks in the epic
    pub epic_task_count: Option<u32>,
    /// Number of tasks with DONE or VALIDATED status
    pub epic_done_count: Option<u32>,
}

impl WorkflowStateDto {
    /// Build a DTO from a WorkflowState, deriving product phase fields.
    pub fn from_state(state: &WorkflowState) -> Self {
        let product_phase = derive_product_phase(state);

        let next_tech_phase = peek_next_phase(&state.phase).map(|s| s.to_string());

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
            task: String::new(),
            mode: state.mode,
            gate_validated: state.gate_validated,
            gate_ready: state.gate_ready,
            permission_mode: state.permission_mode.clone(),
            policy_profile: state.policy_profile.clone(),
            last_updated: state.last_updated.clone(),
            history: state.history.clone(),
            product_phase,
            next_product_phase,
            next_tech_phase,
            epic_status: None,
            epic_objective: None,
            epic_task_count: None,
            epic_done_count: None,
        }
    }

    /// Enrich the DTO with data from a parsed `CurrentEpic`.
    ///
    /// Populates `epic_status`, `epic_objective`, `epic_task_count`, and
    /// `epic_done_count` from the current-epic.md data.
    pub fn enrich_from_epic(&mut self, epic: &crate::memory::reader::CurrentEpic) {
        self.epic_status = Some(epic.status.clone());
        self.epic_objective = Some(epic.objective.clone());
        self.epic_task_count = Some(epic.tasks.len() as u32);
        let done_count = epic.tasks.iter().filter(|t| {
            let s = t.status.to_uppercase();
            s == "DONE" || s == "VALIDATED"
        }).count();
        self.epic_done_count = Some(done_count as u32);
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
        assert_eq!(dto.task, ""); // task field is deprecated, always empty in DTO
        assert_eq!(dto.mode, WorkflowMode::Free);
        assert!(dto.gate_validated);
        assert!(dto.gate_ready);
        assert_eq!(dto.permission_mode, "accept-edits");
        assert_eq!(dto.policy_profile, "guided-build");
    }
}
