use crate::commands::MemoryStateResponse;
use crate::memory::MemoryReader;
use crate::workflow::state::WorkflowState;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;
use tauri::Emitter;
use tokio::sync::mpsc;

/// Category of file change detected by the watcher.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WatchCategory {
    /// Changes to `.workflow/state.json`
    Workflow,
    /// Changes inside the `.memory/` directory
    Memory,
}

/// Watches `.workflow/state.json` and `.memory/` for changes,
/// bridging notify's synchronous callbacks into the tokio async runtime.
pub struct FileWatcherService {
    project_dir: PathBuf,
}

impl FileWatcherService {
    pub fn new(project_dir: PathBuf) -> Self {
        Self { project_dir }
    }

    /// Start watching for file changes.
    ///
    /// Spawns a background tokio task that receives notify events via an mpsc channel,
    /// applies a 300ms per-category debounce, and emits Tauri events on detected changes.
    /// Workflow changes emit `workflow-change` with the parsed WorkflowState.
    pub fn start(
        &self,
        app_handle: tauri::AppHandle,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let workflow_dir = self.project_dir.join(".workflow");
        let memory_dir = self.project_dir.join(".memory");

        // Channel to bridge sync notify callbacks → async tokio task
        let (tx, mut rx) = mpsc::channel::<Event>(256);

        // Create the watcher with a channel-based event handler
        let mut watcher = RecommendedWatcher::new(
            move |result: Result<Event, notify::Error>| {
                match result {
                    Ok(event) => {
                        // Non-blocking send; if the channel is full we drop the event
                        // (the debounce will coalesce anyway)
                        let _ = tx.try_send(event);
                    }
                    Err(e) => {
                        tracing::error!("File watcher error: {:?}", e);
                    }
                }
            },
            Config::default(),
        )?;

        // Watch .workflow/ directory (filter for state.json by filename in the event loop)
        if workflow_dir.exists() {
            watcher.watch(&workflow_dir, RecursiveMode::NonRecursive)?;
            tracing::info!("Watching workflow directory: {:?}", workflow_dir);
        } else {
            tracing::warn!(
                "Workflow directory does not exist yet, skipping watch: {:?}",
                workflow_dir
            );
        }

        // Watch .memory/ directory (recursive)
        if memory_dir.exists() {
            watcher.watch(&memory_dir, RecursiveMode::Recursive)?;
            tracing::info!("Watching memory directory: {:?}", memory_dir);
        } else {
            tracing::warn!(
                "Memory directory does not exist yet, skipping watch: {:?}",
                memory_dir
            );
        }

        // Clone paths and handles for the async task
        let workflow_dir_owned = workflow_dir;
        let memory_dir_owned = memory_dir;
        let project_dir = self.project_dir.clone();
        let app_handle = app_handle.clone();
        let state_file_path = workflow_dir_owned.join("state.json");

        // Spawn async consumer — owns the watcher so it stays alive
        // Use tauri::async_runtime::spawn (not tokio::spawn) because setup() may
        // run before the tokio runtime handle is available on the current thread.
        tauri::async_runtime::spawn(async move {
            // Keep watcher alive for the lifetime of this task
            let _watcher = watcher;

            // Per-category debounce tracking
            let mut last_event: HashMap<WatchCategory, Instant> = HashMap::new();
            let debounce = std::time::Duration::from_millis(300);

            while let Some(event) = rx.recv().await {
                for path in &event.paths {
                    let category = if path.starts_with(&workflow_dir_owned)
                        && path.file_name().and_then(|f| f.to_str()) == Some("state.json")
                    {
                        Some(WatchCategory::Workflow)
                    } else if path.starts_with(&memory_dir_owned) {
                        Some(WatchCategory::Memory)
                    } else {
                        None
                    };

                    if let Some(cat) = category {
                        let now = Instant::now();
                        let should_emit = match last_event.get(&cat) {
                            Some(last) => now.duration_since(*last) >= debounce,
                            None => true,
                        };

                        if should_emit {
                            last_event.insert(cat.clone(), now);

                            match cat {
                                WatchCategory::Workflow => {
                                    match tokio::fs::read_to_string(&state_file_path).await {
                                        Ok(content) => {
                                            match serde_json::from_str::<WorkflowState>(&content) {
                                                Ok(state) => {
                                                    tracing::info!(
                                                        "Workflow state changed: phase={}, epic={}",
                                                        state.phase,
                                                        state.epic
                                                    );
                                                    if let Err(e) =
                                                        app_handle.emit("workflow-change", &state)
                                                    {
                                                        tracing::warn!(
                                                            "Failed to emit workflow-change: {}",
                                                            e
                                                        );
                                                    }
                                                }
                                                Err(e) => {
                                                    tracing::warn!(
                                                        "Failed to parse workflow state: {}",
                                                        e
                                                    );
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            tracing::warn!(
                                                "Failed to read workflow state file: {}",
                                                e
                                            );
                                        }
                                    }
                                }
                                WatchCategory::Memory => {
                                    tracing::info!("Memory file changed: {:?}", path);

                                    let reader = MemoryReader::new(project_dir.clone());

                                    let index = match reader.read_index().await {
                                        Ok(idx) => idx,
                                        Err(e) => {
                                            tracing::warn!(
                                                "Failed to read memory index: {}",
                                                e
                                            );
                                            continue;
                                        }
                                    };

                                    let memory_state = match reader.read_state().await {
                                        Ok(s) => Some(s),
                                        Err(e) => {
                                            tracing::debug!(
                                                "Memory state not available: {}",
                                                e
                                            );
                                            None
                                        }
                                    };

                                    let current_epic =
                                        match reader.read_current_epic().await {
                                            Ok(epic) => epic,
                                            Err(e) => {
                                                tracing::warn!(
                                                    "Failed to read current epic: {}",
                                                    e
                                                );
                                                None
                                            }
                                        };

                                    let response = MemoryStateResponse {
                                        index,
                                        state: memory_state,
                                        current_epic,
                                    };

                                    if let Err(e) =
                                        app_handle.emit("memory-change", &response)
                                    {
                                        tracing::warn!(
                                            "Failed to emit memory-change: {}",
                                            e
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }

            tracing::info!("File watcher task ended");
        });

        Ok(())
    }
}
