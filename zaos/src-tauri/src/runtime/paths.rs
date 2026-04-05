use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Runtime kind — currently only Claude is supported.
///
/// ## Adding a new runtime (e.g. Codex)
///
/// 1. Add a variant to this enum (e.g. `Codex`)
/// 2. Extend `RuntimePaths::for_kind()` with the new directory layout
/// 3. Implement the `AgentRuntime` trait (`runtime/mod.rs`) for the new runtime
/// 4. Add a native event mapper in `events/` to translate runtime events → `ZaosEvent`
/// 5. Add a deployer in `deployer/` if the runtime needs embedded config files
///
/// ## What stays runtime-specific (not abstracted)
///
/// - Deployer content: hook format, settings structure (each runtime has its own)
/// - Event mapper: translates native events to ZaosEvent (one mapper per runtime)
/// - CLI interaction: how sessions are started/stopped (AgentRuntime trait impl)
///
/// ## What is already runtime-agnostic
///
/// - ZaosEvent model (frontend never sees raw runtime events)
/// - Workflow pipeline, phases, gates
/// - Product contract, personas, session insights
/// - Policy engine, permission system
/// - Dashboard UI (consumes ZaosEvent + product contract only)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeKind {
    #[default]
    Claude,
}

impl RuntimeKind {
    /// Human-readable name shown in the frontend dashboard.
    /// Used by `get_runtime_info` IPC command.
    pub fn display_name(&self) -> &'static str {
        match self {
            RuntimeKind::Claude => "Claude Code CLI",
        }
    }
}

/// Centralized paths for a given runtime.
/// All code that previously hardcoded `.claude/` should use these paths instead.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimePaths {
    pub base_dir: PathBuf,
    pub agents_dir: PathBuf,
    pub rules_dir: PathBuf,
    pub settings_file: PathBuf,
}

impl RuntimePaths {
    /// Build paths for a given runtime kind.
    /// Each runtime has its own directory convention:
    /// - Claude: `.claude/` with `agents/`, `rules/`, `settings.json`
    /// - Future runtimes: add a match arm here with the appropriate layout
    pub fn for_kind(project_dir: &Path, kind: &RuntimeKind) -> Self {
        match kind {
            RuntimeKind::Claude => {
                let base = project_dir.join(".claude");
                Self {
                    agents_dir: base.join("agents"),
                    rules_dir: base.join("rules"),
                    settings_file: base.join("settings.json"),
                    base_dir: base,
                }
            }
        }
    }
}
