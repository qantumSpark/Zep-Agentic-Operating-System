//! MCP Server — zaos-ide
//!
//! Embeds a Streamable HTTP MCP server inside the Tauri process using `rmcp`.
//! Claude Code CLI connects via `type: "http"` in `.mcp.json`.
//! Provides 4 tools: get_ui_state, notify, capture_screenshot, show_diff.

use std::path::PathBuf;
use std::sync::Arc;

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler,
};
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::screenshots::ScreenshotOrchestrator;
use crate::workflow::WorkflowEngine;

// =============================================================================
// Tool parameter / response types
// =============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct NotifyParams {
    /// Notification title
    pub title: String,
    /// Notification body message
    pub message: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CaptureScreenshotParams {
    /// Optional URL to capture (for web projects)
    pub url: Option<String>,
    /// Optional iteration id to attach the screenshot to
    pub iteration_id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ShowDiffParams {
    /// File path the diff applies to
    pub path: String,
    /// Content before the change
    pub before: String,
    /// Content after the change
    pub after: String,
    /// Optional description of the change
    pub description: Option<String>,
}

// =============================================================================
// MCP Server struct
// =============================================================================

/// Shared state accessible from all MCP tool handlers.
/// Cloned once per MCP session via the factory closure.
#[derive(Clone)]
pub struct ZaosIdeServer {
    tool_router: ToolRouter<ZaosIdeServer>,
    workflow_engine: Arc<Mutex<WorkflowEngine>>,
    screenshot_orchestrator: Arc<Mutex<ScreenshotOrchestrator>>,
    app_handle: tauri::AppHandle,
    project_dir: PathBuf,
}

#[tool_router]
impl ZaosIdeServer {
    pub fn new(
        workflow_engine: Arc<Mutex<WorkflowEngine>>,
        screenshot_orchestrator: Arc<Mutex<ScreenshotOrchestrator>>,
        app_handle: tauri::AppHandle,
        project_dir: PathBuf,
    ) -> Self {
        Self {
            tool_router: Self::tool_router(),
            workflow_engine,
            screenshot_orchestrator,
            app_handle,
            project_dir,
        }
    }

    // ---- Tool: get_ui_state ----

    #[tool(description = "Get the current ZAOS UI state: workflow phase, epic, task, mode, and project directory. Use this to understand what the user is working on.")]
    async fn get_ui_state(&self) -> Result<CallToolResult, McpError> {
        let wf = self.workflow_engine.lock().await;
        let state = wf.get_state();

        #[derive(Serialize)]
        struct UiState {
            phase: String,
            epic: String,
            task: String,
            mode: String,
            gate_validated: bool,
            project_dir: String,
        }

        let ui_state = UiState {
            phase: state.phase.clone(),
            epic: state.epic.clone(),
            task: state.task.clone(),
            mode: format!("{:?}", state.mode).to_lowercase(),
            gate_validated: state.gate_validated,
            project_dir: self.project_dir.to_string_lossy().to_string(),
        };

        let json = serde_json::to_string_pretty(&ui_state)
            .unwrap_or_else(|_| "{}".to_string());

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    // ---- Tool: notify ----

    #[tool(description = "Send a notification to the ZAOS user. Use for important status updates, task completion, or alerts that need user attention.")]
    async fn notify(
        &self,
        Parameters(params): Parameters<NotifyParams>,
    ) -> Result<CallToolResult, McpError> {
        #[derive(Serialize, Clone)]
        struct NotifyPayload {
            title: String,
            message: String,
        }

        let title_for_log = params.title.clone();
        let payload = NotifyPayload {
            title: params.title,
            message: params.message,
        };

        self.app_handle
            .emit("mcp-notify", &payload)
            .map_err(|e| McpError::internal_error(format!("Failed to emit notify event: {}", e), None))?;

        tracing::info!(title = %title_for_log, "MCP notify sent");
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Notification sent: {}",
            title_for_log
        ))]))
    }

    // ---- Tool: capture_screenshot ----

    #[tool(description = "Trigger a screenshot capture in ZAOS. For web projects, provide a URL. The screenshot will appear in the ZAOS gallery. Returns the capture request status.")]
    async fn capture_screenshot(
        &self,
        Parameters(params): Parameters<CaptureScreenshotParams>,
    ) -> Result<CallToolResult, McpError> {
        let orch = self.screenshot_orchestrator.lock().await;
        let result = orch
            .request_capture(params.url.clone(), None, params.iteration_id.clone())
            .await
            .map_err(|e| McpError::internal_error(format!("Capture failed: {}", e), None))?;

        let msg = match result {
            crate::screenshots::types::CaptureRequest::FilesystemPending { expected_path } => {
                format!("Capture pending — file expected at: {}", expected_path.display())
            }
            crate::screenshots::types::CaptureRequest::CliPromptSent { .. } => {
                "Capture prompt sent to CLI — screenshot will appear when ready".to_string()
            }
        };

        tracing::info!("MCP capture_screenshot: {}", msg);
        Ok(CallToolResult::success(vec![Content::text(msg)]))
    }

    // ---- Tool: show_diff ----

    #[tool(description = "Display a visual diff in the ZAOS dashboard. Provide the file path, before content, and after content. The diff will be rendered in the UI for the user to review.")]
    async fn show_diff(
        &self,
        Parameters(params): Parameters<ShowDiffParams>,
    ) -> Result<CallToolResult, McpError> {
        #[derive(Serialize, Clone)]
        struct DiffPayload {
            path: String,
            before: String,
            after: String,
            description: Option<String>,
        }

        let payload = DiffPayload {
            path: params.path.clone(),
            before: params.before,
            after: params.after,
            description: params.description,
        };

        self.app_handle
            .emit("mcp-show-diff", &payload)
            .map_err(|e| McpError::internal_error(format!("Failed to emit diff event: {}", e), None))?;

        tracing::info!(path = %params.path, "MCP show_diff emitted");
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Diff displayed for: {}",
            params.path
        ))]))
    }
}

