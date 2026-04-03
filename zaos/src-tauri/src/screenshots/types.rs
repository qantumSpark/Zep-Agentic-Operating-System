use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A captured screenshot with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Screenshot {
    pub id: String,
    pub filename: String,
    pub path: PathBuf,
    pub timestamp: String,
    pub session_id: Option<String>,
    pub iteration_id: Option<String>,
    pub source: CaptureSource,
    pub metadata: ScreenshotMetadata,
}

/// How the screenshot was captured
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CaptureSource {
    CliMcp,
    Filesystem,
    Manual,
}

/// Additional metadata attached to a screenshot
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotMetadata {
    pub url: Option<String>,
    pub viewport: Option<Viewport>,
    pub project_type: Option<String>,
    pub label: Option<String>,
}

/// Viewport dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

/// An iteration in the capture-evaluate-fix loop
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct Iteration {
    pub id: String,
    pub started_at: String,
    pub status: IterationStatus,
    pub screenshots: Vec<String>,
    pub findings: Vec<String>,
    pub target_description: Option<String>,
}

/// Status of an iteration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub enum IterationStatus {
    Capturing,
    Evaluating,
    Fixing,
    Comparing,
    Done,
}

/// Detected project type with type-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
#[allow(dead_code)]
pub enum ProjectType {
    Web { base_url: Option<String> },
    Godot,
    Flutter,
    TauriApp,
    Manual,
}

/// Context passed to capture adapters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureContext {
    pub target_dir: PathBuf,
    pub capture_id: String,
    pub url: Option<String>,
    pub viewport: Option<Viewport>,
    pub iteration_id: Option<String>,
}

impl CaptureContext {
    /// Expected file path for the capture output.
    pub fn expected_path(&self) -> PathBuf {
        self.target_dir.join(format!("{}.png", self.capture_id))
    }
}

/// Result of a capture request from an adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "kind")]
pub enum CaptureRequest {
    FilesystemPending { expected_path: PathBuf },
    CliPromptSent { prompt: String, expected_path: PathBuf },
}
