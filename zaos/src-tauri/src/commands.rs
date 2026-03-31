use crate::deployer;
use crate::events::CliEvent;
use crate::memory::{MemoryReader, MemoryStateResponse};
use crate::screenshots::{FilesystemAdapter, Screenshot, ScreenshotOrchestrator};
use crate::session::{CliSession, SessionManager};
use crate::watchers::FileWatcherService;
use crate::workflow::{WorkflowEngine, WorkflowMode};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::{Mutex, RwLock};

/// Shared app state
pub struct AppState {
    pub session_manager: Arc<Mutex<SessionManager>>,
    pub workflow_engine: Arc<Mutex<WorkflowEngine>>,
    pub watcher_service: Arc<Mutex<FileWatcherService>>,
    pub screenshot_orchestrator: Arc<Mutex<ScreenshotOrchestrator>>,
    pub project_dir: Arc<RwLock<PathBuf>>,
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
            watcher_service: Arc::new(Mutex::new(
                FileWatcherService::new(project_dir.clone()),
            )),
            screenshot_orchestrator: Arc::new(Mutex::new(
                ScreenshotOrchestrator::new(
                    project_dir.join(".screenshots"),
                    Box::new(FilesystemAdapter),
                ),
            )),
            project_dir: Arc::new(RwLock::new(project_dir)),
        }
    }

    pub async fn project_dir(&self) -> PathBuf {
        self.project_dir.read().await.clone()
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
    pub sessions: Vec<CliSession>,
}

// =============================================================================
// Tauri Commands
// =============================================================================

/// Send a prompt to Claude Code CLI.
/// Starts a long-lived CLI session if not already running, then sends the message via stdin.
/// Forwards all parsed CLI events to the frontend via app.emit("agent-event").
#[tauri::command]
pub async fn send_prompt(
    text: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SendPromptResponse, String> {
    tracing::info!("send_prompt called with: {}", &text);

    let session_manager = state.session_manager.clone();

    // Single lock scope: start session if needed, then send message
    {
        let mut session = session_manager.lock().await;
        if !session.is_session_started() {
            let mut rx = session
                .start_session()
                .await
                .map_err(|e| format!("Failed to start CLI session: {}", e))?;

            // Spawn event forwarder (runs for the lifetime of the session)
            let sm = session_manager.clone();
            let app_handle = app.clone();
            tokio::spawn(async move {
                tracing::info!("Event forwarder started");

                while let Ok(event) = rx.recv().await {
                    tracing::debug!("Forwarding event to frontend: {:?}", event);

                    // Extract session_id from system init events
                    if let CliEvent::System(ref sys) = event {
                        let mut mgr = sm.lock().await;
                        mgr.set_session_id(sys.session_id.clone());
                        tracing::info!("Session ID captured: {}", sys.session_id);
                    }

                    // Emit to frontend
                    if let Err(e) = app_handle.emit("agent-event", &event) {
                        tracing::error!("Failed to emit event: {}", e);
                    }
                }

                tracing::info!("CLI session ended");
            });
        }

        session
            .send_message(&text)
            .await
            .map_err(|e| format!("Failed to send message: {}", e))?;
    }

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
    let mut session = state.session_manager.lock().await;
    session
        .interrupt()
        .await
        .map_err(|e| format!("Failed to interrupt: {}", e))?;
    Ok(InterruptResponse {
        success: true,
        message: "Session interrupted".to_string(),
    })
}
#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionResponse {
    pub success: bool,
    pub message: String,
}

