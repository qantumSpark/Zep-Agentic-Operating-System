// Tauri entry point for ZAOS
// Registers all commands and sets up the event system

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod events;
mod init;
mod memory;
mod mcp_server;
mod screenshots;
mod session;
mod watchers;
mod workflow;

use commands::AppState;
use std::path::PathBuf;
use tauri::Manager;
use tracing_subscriber::EnvFilter;

fn main() {
    // Initialize tracing with default level info
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tracing::info!("ZAOS startup");

    // Get project directory from environment, or derive from current dir.
    // When run via `tauri dev`, cwd is src-tauri/. Walk up to find the repo root.
    let project_dir = std::env::var("ZAOS_PROJECT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            // If cwd ends with src-tauri, go up one level (src-tauri → zaos)
            if cwd.ends_with("src-tauri") {
                cwd.parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or(cwd)
            } else {
                cwd
            }
        });

    tracing::info!("Project directory: {:?}", project_dir);

    // Ensure .workflow/ and .memory/ directories exist with default files
    init::ensure_project_dirs(&project_dir);

    // Create app state
    let app_state = AppState::new(project_dir);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::send_prompt,
            commands::interrupt_session,
            commands::respond_permission,
            commands::validate_gate,
            commands::set_mode,
            commands::get_workflow_state,
            commands::get_memory_state,
            commands::check_cli_auth,
            commands::list_sessions,
            commands::get_screenshots,
            commands::add_screenshot,
            commands::delete_screenshot,
            commands::request_capture,
        ])
        .setup(|app| {
            let state = app.state::<AppState>();
            let watcher = state.watcher_service.clone();
            let orch = state.screenshot_orchestrator.clone();
            let handle = app.handle().clone();

            match watcher.start(handle, orch) {
                Ok(()) => tracing::info!("FileWatcherService started"),
                Err(e) => tracing::warn!("FileWatcherService failed to start: {}", e),
            }

            tracing::info!("App setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
