use crate::events::CliEvent;
use crate::session::SessionManager;
use crate::workflow::{WorkflowEngine, WorkflowMode};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;

/// Shared app state
pub struct AppState {
    pub session_manager: Arc<Mutex<SessionManager>>,
    pub workflow_engine: Arc<Mutex<WorkflowEngine>>,
    pub project_dir: PathBuf,
}

impl AppState {
    pub fn new(project_dir: PathBuf) -> Self {
        AppState {
            session_manager: Arc::new(Mutex::new(
                SessionManager::new(project_dir.clone()),
            )),
            workflow_engine: Arc::new(Mutex::new(
                WorkflowEngine::new(project_dir.clone()),
            )),
            project_dir,
        }
    }
}
// =============================================================================
// Response Types
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct SendPromptResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InterruptResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateGateResponse {
    pub success: bool,
    pub next_phase: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetModeResponse {
    pub success: bool,
    pub mode: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowStateResponse {
    pub phase: String,
    pub epic: String,
    pub task: String,
    pub mode: String,
    pub gate_validated: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckAuthResponse {
    pub authenticated: bool,
    pub version: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListSessionsResponse {
    pub sessions: Vec<String>,
}

// =============================================================================
// Tauri Commands
// =============================================================================

/// Send a prompt to Claude Code CLI.
/// Spawns a new process per message (with --resume for continuity).
/// Forwards all parsed CLI events to the frontend via app.emit("agent-event").
#[tauri::command]
pub async fn send_prompt(
    text: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SendPromptResponse, String> {
    tracing::info!("send_prompt called with: {}", &text);

    // Spawn CLI process for this prompt
    let (mut child, mut rx) = {
        let mut session = state.session_manager.lock().await;
        session
            .spawn_for_prompt(&text)
            .await
            .map_err(|e| format!("Failed to spawn CLI: {}", e))?
    };

    // Spawn a background task to forward events to the frontend
    let session_manager = state.session_manager.clone();
    tokio::spawn(async move {
        tracing::info!("Event forwarder started");

        while let Ok(event) = rx.recv().await {
            tracing::debug!("Forwarding event to frontend: {:?}", event);

            // Extract session_id from system init events
            if let CliEvent::System(ref sys) = event {
                let mut mgr = session_manager.lock().await;
                mgr.set_session_id(sys.session_id.clone());
                tracing::info!("Session ID captured: {}", sys.session_id);
            }
            // Emit to frontend
            if let Err(e) = app.emit("agent-event", &event) {
                tracing::error!("Failed to emit event: {}", e);
            }
        }

        // CLI process finished — wait for exit
        let _ = child.wait().await;
        let mut mgr = session_manager.lock().await;
        mgr.set_running(false);
        tracing::info!("CLI process finished");
    });

    Ok(SendPromptResponse {
        success: true,
        message: "Prompt sent".to_string(),
    })
}

/// Interrupt the current CLI session (placeholder)
#[tauri::command]
pub async fn interrupt_session(
    state: State<'_, AppState>,
) -> Result<InterruptResponse, String> {
    tracing::info!("interrupt_session called");
    Ok(InterruptResponse {
        success: true,
        message: "Session interrupted".to_string(),
    })
}
/// Validate gate and move to next phase
#[tauri::command]
pub async fn validate_gate(
    state: State<'_, AppState>,
) -> Result<ValidateGateResponse, String> {
    let mut workflow = state.workflow_engine.lock().await;
    workflow
        .validate_gate()
        .await
        .map_err(|e| format!("Failed to validate gate: {}", e))?;
    let next_phase = workflow
        .next_phase()
        .await
        .map_err(|e| format!("Failed to advance phase: {}", e))?;
    Ok(ValidateGateResponse {
        success: true,
        next_phase,
        message: "Gate validated".to_string(),
    })
}

/// Set workflow mode (free or pipeline)
#[tauri::command]
pub async fn set_mode(
    mode: String,
    state: State<'_, AppState>,
) -> Result<SetModeResponse, String> {
    let mut workflow = state.workflow_engine.lock().await;
    let workflow_mode = match mode.as_str() {
        "free" => WorkflowMode::Free,
        "pipeline" => WorkflowMode::Pipeline,
        _ => return Err("Invalid mode".to_string()),
    };
    workflow
        .set_mode(workflow_mode)
        .await
        .map_err(|e| format!("Failed to set mode: {}", e))?;
    Ok(SetModeResponse {
        success: true,
        mode: mode.to_string(),
    })
}

/// Get current workflow state
#[tauri::command]
pub async fn get_workflow_state(
    state: State<'_, AppState>,
) -> Result<WorkflowStateResponse, String> {
    let mut workflow = state.workflow_engine.lock().await;
    workflow
        .load_state()
        .await
        .map_err(|e| format!("Failed to load state: {}", e))?;
    let wf_state = workflow.get_state();
    Ok(WorkflowStateResponse {
        phase: wf_state.phase.clone(),
        epic: wf_state.epic.clone(),
        task: wf_state.task.clone(),
        mode: format!("{:?}", wf_state.mode).to_lowercase(),
        gate_validated: wf_state.gate_validated,
    })
}
/// Check CLI authentication status
#[tauri::command]
pub async fn check_cli_auth() -> Result<CheckAuthResponse, String> {
    match SessionManager::check_cli_auth().await {
        Ok(version) => Ok(CheckAuthResponse {
            authenticated: true,
            version: version.trim().to_string(),
            message: "Claude CLI authenticated".to_string(),
        }),
        Err(_) => Ok(CheckAuthResponse {
            authenticated: false,
            version: String::new(),
            message: "CLI not found or not authenticated".to_string(),
        }),
    }
}

/// List available sessions
#[tauri::command]
pub async fn list_sessions(
    state: State<'_, AppState>,
) -> Result<ListSessionsResponse, String> {
    let session = state.session_manager.lock().await;
    let sessions = session
        .list_sessions()
        .await
        .map_err(|e| format!("Failed to list sessions: {}", e))?;
    Ok(ListSessionsResponse { sessions })
}