/// Respond to a pending permission prompt from the CLI
#[tauri::command]
pub async fn respond_permission(
    id: String,
    allow: bool,
    tool_input: Option<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<PermissionResponse, String> {
    tracing::info!("respond_permission called: id={}, allow={}", id, allow);
    let mut session = state.session_manager.lock().await;
    session
        .send_permission_response(&id, allow, tool_input)
        .await
        .map_err(|e| format!("Failed to send permission response: {}", e))?;
    Ok(PermissionResponse {
        success: true,
        message: format!(
            "Permission {} for {}",
            if allow { "granted" } else { "denied" },
            id
        ),
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
    let workflow = state.workflow_engine.lock().await;
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

/// Save a session log to .memory/sessions/
#[tauri::command]
pub async fn save_session_log(
    state: State<'_, AppState>,
    data: crate::session::logger::SessionLogData,
) -> Result<(), String> {
    let project_dir = state.project_dir().await;
    crate::session::logger::write_session_log(&project_dir, &data)
        .await
        .map_err(|e| e.to_string())
}

/// Get current memory state (index + state + current epic)
#[tauri::command]
pub async fn get_memory_state(
    state: State<'_, AppState>,
) -> Result<MemoryStateResponse, String> {
    tracing::info!("get_memory_state called");
    let project_dir = state.project_dir().await;
    let reader = MemoryReader::new(project_dir);
    reader.read_all().await
}

// =============================================================================
// Screenshot Response Types
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct GetScreenshotsResponse {
    pub screenshots: Vec<Screenshot>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenshotActionResponse {
    pub success: bool,
    pub message: String,
}

// =============================================================================
// Screenshot Commands
// =============================================================================

/// List all screenshots, optionally filtered by session_id
#[tauri::command]
pub async fn get_screenshots(
    session_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<GetScreenshotsResponse, String> {
    tracing::info!("get_screenshots called, session_id={:?}", session_id);
    let orch = state.screenshot_orchestrator.lock().await;
    let screenshots = orch
        .get_gallery(session_id.as_deref())
        .await
        .map_err(|e| format!("Failed to get screenshots: {}", e))?;
    Ok(GetScreenshotsResponse { screenshots })
}

/// Import a screenshot from a file path
#[tauri::command]
pub async fn add_screenshot(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<Screenshot, String> {
    tracing::info!("add_screenshot called, file_path={}", file_path);
    let path = std::path::Path::new(&file_path);
    let orch = state.screenshot_orchestrator.lock().await;
    orch.index_file(path)
        .await
        .map_err(|e| format!("Failed to add screenshot: {}", e))
}

/// Delete a screenshot by id
#[tauri::command]
pub async fn delete_screenshot(
    id: String,
    state: State<'_, AppState>,
) -> Result<ScreenshotActionResponse, String> {
    tracing::info!("delete_screenshot called, id={}", id);
    let orch = state.screenshot_orchestrator.lock().await;
    let found = orch
        .delete_screenshot(&id)
        .await
        .map_err(|e| format!("Failed to delete screenshot: {}", e))?;
    Ok(ScreenshotActionResponse {
        success: found,
        message: if found {
            "Deleted".into()
        } else {
            "Not found".into()
        },
    })
}

/// Request a new capture via the active adapter
#[tauri::command]
pub async fn request_capture(
    url: Option<String>,
    iteration_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<ScreenshotActionResponse, String> {
    tracing::info!("request_capture called, url={:?}, iteration_id={:?}", url, iteration_id);
    let orch = state.screenshot_orchestrator.lock().await;
    orch.request_capture(url, None, iteration_id)
        .await
        .map_err(|e| format!("Failed to request capture: {}", e))?;
    Ok(ScreenshotActionResponse {
        success: true,
        message: "Capture requested".into(),
    })
}

// =============================================================================
// Deployer Commands
// =============================================================================

#[derive(Debug, Serialize)]
pub struct WorkflowKitStatus {
    pub deployed: bool,
    pub agent_count: usize,
    pub rule_count: usize,
    pub hooks_active: bool,
    pub last_deployed: Option<String>,
    pub config: deployer::config::WorkflowKitConfig,
}

/// Deploy workflow kit: sync agents, rules, settings, and hooks binary
#[tauri::command]
pub async fn deploy_workflow_kit(
    state: State<'_, AppState>,
) -> Result<deployer::sync::SyncReport, String> {
    tracing::info!("deploy_workflow_kit called");
    let project_dir = state.project_dir().await;
    deployer::deploy(&project_dir).map_err(|e| e.to_string())
}

/// Get the current status of deployed workflow kit components
#[tauri::command]
pub async fn get_workflow_kit_status(
    state: State<'_, AppState>,
) -> Result<WorkflowKitStatus, String> {
    tracing::info!("get_workflow_kit_status called");
    let project_dir = state.project_dir().await;
    let config = deployer::config::WorkflowKitConfig::load(&project_dir);
    let manifest = deployer::sync::DeployManifest::load(&project_dir);

    let agents_dir = project_dir.join(".claude").join("agents");
    let agent_count = std::fs::read_dir(&agents_dir)
        .map(|entries| entries.filter_map(|e| e.ok()).count())
        .unwrap_or(0);

    let rules_dir = project_dir.join(".claude").join("rules");
    let rule_count = std::fs::read_dir(&rules_dir)
        .map(|entries| entries.filter_map(|e| e.ok()).count())
        .unwrap_or(0);

    let settings_exists = project_dir.join(".claude").join("settings.json").exists();

    Ok(WorkflowKitStatus {
        deployed: agent_count > 0,
        agent_count,
        rule_count,
        hooks_active: settings_exists && config.hooks_enabled,
        last_deployed: manifest.last_deployed,
        config,
    })
}

/// Update the workflow kit configuration
#[tauri::command]
pub async fn update_workflow_kit_config(
    state: State<'_, AppState>,
    config: deployer::config::WorkflowKitConfig,
) -> Result<(), String> {
    tracing::info!("update_workflow_kit_config called");
    let project_dir = state.project_dir().await;
    config.save(&project_dir).map_err(|e| e.to_string())
}

// =============================================================================
// Agent CRUD Commands
// =============================================================================

#[derive(Debug, Serialize)]
pub struct AgentInfo {
    pub name: String,
    pub description: String,
}

/// List all agent files in .claude/agents/ with name and first-line description
#[tauri::command]
pub async fn list_agents(
    state: State<'_, AppState>,
) -> Result<Vec<AgentInfo>, String> {
    let project_dir = state.project_dir().await;
    let agents_dir = project_dir.join(".claude").join("agents");
    let mut agents = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&agents_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "md") {
                let name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                let description = std::fs::read_to_string(&path)
                    .ok()
                    .and_then(|c| c.lines().next().map(|l| l.to_string()))
                    .unwrap_or_default();
                agents.push(AgentInfo { name, description });
            }
        }
    }

    agents.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(agents)
}

/// Read a single agent file
#[tauri::command]
pub async fn read_agent(
    state: State<'_, AppState>,
    name: String,
) -> Result<String, String> {
    tracing::info!("read_agent called: {}", name);
    let project_dir = state.project_dir().await;
    let path = project_dir.join(".claude").join("agents").join(format!("{}.md", name));
    std::fs::read_to_string(&path).map_err(|e| format!("Failed to read agent {}: {}", name, e))
}

/// Create or update an agent file
#[tauri::command]
pub async fn save_agent(
    state: State<'_, AppState>,
    name: String,
    content: String,
) -> Result<(), String> {
    tracing::info!("save_agent called: {}", name);
    let project_dir = state.project_dir().await;
    let agents_dir = project_dir.join(".claude").join("agents");
    std::fs::create_dir_all(&agents_dir).map_err(|e| e.to_string())?;
    let path = agents_dir.join(format!("{}.md", name));
    std::fs::write(&path, content).map_err(|e| format!("Failed to save agent {}: {}", name, e))?;
    tracing::info!("Agent saved: {}", name);
    Ok(())
}

/// Delete an agent file
#[tauri::command]
pub async fn delete_agent(
    state: State<'_, AppState>,
    name: String,
) -> Result<(), String> {
    tracing::info!("delete_agent called: {}", name);
    let project_dir = state.project_dir().await;
    let path = project_dir.join(".claude").join("agents").join(format!("{}.md", name));
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("Failed to delete agent {}: {}", name, e))?;
        tracing::info!("Agent deleted: {}", name);
    }
    Ok(())
}

// =============================================================================
// Project Commands
// =============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectInfo {
    pub path: String,
    pub name: String,
}

impl ProjectInfo {
    fn from_path(dir: &std::path::Path) -> Self {
        let path = dir.to_string_lossy().to_string();
        let name = dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone());
        Self { path, name }
    }
}

/// Get current project info (path and name)
#[tauri::command]
pub async fn get_project_info(
    state: State<'_, AppState>,
) -> Result<ProjectInfo, String> {
    tracing::info!("get_project_info called");
    let project_dir = state.project_dir().await;
    Ok(ProjectInfo::from_path(&project_dir))
}

/// Switch to a different project directory
#[tauri::command]
pub async fn switch_project(
    path: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ProjectInfo, String> {
    tracing::info!("switch_project called with path: {}", path);

    let new_dir = PathBuf::from(&path);

    // 1. Validate path is a directory
    if !new_dir.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }

    // 2. Kill session if running
    state
        .session_manager
        .lock()
        .await
        .interrupt()
        .await
        .map_err(|e| format!("Failed to interrupt session: {}", e))?;

    // 3. Stop watchers
    state.watcher_service.lock().await.stop();

    // 4. Init new project dirs
    crate::init::ensure_project_dirs(&new_dir);

    // 5. Replace session_manager
    *state.session_manager.lock().await = SessionManager::new(new_dir.clone());

    // 6. Replace workflow_engine
    *state.workflow_engine.lock().await = WorkflowEngine::new(new_dir.clone());

    // 7. Replace screenshot_orchestrator
    *state.screenshot_orchestrator.lock().await = ScreenshotOrchestrator::new(
        new_dir.join(".screenshots"),
        Box::new(FilesystemAdapter),
    );

    // 8. Update project_dir (after all services are replaced)
    *state.project_dir.write().await = new_dir.clone();

    // 9. Restart watchers
    let mut watcher = state.watcher_service.lock().await;
    *watcher = FileWatcherService::new(new_dir.clone());
    watcher
        .start(app.clone(), state.screenshot_orchestrator.clone())
        .map_err(|e| format!("Failed to start watchers: {}", e))?;

    // 10. Build ProjectInfo
    let info = ProjectInfo::from_path(&new_dir);

    // 11. Emit project-changed event
    app.emit("project-changed", &info)
        .map_err(|e| format!("Failed to emit project-changed: {}", e))?;

    tracing::info!("Project switched to: {}", info.path);
    Ok(info)
}