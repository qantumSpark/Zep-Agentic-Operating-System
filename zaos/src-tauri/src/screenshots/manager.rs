use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScreenshotError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Screenshot not found")]
    NotFound,

    #[error("Directory watch error: {0}")]
    WatchError(String),
}

pub type Result<T> = std::result::Result<T, ScreenshotError>;

/// Screenshot metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Screenshot {
    pub id: String,
    pub timestamp: String,
    pub path: PathBuf,
    pub session_id: String,
    pub tool_use_id: Option<String>,
}

/// ScreenshotManager handles screenshot capture and gallery
pub struct ScreenshotManager {
    watch_dir: PathBuf,
}

impl ScreenshotManager {
    pub fn new(watch_dir: PathBuf) -> Self {
        ScreenshotManager { watch_dir }
    }

    /// Watch directory for new screenshots
    /// TODO: Implement file watcher with notify crate
    pub async fn watch_directory(&self) -> Result<()> {
        tracing::info!("Watching directory: {:?}", self.watch_dir);
        // TODO: Use notify::Watcher to monitor directory
        Ok(())
    }

    /// Capture screenshot on demand via GoPeak MCP
    /// TODO: Implement MCP call to GoPeak
    pub async fn capture_on_demand(&self) -> Result<Screenshot> {
        tracing::info!("Capturing screenshot on demand");
        // TODO: Call GoPeak MCP tool to capture screenshot
        Err(ScreenshotError::NotFound)
    }

    /// Compare two screenshots
    pub async fn compare(&self, before: &Screenshot, after: &Screenshot) -> Result<String> {
        tracing::info!("Comparing screenshots: {:?} vs {:?}", before.path, after.path);
        // TODO: Implement visual diff
        Ok("diff result".to_string())
    }

    /// Get gallery of screenshots for session
    pub async fn get_gallery(&self, session_id: &str) -> Result<Vec<Screenshot>> {
        tracing::info!("Getting gallery for session: {}", session_id);
        // TODO: Query screenshots from session metadata
        Ok(Vec::new())
    }

    /// Cleanup old screenshots
    pub async fn cleanup_old(&self, max_age_days: u32) -> Result<()> {
        tracing::info!("Cleaning up screenshots older than {} days", max_age_days);
        // TODO: Implement cleanup logic
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screenshot_manager_creation() {
        let manager = ScreenshotManager::new(PathBuf::from("/tmp"));
        assert!(!manager.watch_dir.as_os_str().is_empty());
    }
}
