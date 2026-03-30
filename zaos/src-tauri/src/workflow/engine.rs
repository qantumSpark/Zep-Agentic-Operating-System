use crate::workflow::state::{WorkflowMode, WorkflowState};
use std::path::PathBuf;
use thiserror::Error;
use tokio::fs;
use tokio::sync::broadcast;

#[derive(Error, Debug)]
pub enum WorkflowError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid phase: {0}")]
    InvalidPhase(String),

    #[error("State file not found")]
    StateNotFound,

    #[error("Watch error: {0}")]
    WatchError(String),
}

pub type Result<T> = std::result::Result<T, WorkflowError>;

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
        self.persist_and_notify().await?;
        tracing::info!("Phase changed: {} -> {}", old_phase, self.current_state.phase);
        Ok(())
    }

    /// Validate gate for current phase
    pub async fn validate_gate(&mut self) -> Result<bool> {
        self.current_state.gate_validated = true;
        self.persist_and_notify().await?;
        tracing::info!("Gate validated for phase: {}", self.current_state.phase);
        Ok(true)
    }

    /// Move to next phase in the pipeline
    pub async fn next_phase(&mut self) -> Result<String> {
        let next = match self.current_state.phase.as_str() {
            "idle" => "comprehension",
            "comprehension" => "specification",
            "specification" => "architecture",
            "architecture" => "implementation",
            "implementation" => "review",
            "review" => "test",
            "test" => "closure",
            "closure" => "idle",
            phase => return Err(WorkflowError::InvalidPhase(phase.to_string())),
        };

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

    /// Set epic
    pub async fn set_epic(&mut self, name: String) -> Result<()> {
        self.current_state.epic = name;
        self.persist_and_notify().await?;
        Ok(())
    }

    /// Set task
    pub async fn set_task(&mut self, description: String) -> Result<()> {
        self.current_state.task = description;
        self.persist_and_notify().await?;
        Ok(())
    }

    /// Get pipeline for a task type
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
    pub fn watch_state_file(&self) -> broadcast::Receiver<WorkflowState> {
        self.tx.subscribe()
    }

    /// Get current state
    pub fn get_state(&self) -> &WorkflowState {
        &self.current_state
    }

    /// Get mutable current state
    pub fn get_state_mut(&mut self) -> &mut WorkflowState {
        &mut self.current_state
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
}
