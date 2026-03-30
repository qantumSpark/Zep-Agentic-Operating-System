// Project directory initialization
// Ensures .workflow/ and .memory/ directories exist with default files at startup.

use crate::workflow::state::WorkflowState;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

/// Ensure project directories and default files exist.
/// Called before AppState creation so watchers and engines have valid paths.
/// Uses std::fs (not tokio) since this runs before the async runtime.
pub fn ensure_project_dirs(project_dir: &Path) {
    let workflow_dir = project_dir.join(".workflow");
    let memory_dir = project_dir.join(".memory");

    // --- .workflow/ ---
    ensure_dir(&workflow_dir);
    ensure_file(
        &workflow_dir.join("state.json"),
        || {
            let state = WorkflowState::default();
            serde_json::to_string_pretty(&state).unwrap_or_else(|e| {
                tracing::warn!("Failed to serialize default WorkflowState: {}", e);
                "{}".to_string()
            })
        },
    );

    // --- .memory/ ---
    ensure_dir(&memory_dir);
    ensure_file(
        &memory_dir.join("INDEX.md"),
        || {
            concat!(
                "# Memory Index\n",
                "\n",
                "Central index of project memory files.\n",
                "\n",
                "## Entries\n",
                "\n",
                "- [State](state.md) — Current session state\n",
                "- [Current Epic](current-epic.md) — Active epic details\n",
            )
            .to_string()
        },
    );
    ensure_file(
        &memory_dir.join("state.md"),
        || {
            concat!(
                "# State\n",
                "\n",
                "## Current Focus\n",
                "\n",
                "_No active task._\n",
                "\n",
                "## Recent Changes\n",
                "\n",
                "_None yet._\n",
            )
            .to_string()
        },
    );
    ensure_file(
        &memory_dir.join("current-epic.md"),
        || {
            concat!(
                "# Current Epic\n",
                "\n",
                "## Epic\n",
                "\n",
                "_No epic selected._\n",
                "\n",
                "## Task Plan\n",
                "\n",
                "_No tasks defined._\n",
                "\n",
                "## Progress\n",
                "\n",
                "_Not started._\n",
            )
            .to_string()
        },
    );

    tracing::info!("Project directories initialized");
}

/// Create a directory (no-op if it already exists).
fn ensure_dir(path: &Path) {
    if let Err(e) = fs::create_dir_all(path) {
        tracing::warn!("Failed to create directory {:?}: {}", path, e);
    }
}

/// Atomically create a file with default content if it doesn't already exist.
/// Uses `create_new(true)` to avoid TOCTOU races.
/// The content is produced lazily via a closure so we only build it when needed.
fn ensure_file<F: FnOnce() -> String>(path: &Path, content_fn: F) {
    match OpenOptions::new().create_new(true).write(true).open(path) {
        Ok(mut file) => {
            let content = content_fn();
            if let Err(e) = file.write_all(content.as_bytes()) {
                tracing::warn!("Failed to write default content to {:?}: {}", path, e);
            } else {
                tracing::info!("Created default file {:?}", path);
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            // File already exists — nothing to do
        }
        Err(e) => {
            tracing::warn!("Failed to create file {:?}: {}", path, e);
        }
    }
}