#[tool_handler]
impl ServerHandler for ZaosIdeServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder().enable_tools().build(),
        )
        .with_server_info(Implementation::from_build_env())
        .with_instructions(
            "ZAOS IDE tools: get workflow state, send notifications, capture screenshots, show diffs."
                .to_string(),
        )
    }
}

// =============================================================================
// Server startup
// =============================================================================

/// Information about the running MCP server, used for cleanup on shutdown.
pub struct McpServerHandle {
    pub port: u16,
    pub token: String,
    pub cancel_token: CancellationToken,
    pub project_dir: PathBuf,
}

impl McpServerHandle {
    /// Clean up files written at startup (.zaos/mcp-port.json, .mcp.json entry).
    pub fn cleanup(&self) {
        let port_file = self.project_dir.join(".zaos").join("mcp-port.json");
        if let Err(e) = std::fs::remove_file(&port_file) {
            if e.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!("Failed to remove mcp-port.json: {}", e);
            }
        }

        // Remove zaos-ide entry from .mcp.json (or delete if it only had our entry)
        let mcp_json_path = self.project_dir.join(".mcp.json");
        if let Ok(contents) = std::fs::read_to_string(&mcp_json_path) {
            if let Ok(mut obj) = serde_json::from_str::<serde_json::Value>(&contents) {
                if let Some(servers) = obj.get_mut("mcpServers").and_then(|s| s.as_object_mut()) {
                    servers.remove("zaos-ide");
                    if servers.is_empty() {
                        let _ = std::fs::remove_file(&mcp_json_path);
                    } else {
                        let updated = serde_json::to_string_pretty(&obj).unwrap_or_default();
                        let _ = std::fs::write(&mcp_json_path, updated);
                    }
                }
            }
        }

        tracing::info!("MCP server cleanup complete");
    }
}

