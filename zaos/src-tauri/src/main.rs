// Tauri entry point for ZAOS
// Registers all commands and sets up the event system

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod events;
mod memory;
mod mcp_server;
mod screenshots;
mod session;
mod workflow;

use commands::AppState;
use std::path::PathBuf;
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

    // Get project directory from environment or use current directory
    let project_dir = std::env::var("ZAOS_PROJECT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    tracing::info!("Project directory: {:?}", project_dir);

    // Create app state
    let app_state = AppState::new(project_dir);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::send_prompt,
            commands::interrupt_session,
            commands::validate_gate,
            commands::set_mode,
            commands::get_workflow_state,
            commands::check_cli_auth,
            commands::list_sessions,
        ])
        .setup(|app| {
            tracing::info!("App setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
