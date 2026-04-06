use crate::deployer;
use crate::events::zaos_events::ZaosEvent;
use crate::mcp_server::McpServerHandle;
use crate::memory::{MemoryHealthReport, MemoryReader, MemoryStateResponse, Persona, SessionInsightsEditorial};
use crate::runtime::{RuntimeKind, RuntimePaths};
use crate::screenshots::{FilesystemAdapter, Screenshot, ScreenshotOrchestrator};
use crate::session::{CliSession, SessionManager};
use crate::watchers::FileWatcherService;
use crate::workflow::{WorkflowEngine, WorkflowMode, WorkflowStateDto};
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
    pub permission_mode: Arc<RwLock<String>>,
    /// Runtime kind, wrapped in Arc<RwLock<>> to allow future runtime switching.
    pub runtime_kind: Arc<RwLock<RuntimeKind>>,
    pub runtime_paths: Arc<RwLock<RuntimePaths>>,
    /// MCP server handle for shutdown/restart lifecycle.
    pub mcp_handle: Arc<Mutex<Option<McpServerHandle>>>,
}

impl AppState {
    pub fn new(project_dir: PathBuf) -> Self {
        let runtime_kind = RuntimeKind::default(); // Claude
        let runtime_paths = RuntimePaths::for_kind(&project_dir, &runtime_kind);
        AppState {
            session_manager: Arc::new(Mutex::new(
                SessionManager::new(project_dir.clone(), runtime_kind),
            )),
            workflow_engine: Arc::new(Mutex::new(
                WorkflowEngine::new(project_dir.clone()),
            )),
            watcher_service: Arc::new(Mutex::new(
                FileWatcherService::new(project_dir.clone(), runtime_paths.clone()),
            )),
            screenshot_orchestrator: Arc::new(Mutex::new(
                ScreenshotOrchestrator::new(
                    project_dir.join(".screenshots"),
                    Box::new(FilesystemAdapter),
                ),
            )),
            project_dir: Arc::new(RwLock::new(project_dir)),
            permission_mode: Arc::new(RwLock::new("strict".to_string())),
            runtime_kind: Arc::new(RwLock::new(runtime_kind)),
            runtime_paths: Arc::new(RwLock::new(runtime_paths)),
            mcp_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn project_dir(&self) -> PathBuf {
        self.project_dir.read().await.clone()
    }

    pub async fn runtime_paths(&self) -> RuntimePaths {
        self.runtime_paths.read().await.clone()
    }
}
/// Validate that a resource name contains only safe characters (no path traversal)
fn validate_safe_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") || name.contains('\0') {
        return Err(format!("Invalid name '{}': contains path traversal characters", name));
    }
    if name.starts_with('.') {
        return Err(format!("Invalid name '{}': cannot start with '.'", name));
    }
    Ok(())
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
pub struct CheckAuthResponse {
    pub authenticated: bool,
    pub version: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListSessionsResponse {
    pub sessions: Vec<CliSession>,
}

#[derive(Debug, Serialize)]
pub struct SetPolicyProfileResponse {
    pub success: bool,
    pub profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeInfo {
    pub kind: RuntimeKind,
    pub name: String,
    pub paths: RuntimePaths,
}

// =============================================================================
// Tauri Commands
// =============================================================================

/// Send a prompt to the active runtime CLI session.
/// Starts a long-lived CLI session if not already running, then sends the message via stdin.
/// Forwards all parsed CLI events to the frontend via app.emit("agent-event").
#[tauri::command]
pub async fn send_prompt(
    text: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SendPromptResponse, String> {
    tracing::info!("send_prompt called with: {}", &text);

    // Reset gate_ready — new prompt means new work
    {
        let mut engine = state.workflow_engine.lock().await;
        let wf_state = engine.get_state();
        if wf_state.gate_ready {
            tracing::info!("Resetting gate_ready to false: new prompt received");
            if let Err(e) = engine.set_gate_ready(false).await {
                tracing::warn!("Failed to reset gate_ready: {}", e);
            }
        }
    }

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
            let pm = state.permission_mode.clone();
            let we = state.workflow_engine.clone();
            let app_handle = app.clone();
            tokio::spawn(async move {
                tracing::info!("Event forwarder started");

                while let Ok(event) = rx.recv().await {
                    tracing::debug!("Forwarding event to frontend: {:?}", event);

                    // Extract session_id from system init events
                    if let ZaosEvent::SessionInit { ref session_id, .. } = event {
                        let mut mgr = sm.lock().await;
                        mgr.set_session_id(session_id.clone());
                        tracing::info!("Session ID captured: {}", session_id);
                    }

                    // Policy Engine evaluation for permission requests
                    if let ZaosEvent::ApprovalRequested { ref request_id, ref tool_name, ref tool_input, ref description, .. } = event {
                        use crate::policy::{derive_action_type, is_destructive_command, evaluate as policy_evaluate, ActionContext, PolicyProfile, Verdict, verdict_to_str, risk_level_to_str};
                        use crate::workflow::product_phase::derive_product_phase;

                        let tool_name_str = tool_name.as_deref().unwrap_or("unknown").to_string();
                        let tool_input_val = tool_input.clone().unwrap_or(serde_json::Value::Null);

                        // 1. Build ActionContext
                        let mut action_type = derive_action_type(&tool_name_str);

                        // Extract file_paths from tool_input
                        let file_paths = {
                            let mut paths = Vec::new();
                            // Write/Edit: single file_path
                            if let Some(fp) = tool_input_val.get("file_path").and_then(|v| v.as_str()) {
                                paths.push(fp.to_string());
                            }
                            // MultiEdit: edits array
                            if let Some(edits) = tool_input_val.get("edits").and_then(|v| v.as_array()) {
                                for edit in edits {
                                    if let Some(fp) = edit.get("file_path").and_then(|v| v.as_str()) {
                                        paths.push(fp.to_string());
                                    }
                                }
                            }
                            paths
                        };

                        let is_destructive = is_destructive_command(&tool_input_val);

                        // Extract command_text for Bash tools (used by is_test_command)
                        let command_text = if tool_name_str == "Bash" {
                            tool_input_val.get("command").and_then(|v| v.as_str()).map(|s| s.to_string())
                        } else {
                            None
                        };

                        // Override action_type: destructive Bash commands → FileDelete
                        // so they receive the stricter FileDelete policy verdict
                        if action_type == crate::policy::ActionType::BashCommand && is_destructive {
                            action_type = crate::policy::ActionType::FileDelete;
                        }

                        // Get product_phase and policy profile from workflow state (single lock)
                        let (product_phase, profile) = {
                            let engine = we.lock().await;
                            let wf_state = engine.get_state();
                            let pp = derive_product_phase(wf_state);
                            let prof = PolicyProfile::from_str(&wf_state.policy_profile).unwrap_or_default();
                            (pp, prof)
                        };

                        let ctx = ActionContext {
                            action_type,
                            file_paths,
                            tool_name: Some(tool_name_str.clone()),
                            command_text,
                            product_phase,
                            is_destructive,
                            is_reversible: !is_destructive,
                        };

                        let decision = policy_evaluate(&profile, &ctx);

                        // 2. Derive permission_mode verdict
                        let pm_verdict = {
                            let mode = pm.read().await;
                            match mode.as_str() {
                                "accept-edits" => {
                                    const AUTO_APPROVE_TOOLS: &[&str] = &["Write", "Edit", "MultiEdit", "WebSearch", "WebFetch"];
                                    if AUTO_APPROVE_TOOLS.contains(&tool_name_str.as_str()) || tool_name_str.starts_with("mcp__") {
                                        Verdict::Allow
                                    } else {
                                        Verdict::Ask
                                    }
                                }
                                _ => Verdict::Ask, // "strict" or anything else -> Ask
                            }
                        };

                        // 3. Final verdict = most restrictive (Deny > Ask > Allow)
                        let final_verdict = crate::policy::max_verdict(decision.verdict, pm_verdict);

                        tracing::info!(
                            "Policy decision: tool={}, verdict={}, risk={}, reason={}, profile={}, pm_verdict={}, final={}",
                            tool_name_str, verdict_to_str(decision.verdict), risk_level_to_str(decision.risk_level), decision.reason, profile, verdict_to_str(pm_verdict), verdict_to_str(final_verdict)
                        );

                        match final_verdict {
                            Verdict::Allow => {
                                // Emit PolicyDecision event for frontend log, then auto-approve
                                let log_event = ZaosEvent::PolicyDecision {
                                    tool_name: Some(tool_name_str.clone()),
                                    verdict: verdict_to_str(decision.verdict).to_string(),
                                    risk_level: risk_level_to_str(decision.risk_level).to_string(),
                                    reason: decision.reason.clone(),
                                    matched_rules: decision.matched_rules.clone(),
                                };
                                if let Err(e) = app_handle.emit("agent-event", &log_event) {
                                    tracing::error!("Failed to emit policy_decision event: {}", e);
                                }

                                let input = tool_input.clone();
                                let mut mgr = sm.lock().await;
                                match mgr.send_permission_response(request_id, true, input).await {
                                    Ok(()) => {
                                        tracing::info!("Auto-approved {} (request {})", tool_name_str, request_id);
                                        continue;
                                    }
                                    Err(e) => {
                                        tracing::error!("Auto-approval response failed: {}", e);
                                        continue;
                                    }
                                }
                            }
                            Verdict::Deny => {
                                // Emit PolicyDecision event for frontend log, then auto-deny
                                let log_event = ZaosEvent::PolicyDecision {
                                    tool_name: Some(tool_name_str.clone()),
                                    verdict: verdict_to_str(decision.verdict).to_string(),
                                    risk_level: risk_level_to_str(decision.risk_level).to_string(),
                                    reason: decision.reason.clone(),
                                    matched_rules: decision.matched_rules.clone(),
                                };
                                if let Err(e) = app_handle.emit("agent-event", &log_event) {
                                    tracing::error!("Failed to emit policy_decision event: {}", e);
                                }

                                let mut mgr = sm.lock().await;
                                match mgr.send_permission_response(request_id, false, None).await {
                                    Ok(()) => {
                                        tracing::info!("Auto-denied {} (request {})", tool_name_str, request_id);
                                        continue;
                                    }
                                    Err(e) => {
                                        tracing::error!("Auto-deny response failed: {}", e);
                                        continue;
                                    }
                                }
                            }
                            Verdict::Ask => {
                                // Emit enriched ApprovalRequested event with policy metadata
                                let enriched = ZaosEvent::ApprovalRequested {
                                    request_id: request_id.clone(),
                                    tool_name: tool_name.clone(),
                                    tool_input: tool_input.clone(),
                                    description: description.clone(),
                                    policy_verdict: Some(verdict_to_str(decision.verdict).to_string()),
                                    policy_risk_level: Some(risk_level_to_str(decision.risk_level).to_string()),
                                    policy_reason: Some(decision.reason.clone()),
                                    policy_matched_rules: Some(decision.matched_rules.clone()),
                                };

                                if let Err(e) = app_handle.emit("agent-event", &enriched) {
                                    tracing::error!("Failed to emit enriched ApprovalRequested: {}", e);
                                }
                                continue;
                            }
                        }
                    }

                    // Emit ZAOS event directly to frontend
                    if let Err(e) = app_handle.emit("agent-event", &event) {
                        tracing::error!("Failed to emit ZAOS event: {}", e);
                    }

                    // Detect successful turn end → attempt to mark gate as ready
                    if let ZaosEvent::RunCompleted { is_error, .. } = &event {
                        let mut engine = we.lock().await;
                        if let Err(e) = engine.try_mark_gate_ready_after_turn(*is_error).await {
                            tracing::warn!("Failed in gate_ready evaluation: {}", e);
                        }
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

    // Fire-and-forget: notify Claude CLI without blocking the IPC response
    drop(workflow);
    let session_manager = state.session_manager.clone();
    let phase_for_msg = next_phase.clone();
    tokio::spawn(async move {
        let mut session = session_manager.lock().await;
        if session.is_session_started() {
            let msg = format!(
                "[GATE VALIDE] L'utilisateur a valide le gate. Phase avancee a : {}. Procede avec cette phase.",
                phase_for_msg
            );
            if let Err(e) = session.send_message(&msg).await {
                tracing::warn!("Failed to notify Claude about gate validation: {}", e);
            }
        }
    });

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

/// Set permission mode (strict or accept-edits)
#[tauri::command]
pub async fn set_permission_mode(
    mode: String,
    state: State<'_, AppState>,
) -> Result<SetModeResponse, String> {
    let valid_modes = ["strict", "accept-edits"];
    if !valid_modes.contains(&mode.as_str()) {
        return Err(format!("Invalid permission mode: {}. Valid: strict, accept-edits", mode));
    }

    // Update runtime cache
    *state.permission_mode.write().await = mode.clone();

    // Persist via engine (uses persist_and_notify for broadcast)
    let mut engine = state.workflow_engine.lock().await;
    engine.set_permission_mode(mode.clone()).await.map_err(|e| e.to_string())?;

    tracing::info!("Permission mode set to: {}", mode);
    Ok(SetModeResponse {
        success: true,
        mode,
    })
}

/// Get current workflow state, enriched with epic data from .memory/current-epic.md
#[tauri::command]
pub async fn get_workflow_state(
    state: State<'_, AppState>,
) -> Result<WorkflowStateDto, String> {
    let workflow = state.workflow_engine.lock().await;
    let wf_state = workflow.get_state();
    let mut dto = WorkflowStateDto::from_state(wf_state);

    // Enrich with current-epic.md data (best-effort, leave None on error)
    let project_dir = state.project_dir().await;
    let reader = MemoryReader::new(project_dir);
    if let Ok(Some(epic)) = reader.read_current_epic().await {
        dto.enrich_from_epic(&epic);
    }

    Ok(dto)
}
/// Manually set gate_ready flag (for debug/recovery)
#[tauri::command]
pub async fn set_gate_ready(
    ready: bool,
    state: State<'_, AppState>,
) -> Result<WorkflowStateDto, String> {
    tracing::info!("set_gate_ready called: ready={}", ready);
    let mut engine = state.workflow_engine.lock().await;
    engine
        .set_gate_ready(ready)
        .await
        .map_err(|e| format!("Failed to set gate_ready: {}", e))?;
    let wf_state = engine.get_state();
    Ok(WorkflowStateDto::from_state(wf_state))
}

/// Check CLI authentication status
#[tauri::command]
pub async fn check_cli_auth(
    state: State<'_, AppState>,
) -> Result<CheckAuthResponse, String> {
    let session = state.session_manager.lock().await;
    match session.check_cli_auth().await {
        Ok(version) => Ok(CheckAuthResponse {
            authenticated: true,
            version: version.trim().to_string(),
            message: format!("{} authenticated", session.runtime_name()),
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

/// Get memory health diagnostic (file presence, parsability, epic name consistency)
#[tauri::command]
pub async fn get_memory_health(
    state: State<'_, AppState>,
) -> Result<MemoryHealthReport, String> {
    tracing::info!("get_memory_health called");

    // Get the epic name from the workflow engine (state.json)
    let workflow_epic = {
        let engine = state.workflow_engine.lock().await;
        engine.get_state().epic.clone()
    };

    let project_dir = state.project_dir().await;
    let reader = MemoryReader::new(project_dir);
    Ok(reader.check_health(&workflow_epic).await)
}

/// Get product contract (all 5 product artifacts aggregated)
#[tauri::command]
pub async fn get_product_contract(
    state: State<'_, AppState>,
) -> Result<crate::memory::ProductContract, String> {
    tracing::info!("get_product_contract called");
    let project_dir = state.project_dir().await;
    let reader = MemoryReader::new(project_dir);
    Ok(reader.read_product_contract().await)
}

/// Get all ZAOS personas from .zaos/personas/
#[tauri::command]
pub async fn get_personas(
    state: State<'_, AppState>,
) -> Result<Vec<Persona>, String> {
    tracing::info!("get_personas called");
    let project_dir = state.project_dir().await;
    let reader = MemoryReader::new(project_dir);
    Ok(reader.read_personas().await)
}

/// Save session insights with auto-metadata from backend state.
/// Frontend provides metrics (duration, tokens, agents) and editorial content.
/// `editorial` is optional — when `None`, the backend defaults to empty arrays
/// (decisions, blockers, learnings) via `SessionInsightsEditorial::default()`.
/// Backend overwrites session_id, phase, epic from its own state.
#[tauri::command]
pub async fn save_session_insights(
    state: State<'_, AppState>,
    duration_secs: u64,
    tokens_input: u64,
    tokens_output: u64,
    agents_used: Vec<String>,
    editorial: Option<SessionInsightsEditorial>,
) -> Result<(), String> {
    tracing::info!("save_session_insights called");
    let project_dir = state.project_dir().await;

    // Auto-metadata from backend state (source of truth)
    let session_id = {
        let mgr = state.session_manager.lock().await;
        mgr.get_session_id().unwrap_or("unknown").to_string()
    };

    let (phase, epic) = {
        let engine = state.workflow_engine.lock().await;
        let ws = engine.get_state();
        (ws.phase.clone(), ws.epic.clone())
    };

    let editorial = editorial.unwrap_or_default();

    crate::session::logger::write_session_insights(
        &project_dir,
        &session_id,
        &phase,
        &epic,
        duration_secs,
        tokens_input,
        tokens_output,
        &agents_used,
        &editorial,
    )
    .await
    .map_err(|e| e.to_string())
}

/// Update only the editorial sections (decisions, blockers, learnings)
/// of an existing `session-insights.md`, preserving original metadata.
#[tauri::command]
pub async fn update_session_editorial(
    state: State<'_, AppState>,
    editorial: SessionInsightsEditorial,
) -> Result<(), String> {
    tracing::info!("update_session_editorial called");
    let project_dir = state.project_dir().await;
    crate::session::logger::update_editorial_only(&project_dir, &editorial).await
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

    // Validate path is within screenshots directory
    {
        let orch = state.screenshot_orchestrator.lock().await;
        let screenshots_dir = orch.screenshots_dir();
        let canonical_file = std::fs::canonicalize(path)
            .map_err(|e| format!("Invalid screenshot path: {}", e))?;
        let canonical_dir = std::fs::canonicalize(screenshots_dir)
            .map_err(|e| format!("Screenshots directory error: {}", e))?;
        if !canonical_file.starts_with(&canonical_dir) {
            return Err("Screenshot path must be within the screenshots directory".to_string());
        }
    }

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

    let runtime_paths = state.runtime_paths().await;

    let agents_dir = &runtime_paths.agents_dir;
    let agent_count = std::fs::read_dir(agents_dir)
        .map(|entries| entries.filter_map(|e| e.ok()).count())
        .unwrap_or(0);

    let rules_dir = &runtime_paths.rules_dir;
    let rule_count = std::fs::read_dir(rules_dir)
        .map(|entries| entries.filter_map(|e| e.ok()).count())
        .unwrap_or(0);

    let settings_exists = runtime_paths.settings_file.exists();

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

/// List all agent files in the runtime agents directory with name and first-line description
#[tauri::command]
pub async fn list_agents(
    state: State<'_, AppState>,
) -> Result<Vec<AgentInfo>, String> {
    let agents_dir = state.runtime_paths().await.agents_dir;
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
    validate_safe_name(&name)?;
    let path = state.runtime_paths().await.agents_dir.join(format!("{}.md", name));
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
    validate_safe_name(&name)?;
    let runtime_paths = state.runtime_paths().await;
    let agents_dir = &runtime_paths.agents_dir;
    std::fs::create_dir_all(agents_dir).map_err(|e| e.to_string())?;
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
    validate_safe_name(&name)?;
    let path = state.runtime_paths().await.agents_dir.join(format!("{}.md", name));
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

/// Get runtime info (kind, display name, paths)
#[tauri::command]
pub async fn get_runtime_info(
    state: State<'_, AppState>,
) -> Result<RuntimeInfo, String> {
    tracing::info!("get_runtime_info called");
    let rk = *state.runtime_kind.read().await;
    Ok(RuntimeInfo {
        kind: rk,
        name: rk.display_name().to_string(),
        paths: state.runtime_paths().await,
    })
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

    // Validate path is under user's home directory to prevent scope escape
    if let Some(home) = dirs::home_dir() {
        let canonical = new_dir.canonicalize()
            .map_err(|e| format!("Invalid path: {}", e))?;
        let canonical_home = home.canonicalize().unwrap_or_else(|e| {
            tracing::warn!("Could not canonicalize home directory: {}", e);
            home
        });
        if !canonical.starts_with(&canonical_home) {
            return Err("Projects must be under user home directory".to_string());
        }
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
    let rk = *state.runtime_kind.read().await;
    *state.session_manager.lock().await = SessionManager::new(new_dir.clone(), rk);

    // 6. Replace workflow_engine
    *state.workflow_engine.lock().await = WorkflowEngine::new(new_dir.clone());

    // 6b. Load persisted workflow state from new project
    let loaded_permission_mode = {
        let mut engine = state.workflow_engine.lock().await;
        match engine.load_state().await {
            Ok(wf_state) => {
                tracing::info!(
                    "Workflow state loaded for new project: phase={}, mode={:?}, permission={}",
                    wf_state.phase, wf_state.mode, wf_state.permission_mode
                );
                wf_state.permission_mode.clone()
            }
            Err(e) => {
                tracing::warn!("Could not load workflow state for new project (using defaults): {}", e);
                "strict".to_string()
            }
        }
    };
    // Sync permission_mode cache (outside engine lock)
    *state.permission_mode.write().await = loaded_permission_mode;

    // 7. Replace screenshot_orchestrator
    *state.screenshot_orchestrator.lock().await = ScreenshotOrchestrator::new(
        new_dir.join(".screenshots"),
        Box::new(FilesystemAdapter),
    );

    // 8. (permission_mode is now synced in step 6b from loaded state)

    // 9. Update project_dir (after all services are replaced)
    *state.project_dir.write().await = new_dir.clone();

    // 10. Update runtime_paths
    let new_runtime_paths = RuntimePaths::for_kind(&new_dir, &rk);
    *state.runtime_paths.write().await = new_runtime_paths.clone();

    // 11. Restart watchers (use the same new_runtime_paths)
    let mut watcher = state.watcher_service.lock().await;
    *watcher = FileWatcherService::new(new_dir.clone(), new_runtime_paths);
    watcher
        .start(app.clone(), state.screenshot_orchestrator.clone(), state.workflow_engine.clone(), state.permission_mode.clone())
        .map_err(|e| format!("Failed to start watchers: {}", e))?;

    // 11b. Restart MCP server for new project
    {
        // Stop old MCP server and cleanup its discovery files
        let mut mcp_guard = state.mcp_handle.lock().await;
        if let Some(old_handle) = mcp_guard.take() {
            old_handle.cancel_token.cancel();
            old_handle.cleanup();
            tracing::info!("Old MCP server stopped and cleaned up");
        }

        // Start new MCP server for the new project
        match crate::mcp_server::start_mcp_server(
            state.workflow_engine.clone(),
            state.screenshot_orchestrator.clone(),
            app.clone(),
            new_dir.clone(),
        ).await {
            Ok(new_handle) => {
                tracing::info!(port = new_handle.port, "MCP server restarted for new project");
                *mcp_guard = Some(new_handle);
            }
            Err(e) => {
                tracing::error!("Failed to restart MCP server: {}", e);
                // Non-fatal: project switch continues without MCP
            }
        }
    }

    // 12. Build ProjectInfo
    let info = ProjectInfo::from_path(&new_dir);

    // 13. Emit project-changed event
    app.emit("project-changed", &info)
        .map_err(|e| format!("Failed to emit project-changed: {}", e))?;

    tracing::info!("Project switched to: {}", info.path);
    Ok(info)
}

// =============================================================================
// Workflow Init Commands
// =============================================================================

/// Set the active policy profile (observe, guided-build, autopilot-safe, release-guarded)
#[tauri::command]
pub async fn set_policy_profile(
    profile: String,
    state: State<'_, AppState>,
) -> Result<SetPolicyProfileResponse, String> {
    use crate::policy::PolicyProfile;
    PolicyProfile::from_str(&profile)
        .ok_or_else(|| format!("Invalid policy profile: {}. Valid: observe, guided-build, autopilot-safe, release-guarded", profile))?;

    let mut engine = state.workflow_engine.lock().await;
    engine.set_policy_profile(profile.clone()).await.map_err(|e| e.to_string())?;

    tracing::info!("Policy profile set to: {}", profile);
    Ok(SetPolicyProfileResponse {
        success: true,
        profile,
    })
}

/// Preview/debug: evaluate a policy decision for a given action without enforcing it
#[tauri::command]
pub async fn get_policy_evaluation(
    action_type: String,
    file_paths: Vec<String>,
    tool_name: Option<String>,
    is_destructive: Option<bool>,
    state: State<'_, AppState>,
) -> Result<crate::policy::PolicyDecision, String> {
    use crate::policy::{ActionType, ActionContext, PolicyProfile, evaluate};
    use crate::workflow::product_phase::derive_product_phase;

    let engine = state.workflow_engine.lock().await;
    let wf_state = engine.get_state();

    // Parse action_type
    let action = match action_type.as_str() {
        "file_write" => ActionType::FileWrite,
        "file_delete" => ActionType::FileDelete,
        "bash_command" => ActionType::BashCommand,
        "web_fetch" => ActionType::WebFetch,
        "tool_call" => ActionType::ToolCall,
        _ => return Err(format!("Invalid action_type: {}. Valid: file_write, file_delete, bash_command, web_fetch, tool_call", action_type)),
    };

    // Parse policy profile from state
    let profile = match PolicyProfile::from_str(&wf_state.policy_profile) {
        Some(p) => p,
        None => {
            tracing::warn!("Invalid policy_profile in state.json: '{}', falling back to guided-build", wf_state.policy_profile);
            PolicyProfile::default()
        }
    };

    // Derive product phase from current state
    let product_phase = derive_product_phase(wf_state);

    let ctx = ActionContext {
        action_type: action,
        file_paths,
        tool_name,
        command_text: None, // debug endpoint: no raw command text
        product_phase,
        is_destructive: is_destructive.unwrap_or(false),
        is_reversible: true,
    };

    Ok(evaluate(&profile, &ctx))
}

/// Start a new epic: set name, transition to comprehension phase, write current-epic.md
#[tauri::command]
pub async fn start_epic(
    name: String,
    description: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    tracing::info!("start_epic called: name={}, description={}", name, description);

    let project_dir = state.project_dir().await;

    let mut engine = state.workflow_engine.lock().await;
    engine.start_epic(name.clone()).await.map_err(|e| e.to_string())?;

    let epic_content = format!(
        "# Epic active : {}\n\n> Milestone : (a definir)\n> Statut : EN COURS\n\n## Objectif\n\n{}\n\n## Tasks\n\n| # | Task | Fichier(s) | Statut | Notes |\n|---|------|-----------|--------|-------|\n\n_En attente du plan._\n",
        name,
        if description.is_empty() { "_Pas de description._" } else { &description }
    );
    let epic_path = project_dir.join(".memory").join("current-epic.md");
    tokio::fs::write(&epic_path, epic_content)
        .await
        .map_err(|e| format!("Failed to write current-epic.md: {}", e))?;

    tracing::info!("Epic started: {}", name);
    Ok(())
}