/// Start the MCP Streamable HTTP server on a random port.
///
/// Writes `.zaos/mcp-port.json` and updates `.mcp.json` for Claude Code discovery.
/// Returns a handle for shutdown cleanup.
pub async fn start_mcp_server(
    workflow_engine: Arc<Mutex<WorkflowEngine>>,
    screenshot_orchestrator: Arc<Mutex<ScreenshotOrchestrator>>,
    app_handle: tauri::AppHandle,
    project_dir: PathBuf,
) -> Result<McpServerHandle, Box<dyn std::error::Error + Send + Sync>> {
    use rmcp::transport::streamable_http_server::{
        session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
    };

    let ct = CancellationToken::new();

    // Auth token for this session
    let token = uuid::Uuid::new_v4().to_string();
    let token_for_middleware = token.clone();

    // Factory: each MCP session gets a clone of the server (Arcs are cheap to clone)
    let wf = workflow_engine.clone();
    let orch = screenshot_orchestrator.clone();
    let handle = app_handle.clone();
    let dir = project_dir.clone();

    let service = StreamableHttpService::new(
        move || Ok(ZaosIdeServer::new(wf.clone(), orch.clone(), handle.clone(), dir.clone())),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default().with_cancellation_token(ct.child_token()),
    );

    // Auth middleware: reject requests without valid Bearer token
    let expected_header = format!("Bearer {}", token_for_middleware);
    let auth_layer = axum::middleware::from_fn(move |req: axum::extract::Request, next: axum::middleware::Next| {
        let expected = expected_header.clone();
        async move {
            let auth_header = req
                .headers()
                .get("authorization")
                .and_then(|v| v.to_str().ok());

            match auth_header {
                Some(h) if h == expected => {
                    next.run(req).await
                }
                _ => {
                    axum::http::Response::builder()
                        .status(axum::http::StatusCode::UNAUTHORIZED)
                        .body(axum::body::Body::from("Unauthorized"))
                        .expect("static 401 response")
                }
            }
        }
    });

    let router = axum::Router::new()
        .nest_service("/mcp", service)
        .layer(auth_layer);

    // Bind to random port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();

    tracing::info!(port = port, "MCP server starting on 127.0.0.1:{}", port);

    // Write discovery files
    write_port_file(&project_dir, port, &token)?;
    write_mcp_json(&project_dir, port, &token)?;

    // Spawn the server
    let ct_clone = ct.clone();
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                ct_clone.cancelled().await;
            })
            .await
        {
            tracing::error!("MCP server error: {}", e);
        }
        tracing::info!("MCP server stopped");
    });

    Ok(McpServerHandle {
        port,
        token,
        cancel_token: ct,
        project_dir,
    })
}

// =============================================================================
// Discovery file writers
// =============================================================================

/// Write `.zaos/mcp-port.json` with port and token for debugging/tooling.
fn write_port_file(
    project_dir: &PathBuf,
    port: u16,
    token: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let zaos_dir = project_dir.join(".zaos");
    std::fs::create_dir_all(&zaos_dir)?;

    #[derive(Serialize)]
    struct PortFile {
        port: u16,
        token: String,
        pid: u32,
        started_at: String,
    }

    let content = serde_json::to_string_pretty(&PortFile {
        port,
        token: token.to_string(),
        pid: std::process::id(),
        started_at: chrono::Utc::now().to_rfc3339(),
    })?;

    std::fs::write(zaos_dir.join("mcp-port.json"), content)?;
    tracing::info!("Wrote .zaos/mcp-port.json");
    Ok(())
}

/// Write or update `.mcp.json` with the zaos-ide server entry.
fn write_mcp_json(
    project_dir: &PathBuf,
    port: u16,
    token: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mcp_path = project_dir.join(".mcp.json");

    // Read existing .mcp.json or start fresh
    let mut root = if let Ok(contents) = std::fs::read_to_string(&mcp_path) {
        serde_json::from_str::<serde_json::Value>(&contents).unwrap_or_else(|_| {
            serde_json::json!({ "mcpServers": {} })
        })
    } else {
        serde_json::json!({ "mcpServers": {} })
    };

    // Upsert zaos-ide entry
    let root_obj = root.as_object_mut().ok_or("Invalid .mcp.json: root is not an object")?;
    let servers = root_obj
        .entry("mcpServers")
        .or_insert_with(|| serde_json::json!({}));

    let servers_obj = servers.as_object_mut().ok_or("Invalid .mcp.json: mcpServers is not an object")?;
    servers_obj.insert(
        "zaos-ide".to_string(),
        serde_json::json!({
            "type": "http",
            "url": format!("http://127.0.0.1:{}/mcp", port),
            "headers": {
                "Authorization": format!("Bearer {}", token)
            }
        }),
    );

    std::fs::write(&mcp_path, serde_json::to_string_pretty(&root)?)?;
    tracing::info!("Wrote .mcp.json with zaos-ide entry (port {})", port);
    Ok(())
}
