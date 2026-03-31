use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::session::SessionManager;
use super::manager::{Result, ScreenshotError};
use super::types::{CaptureContext, CaptureRequest};

/// Trait for screenshot capture adapters.
///
/// Each adapter knows how to request a screenshot capture through
/// a specific mechanism (filesystem watch, CLI MCP call, etc.).
#[async_trait]
pub trait CaptureAdapter: Send + Sync {
    /// Human-readable name of this adapter
    fn name(&self) -> &str;

    /// Request a screenshot capture. Returns a CaptureRequest describing
    /// what was done (pending filesystem watch, or CLI prompt sent).
    async fn request_capture(
        &self,
        ctx: CaptureContext,
    ) -> Result<CaptureRequest>;
}

/// The simplest adapter — it doesn't actively capture anything.
/// It returns a `FilesystemPending` with the expected path where
/// a file should appear. The FileWatcher will detect it.
pub struct FilesystemAdapter;

#[async_trait]
impl CaptureAdapter for FilesystemAdapter {
    fn name(&self) -> &str {
        "filesystem"
    }

    async fn request_capture(
        &self,
        ctx: CaptureContext,
    ) -> Result<CaptureRequest> {
        let expected_path = ctx.expected_path();
        tracing::info!("FilesystemAdapter: waiting for file at {:?}", expected_path);
        Ok(CaptureRequest::FilesystemPending { expected_path })
    }
}

/// Adapter that sends a formatted prompt to the Claude CLI session
/// to trigger an MCP screenshot capture (chrome-devtools, GoPeak, etc.).
pub struct CliMcpAdapter {
    session_manager: Arc<Mutex<SessionManager>>,
}

impl CliMcpAdapter {
    pub fn new(session_manager: Arc<Mutex<SessionManager>>) -> Self {
        Self { session_manager }
    }

    /// Format a capture prompt based on the project context
    fn format_capture_prompt(ctx: &CaptureContext) -> String {
        let save_path = ctx.expected_path();

        let mut prompt = String::new();

        if let Some(url) = &ctx.url {
            // Web project — use chrome-devtools MCP
            prompt.push_str(&format!(
                "Take a screenshot of the page at {} using the take_screenshot MCP tool. Save it to {}.",
                url,
                save_path.display()
            ));
        } else {
            // Generic — just ask for a screenshot
            prompt.push_str(&format!(
                "Take a screenshot and save it to {}.",
                save_path.display()
            ));
        }

        if let Some(viewport) = &ctx.viewport {
            prompt.push_str(&format!(
                " Use viewport dimensions {}x{}.",
                viewport.width, viewport.height
            ));
        }

        prompt
    }
}

#[async_trait]
impl CaptureAdapter for CliMcpAdapter {
    fn name(&self) -> &str {
        "cli-mcp"
    }

    async fn request_capture(
        &self,
        ctx: CaptureContext,
    ) -> Result<CaptureRequest> {
        let prompt = Self::format_capture_prompt(&ctx);
        let expected_path = ctx.expected_path();

        tracing::info!("CliMcpAdapter: sending capture prompt to CLI");

        let mut session = self.session_manager.lock().await;
        session
            .send_message(&prompt)
            .await
            .map_err(|e| ScreenshotError::CaptureError(format!("Failed to send CLI prompt: {}", e)))?;

        Ok(CaptureRequest::CliPromptSent {
            prompt,
            expected_path,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::Viewport;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_filesystem_adapter_name() {
        let adapter = FilesystemAdapter;
        assert_eq!(adapter.name(), "filesystem");
    }

    #[tokio::test]
    async fn test_filesystem_adapter_returns_pending() {
        let adapter = FilesystemAdapter;
        let ctx = CaptureContext {
            target_dir: PathBuf::from("/tmp/screenshots"),
            capture_id: "test-001".to_string(),
            url: None,
            viewport: None,
            iteration_id: None,
        };

        let result = adapter.request_capture(ctx).await.unwrap();
        match result {
            CaptureRequest::FilesystemPending { expected_path } => {
                assert_eq!(expected_path, PathBuf::from("/tmp/screenshots/test-001.png"));
            }
            _ => panic!("Expected FilesystemPending variant"),
        }
    }

    #[test]
    fn test_cli_mcp_prompt_with_url() {
        let ctx = CaptureContext {
            target_dir: PathBuf::from("/tmp/screenshots"),
            capture_id: "web-001".to_string(),
            url: Some("http://localhost:3000".to_string()),
            viewport: None,
            iteration_id: None,
        };

        let prompt = CliMcpAdapter::format_capture_prompt(&ctx);
        assert!(prompt.contains("take_screenshot"), "should reference MCP tool");
        assert!(prompt.contains("http://localhost:3000"), "should contain the URL");
        assert!(prompt.contains("web-001.png"), "should contain the expected filename");
    }

    #[test]
    fn test_cli_mcp_prompt_without_url() {
        let ctx = CaptureContext {
            target_dir: PathBuf::from("/tmp/screenshots"),
            capture_id: "generic-001".to_string(),
            url: None,
            viewport: None,
            iteration_id: None,
        };

        let prompt = CliMcpAdapter::format_capture_prompt(&ctx);
        assert!(prompt.contains("Take a screenshot"), "should contain generic capture instruction");
        assert!(!prompt.contains("take_screenshot"), "should not reference MCP tool without URL");
    }

    #[test]
    fn test_cli_mcp_prompt_with_viewport() {
        let ctx = CaptureContext {
            target_dir: PathBuf::from("/tmp/screenshots"),
            capture_id: "vp-001".to_string(),
            url: Some("http://localhost:3000".to_string()),
            viewport: Some(Viewport { width: 1920, height: 1080 }),
            iteration_id: None,
        };

        let prompt = CliMcpAdapter::format_capture_prompt(&ctx);
        assert!(prompt.contains("1920x1080"), "should contain viewport dimensions");
    }
}
