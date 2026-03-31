use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowKitConfig {
    /// File extensions that block-code hook will prevent writing without a plan
    pub blocked_extensions: Vec<String>,
    /// Auto-sync agents/hooks/rules on session start
    pub auto_sync: bool,
    /// Enable/disable all hooks
    pub hooks_enabled: bool,
    /// List of disabled rule filenames (e.g., ["03-code-review.mdc"])
    pub disabled_rules: Vec<String>,
}

impl Default for WorkflowKitConfig {
    fn default() -> Self {
        Self {
            blocked_extensions: vec![
                ".rs".into(),
                ".ts".into(),
                ".tsx".into(),
                ".js".into(),
                ".jsx".into(),
                ".css".into(),
            ],
            auto_sync: true,
            hooks_enabled: true,
            disabled_rules: vec![],
        }
    }
}

impl WorkflowKitConfig {
    /// Read config from .zaos/config.json, or return default if not found
    pub fn load(project_dir: &Path) -> Self {
        let path = project_dir.join(".zaos").join("config.json");
        match std::fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|e| {
                tracing::warn!("Invalid .zaos/config.json: {}, using defaults", e);
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }

    /// Save config to .zaos/config.json
    pub fn save(
        &self,
        project_dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let zaos_dir = project_dir.join(".zaos");
        std::fs::create_dir_all(&zaos_dir)?;
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(zaos_dir.join("config.json"), contents)?;
        tracing::info!("Saved .zaos/config.json");
        Ok(())
    }
}
