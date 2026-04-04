use crate::workflow::state::{WorkflowMode, WorkflowState};
use std::path::PathBuf;
use thiserror::Error;
use tokio::fs;
use tokio::sync::broadcast;

/// Check if current-epic.md has any tasks in active state (TODO, EN COURS, etc.).
/// Returns false if all tasks are DONE/VALIDATED or if the table is empty.
fn has_active_tasks(epic_content: &str) -> bool {
    let mut found_data_row = false;

    for line in epic_content.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with('|') || trimmed.contains("---") {
            continue;
        }

        let cells: Vec<&str> = trimmed.split('|').map(|c| c.trim()).collect();
        if cells.len() < 6 {
            continue;
        }

        let first_cell = cells[1];
        if first_cell == "#" || first_cell == "Task" || first_cell == "Statut" {
            continue;
        }

        found_data_row = true;

        let status = cells[4].to_uppercase();
        if status.contains("TODO")
            || status.contains("EN COURS")
            || status.contains("A FAIRE")
            || status.contains("IN_PROGRESS")
            || status.contains("BLOQUE")
        {
            return true;
        }
    }

    if !found_data_row {
        return true; // No data rows = plan not yet created = consider active
    }

    false
}

#[derive(Error, Debug)]
pub enum WorkflowError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid phase: {0}")]
    InvalidPhase(String),

    #[error("State file not found")]
    #[allow(dead_code)]
    StateNotFound,

    #[error("Watch error: {0}")]
    #[allow(dead_code)]
    WatchError(String),
}

pub type Result<T> = std::result::Result<T, WorkflowError>;

/// Returns the next phase in the pipeline without mutating state.
/// Returns None for unrecognized phases. Returns Some("idle") for closure.
pub fn peek_next_phase(current: &str) -> Option<&'static str> {
    match current {
        "idle" => Some("comprehension"),
        "comprehension" => Some("specification"),
        "specification" => Some("architecture"),
        "architecture" => Some("implementation"),
        "implementation" => Some("review"),
        "review" => Some("test"),
        "test" => Some("closure"),
        "closure" => Some("idle"),
        _ => None,
    }
}

/// WorkflowEngine manages the workflow state and transitions
/// Reads/writes .workflow/state.json, watches for changes
pub struct WorkflowEngine {
    state_path: PathBuf,
    current_state: WorkflowState,
    tx: broadcast::Sender<WorkflowState>,
}

impl WorkflowEngine {
    pub fn new(project_dir: PathBuf) -> Self {
        let state_path = project_dir.join(".workflow").join("state.json");
        let (tx, _) = broadcast::channel(32);

        WorkflowEngine {
            state_path,
            current_state: WorkflowState::default(),
            tx,
        }
    }

    /// Load workflow state from .workflow/state.json
    pub async fn load_state(&mut self) -> Result<WorkflowState> {
        let content = fs::read_to_string(&self.state_path).await?;
        self.current_state = serde_json::from_str(&content)?;
        tracing::info!(
            "Loaded workflow state: phase={}, epic={}",
            self.current_state.phase,
            self.current_state.epic
        );
        Ok(self.current_state.clone())
    }

