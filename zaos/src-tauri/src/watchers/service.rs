use crate::memory::MemoryReader;
use crate::screenshots::ScreenshotOrchestrator;
use crate::workflow::state::WorkflowState;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tauri::Emitter;
use tokio::sync::{mpsc, Mutex};

/// Category of file change detected by the watcher.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WatchCategory {
    /// Changes to `.workflow/state.json`
    Workflow,
    /// Changes inside the `.memory/` directory
    Memory,
    /// New image files in `.screenshots/` directory
    Screenshot,
    /// Changes inside the `.claude/` directory
    Claude,
}

/// Watches `.workflow/state.json` and `.memory/` for changes,
/// bridging notify's synchronous callbacks into the tokio async runtime.
pub struct FileWatcherService {
    project_dir: PathBuf,
    task_handle: Option<tauri::async_runtime::JoinHandle<()>>,
}

impl FileWatcherService {
    pub fn new(project_dir: PathBuf) -> Self {
        Self {
            project_dir,
            task_handle: None,
        }
    }

    /// Stop the watcher task. Safe to call even if not started.
    pub fn stop(&mut self) {
        if let Some(handle) = self.task_handle.take() {
            handle.abort();
            tracing::info!("FileWatcherService stopped");
        }
    }

    /// Start watching for file changes.
    ///
    /// Spawns a background tokio task that receives notify events via an mpsc channel,
    /// applies a 300ms per-category debounce, and emits Tauri events on detected changes.
    /// Workflow changes emit `workflow-change` with the parsed WorkflowState.
    pub fn start(
        &mut self,
        app_handle: tauri::AppHandle,
        screenshot_orchestrator: Arc<Mutex<ScreenshotOrchestrator>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let workflow_dir = self.project_dir.join(".workflow");
        let memory_dir = self.project_dir.join(".memory");
        let screenshots_dir = self.project_dir.join(".screenshots");

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

        // Watch .screenshots/ directory (non-recursive, images only)
        if screenshots_dir.exists() {
            watcher.watch(&screenshots_dir, RecursiveMode::NonRecursive)?;
            tracing::info!("Watching screenshots directory: {:?}", screenshots_dir);
        } else {
            tracing::warn!(
                "Screenshots directory does not exist yet: {:?}",
                screenshots_dir
            );
        }

        // Watch .claude/ directory (recursive — agents, settings, etc.)
        let claude_dir = self.project_dir.join(".claude");
        if claude_dir.exists() {
            watcher.watch(&claude_dir, RecursiveMode::Recursive)?;
            tracing::info!("Watching .claude directory: {:?}", claude_dir);
        } else {
            tracing::warn!(
                ".claude directory does not exist yet, skipping watch: {:?}",
                claude_dir
            );
        }

        // Clone paths and handles for the async task
        let workflow_dir_owned = workflow_dir;
        let memory_dir_owned = memory_dir;
        let screenshots_dir_owned = screenshots_dir;
        let claude_dir_owned = claude_dir;
        let project_dir = self.project_dir.clone();
        let app_handle = app_handle.clone();
        let state_file_path = workflow_dir_owned.join("state.json");

        // Spawn async consumer — owns the watcher so it stays alive
        // Use tauri::async_runtime::spawn (not tokio::spawn) because setup() may
        // run before the tokio runtime handle is available on the current thread.
        let handle = tauri::async_runtime::spawn(async move {
            // Keep watcher alive for the lifetime of this task
            let _watcher = watcher;

            // Per-category debounce tracking
            let mut last_event: HashMap<WatchCategory, Instant> = HashMap::new();
            let debounce = std::time::Duration::from_millis(300);

            // Create the reader once, outside the event loop
            let memory_reader = MemoryReader::new(project_dir.clone());

            while let Some(event) = rx.recv().await {
                for path in &event.paths {
                    // Check if it's an image file by extension
                    let is_image = path
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| {
                            matches!(
                                ext.to_lowercase().as_str(),
                                "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp"
                            )
                        })
                        .unwrap_or(false);

                    let category = if path.starts_with(&workflow_dir_owned)
                        && path.file_name().and_then(|f| f.to_str()) == Some("state.json")
                    {
                        Some(WatchCategory::Workflow)
                    } else if path.starts_with(&memory_dir_owned) {
                        Some(WatchCategory::Memory)
                    } else if path.starts_with(&screenshots_dir_owned) && is_image {
                        Some(WatchCategory::Screenshot)
                    } else if path.starts_with(&claude_dir_owned) {
                        Some(WatchCategory::Claude)
                    } else {
                        None
                    };

                    if let Some(cat) = category {
                        let now = Instant::now();
                        let debounce_duration = match &cat {
                            WatchCategory::Screenshot => {
                                std::time::Duration::from_millis(500)
                            }
                            _ => debounce,
                        };
                        let should_emit = match last_event.get(&cat) {
                            Some(last) => now.duration_since(*last) >= debounce_duration,
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

                                    match memory_reader.read_all().await {
                                        Ok(response) => {
                                            if let Err(e) =
                                                app_handle.emit("memory-change", &response)
                                            {
                                                tracing::warn!(
                                                    "Failed to emit memory-change: {}",
                                                    e
                                                );
                                            }
                                        }
                                        Err(e) => {
                                            tracing::warn!(
                                                "Failed to read memory state: {}",
                                                e
                                            );
                                        }
                                    }
                                }
                                WatchCategory::Screenshot => {
                                    match tokio::fs::metadata(path).await {
                                        Ok(meta) if meta.len() > 0 => {
                                            let orch = screenshot_orchestrator.lock().await;
                                            match orch.index_file(path).await {
                                                Ok(screenshot) => {
                                                    tracing::info!("New screenshot indexed: {}", screenshot.filename);
                                                    if let Err(e) = app_handle.emit("screenshot-new", &screenshot) {
                                                        tracing::warn!("Failed to emit screenshot-new: {}", e);
                                                    }
                                                }
                                                Err(e) => {
                                                    tracing::warn!("Failed to index screenshot: {}", e);
                                                }
                                            }
                                        }
                                        Ok(_) => {
                                            tracing::debug!(
                                                "Screenshot file has zero size, skipping: {:?}",
                                                path
                                            );
                                        }
                                        Err(e) => {
                                            tracing::warn!(
                                                "Failed to read screenshot metadata: {}",
                                                e
                                            );
                                        }
                                    }
                                }
                                WatchCategory::Claude => {
                                    tracing::info!(".claude directory changed: {:?}", path);
                                    let changed_file = path
                                        .file_name()
                                        .and_then(|f| f.to_str())
                                        .unwrap_or("unknown")
                                        .to_string();
                                    if let Err(e) = app_handle.emit(
                                        "claude-dir-change",
                                        serde_json::json!({ "file": changed_file }),
                                    ) {
                                        tracing::warn!(
                                            "Failed to emit claude-dir-change: {}",
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

        self.task_handle = Some(handle);
        Ok(())
    }
}