    /// Save workflow state to .workflow/state.json
    pub async fn save_state(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.current_state)?;
        fs::write(&self.state_path, content).await?;
        tracing::info!("Saved workflow state");
        Ok(())
    }

    /// Persist state to disk and notify watchers
    async fn persist_and_notify(&self) -> Result<()> {
        self.save_state().await?;
        let _ = self.tx.send(self.current_state.clone());
        Ok(())
    }

    /// Get current phase
    #[allow(dead_code)]
    pub fn get_phase(&self) -> &str {
        &self.current_state.phase
    }

    /// Set phase and record transition
    pub async fn set_phase(&mut self, new_phase: String) -> Result<()> {
        let old_phase = self.current_state.phase.clone();
        self.current_state.record_transition(
            old_phase.clone(),
            new_phase.clone(),
            Some("manual transition".to_string()),
        );
        self.current_state.phase = new_phase;
        self.current_state.gate_ready = false;
        self.persist_and_notify().await?;
        tracing::info!("Phase changed: {} -> {}", old_phase, self.current_state.phase);
        Ok(())
    }

    /// Validate gate for current phase.
    ///
    /// In pipeline mode during implementation, additionally verifies that all
    /// tasks in current-epic.md are complete (DONE/VALIDATED) before allowing
    /// the gate to pass.
    pub async fn validate_gate(&mut self) -> Result<bool> {
        if !self.current_state.gate_ready {
            return Err(WorkflowError::InvalidPhase(
                "Cannot validate gate: work not complete (gate_ready is false)".to_string(),
            ));
        }

        // In pipeline implementation phase, verify task completion
        if self.current_state.mode == WorkflowMode::Pipeline
            && self.current_state.phase == "implementation"
        {
            let epic_path = self
                .state_path
                .parent()
                .and_then(|wf| wf.parent())
                .map(|proj| proj.join(".memory").join("current-epic.md"));

            if let Some(path) = epic_path {
                if let Ok(content) = fs::read_to_string(&path).await {
                    if has_active_tasks(&content) {
                        return Err(WorkflowError::InvalidPhase(
                            "Cannot validate gate: tasks still active in current-epic.md"
                                .to_string(),
                        ));
                    }
                }
            }
        }

        self.current_state.gate_validated = true;
        self.current_state.gate_ready = false;
        self.persist_and_notify().await?;
        tracing::info!("Gate validated for phase: {}", self.current_state.phase);
        Ok(true)
    }

    /// Move to next phase in the pipeline
    pub async fn next_phase(&mut self) -> Result<String> {
        let next = peek_next_phase(&self.current_state.phase)
            .ok_or_else(|| WorkflowError::InvalidPhase(self.current_state.phase.clone()))?;

        self.current_state.gate_validated = false;
        self.set_phase(next.to_string()).await?;
        Ok(next.to_string())
    }

    /// Set workflow mode (Free or Pipeline)
    pub async fn set_mode(&mut self, mode: WorkflowMode) -> Result<()> {
        self.current_state.mode = mode;
        self.persist_and_notify().await?;
        tracing::info!("Workflow mode changed to: {:?}", mode);
        Ok(())
    }

    /// Set permission mode (strict or accept-edits)
    pub async fn set_permission_mode(&mut self, mode: String) -> Result<()> {
        self.current_state.permission_mode = mode;
        self.persist_and_notify().await?;
        tracing::info!("Permission mode changed to: {}", self.current_state.permission_mode);
        Ok(())
    }

    /// Set policy profile
    pub async fn set_policy_profile(&mut self, profile: String) -> Result<()> {
        self.current_state.policy_profile = profile;
        self.persist_and_notify().await?;
        tracing::info!("Policy profile changed to: {}", self.current_state.policy_profile);
        Ok(())
    }

    /// Set epic
    #[allow(dead_code)]
    pub async fn set_epic(&mut self, name: String) -> Result<()> {
        self.current_state.epic = name;
        self.persist_and_notify().await?;
        Ok(())
    }

    /// Start a new epic: set name, transition to comprehension, reset gate.
    /// Single persist_and_notify call for atomicity.
    pub async fn start_epic(&mut self, name: String) -> Result<()> {
        let old_phase = self.current_state.phase.clone();
        self.current_state.epic = name;
        self.current_state.gate_validated = false;
        self.current_state.gate_ready = false;
        self.current_state.record_transition(
            old_phase,
            "comprehension".to_string(),
            Some("epic started".to_string()),
        );
        self.current_state.phase = "comprehension".to_string();
        self.persist_and_notify().await?;
        tracing::info!("Epic started: {}", self.current_state.epic);
        Ok(())
    }

    /// Set task
    #[allow(dead_code)]
    pub async fn set_task(&mut self, description: String) -> Result<()> {
        self.current_state.task = description;
        self.persist_and_notify().await?;
        Ok(())
    }

    /// Get pipeline for a task type
    #[allow(dead_code)]
    pub fn get_pipeline_for_task_type(&self, task_type: &str) -> Vec<&'static str> {
        match task_type {
            "bugfix" => vec!["comprehension", "implementation", "review", "closure"],
            "feature" => vec![
                "comprehension",
                "specification",
                "architecture",
                "implementation",
                "review",
                "test",
                "closure",
            ],
            "refactor" => vec!["specification", "implementation", "review", "test", "closure"],
            _ => vec![
                "idle",
                "comprehension",
                "specification",
                "architecture",
                "implementation",
                "review",
                "test",
                "closure",
            ],
        }
    }

    /// Watch state file for changes (returns broadcast receiver)
    #[allow(dead_code)]
    pub fn watch_state_file(&self) -> broadcast::Receiver<WorkflowState> {
        self.tx.subscribe()
    }

    /// Get current state
    pub fn get_state(&self) -> &WorkflowState {
        &self.current_state
    }

    /// Get mutable current state
    #[allow(dead_code)]
    pub fn get_state_mut(&mut self) -> &mut WorkflowState {
        &mut self.current_state
    }

    /// Set gate_ready flag and persist
    pub async fn set_gate_ready(&mut self, ready: bool) -> Result<()> {
        self.current_state.gate_ready = ready;
        self.persist_and_notify().await?;
        tracing::info!("Gate ready set to: {}", ready);
        Ok(())
    }

    /// Attempt to mark gate as ready after a successful CLI turn.
    /// Returns Ok(true) if gate_ready was set, Ok(false) if skipped (with reason logged).
    pub async fn try_mark_gate_ready_after_turn(&mut self, is_error: bool) -> Result<bool> {
        if is_error {
            tracing::info!("gate_ready skip: CLI result was an error");
            return Ok(false);
        }

        let state = &self.current_state;

        if state.phase == "idle" {
            tracing::info!("gate_ready skip: phase is idle");
            return Ok(false);
        }

        if state.mode != WorkflowMode::Pipeline {
            tracing::info!("gate_ready skip: mode is {:?}, not pipeline", state.mode);
            return Ok(false);
        }

        if state.gate_validated {
            tracing::info!("gate_ready skip: gate already validated");
            return Ok(false);
        }

        if state.gate_ready {
            tracing::info!("gate_ready skip: already ready");
            return Ok(false);
        }

        self.set_gate_ready(true).await?;
        tracing::info!(
            "gate_ready set to true: phase={}, epic={}",
            self.current_state.phase,
            self.current_state.epic
        );
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = WorkflowEngine::new(PathBuf::from("/tmp"));
        assert_eq!(engine.get_phase(), "idle");
    }

    #[test]
    fn test_pipeline_for_task_type() {
        let engine = WorkflowEngine::new(PathBuf::from("/tmp"));
        let bugfix_pipeline = engine.get_pipeline_for_task_type("bugfix");
        assert_eq!(bugfix_pipeline.len(), 4);
        assert!(bugfix_pipeline.contains(&"implementation"));
    }

    #[tokio::test]
    async fn test_validate_gate_requires_gate_ready() {
        let dir = tempfile::tempdir().expect("tempdir");
        let workflow_dir = dir.path().join(".workflow");
        std::fs::create_dir_all(&workflow_dir).expect("create workflow dir");
        std::fs::write(
            workflow_dir.join("state.json"),
            r#"{"phase":"implementation","epic":"test","task":"","mode":"pipeline","gate_validated":false,"last_updated":"2026-01-01T00:00:00Z","history":[],"session":null}"#,
        ).expect("write state");

        let mut engine = WorkflowEngine::new(dir.path().to_path_buf());
        engine.load_state().await.expect("load");

        // gate_ready defaults to false — validation should fail
        let result = engine.validate_gate().await;
        assert!(result.is_err());

        // Set gate_ready and try again
        engine.set_gate_ready(true).await.expect("set gate_ready");
        let result = engine.validate_gate().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_gate_ready_success_pipeline_active() {
        let dir = tempfile::tempdir().expect("tempdir");
        let workflow_dir = dir.path().join(".workflow");
        std::fs::create_dir_all(&workflow_dir).expect("create workflow dir");
        std::fs::write(
            workflow_dir.join("state.json"),
            r#"{"phase":"implementation","epic":"test","task":"","mode":"pipeline","gate_validated":false,"gate_ready":false,"last_updated":"2026-01-01T00:00:00Z","history":[],"session":null}"#,
        ).expect("write state");

        let mut engine = WorkflowEngine::new(dir.path().to_path_buf());
        engine.load_state().await.expect("load");

        let result = engine.try_mark_gate_ready_after_turn(false).await.expect("try_mark");
        assert!(result, "should set gate_ready on success + pipeline + active phase");
        assert!(engine.get_state().gate_ready);
    }

    #[tokio::test]
    async fn test_gate_ready_skip_idle_phase() {
        let dir = tempfile::tempdir().expect("tempdir");
        let workflow_dir = dir.path().join(".workflow");
        std::fs::create_dir_all(&workflow_dir).expect("create workflow dir");
        std::fs::write(
            workflow_dir.join("state.json"),
            r#"{"phase":"idle","epic":"","task":"","mode":"pipeline","gate_validated":false,"gate_ready":false,"last_updated":"2026-01-01T00:00:00Z","history":[],"session":null}"#,
        ).expect("write state");

        let mut engine = WorkflowEngine::new(dir.path().to_path_buf());
        engine.load_state().await.expect("load");

        let result = engine.try_mark_gate_ready_after_turn(false).await.expect("try_mark");
        assert!(!result, "should skip when phase is idle");
        assert!(!engine.get_state().gate_ready);
    }

    #[tokio::test]
    async fn test_gate_ready_skip_error_result() {
        let dir = tempfile::tempdir().expect("tempdir");
        let workflow_dir = dir.path().join(".workflow");
        std::fs::create_dir_all(&workflow_dir).expect("create workflow dir");
        std::fs::write(
            workflow_dir.join("state.json"),
            r#"{"phase":"implementation","epic":"test","task":"","mode":"pipeline","gate_validated":false,"gate_ready":false,"last_updated":"2026-01-01T00:00:00Z","history":[],"session":null}"#,
        ).expect("write state");

        let mut engine = WorkflowEngine::new(dir.path().to_path_buf());
        engine.load_state().await.expect("load");

        let result = engine.try_mark_gate_ready_after_turn(true).await.expect("try_mark");
        assert!(!result, "should skip when result is error");
        assert!(!engine.get_state().gate_ready);
    }

    #[tokio::test]
    async fn test_gate_ready_skip_already_validated() {
        let dir = tempfile::tempdir().expect("tempdir");
        let workflow_dir = dir.path().join(".workflow");
        std::fs::create_dir_all(&workflow_dir).expect("create workflow dir");
        std::fs::write(
            workflow_dir.join("state.json"),
            r#"{"phase":"implementation","epic":"test","task":"","mode":"pipeline","gate_validated":true,"gate_ready":false,"last_updated":"2026-01-01T00:00:00Z","history":[],"session":null}"#,
        ).expect("write state");

        let mut engine = WorkflowEngine::new(dir.path().to_path_buf());
        engine.load_state().await.expect("load");

        let result = engine.try_mark_gate_ready_after_turn(false).await.expect("try_mark");
        assert!(!result, "should skip when gate already validated");
        assert!(!engine.get_state().gate_ready);
    }

    #[tokio::test]
    async fn test_gate_ready_skip_free_mode() {
        let dir = tempfile::tempdir().expect("tempdir");
        let workflow_dir = dir.path().join(".workflow");
        std::fs::create_dir_all(&workflow_dir).expect("create workflow dir");
        std::fs::write(
            workflow_dir.join("state.json"),
            r#"{"phase":"implementation","epic":"test","task":"","mode":"free","gate_validated":false,"gate_ready":false,"last_updated":"2026-01-01T00:00:00Z","history":[],"session":null}"#,
        ).expect("write state");

        let mut engine = WorkflowEngine::new(dir.path().to_path_buf());
        engine.load_state().await.expect("load");

        let result = engine.try_mark_gate_ready_after_turn(false).await.expect("try_mark");
        assert!(!result, "should skip when mode is free (not pipeline)");
        assert!(!engine.get_state().gate_ready);
    }
}
