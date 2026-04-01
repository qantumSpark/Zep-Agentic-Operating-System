use std::path::{Path, PathBuf};

use chrono::Utc;
use uuid::Uuid;

use super::adapters::CaptureAdapter;
use super::index::{IterationIndex, ScreenshotIndex};
use super::manager::{Result, ScreenshotError};
use super::types::{
    CaptureContext, CaptureRequest, CaptureSource, Iteration, IterationStatus, Screenshot,
    ScreenshotMetadata, Viewport,
};

/// Central coordinator for screenshot captures and iteration management.
///
/// Owns the screenshot and iteration indexes and delegates capture
/// mechanics to a pluggable [`CaptureAdapter`].
pub struct ScreenshotOrchestrator {
    screenshots_dir: PathBuf,
    screenshot_index: ScreenshotIndex,
    iteration_index: IterationIndex,
    adapter: Box<dyn CaptureAdapter>,
}

impl ScreenshotOrchestrator {
    /// Create a new orchestrator.
    ///
    /// Both indexes are initialised pointing at `screenshots_dir`.
    pub fn new(screenshots_dir: PathBuf, adapter: Box<dyn CaptureAdapter>) -> Self {
        let screenshot_index = ScreenshotIndex::new(screenshots_dir.clone());
        let iteration_index = IterationIndex::new(screenshots_dir.clone());
        Self {
            screenshots_dir,
            screenshot_index,
            iteration_index,
            adapter,
        }
    }

    /// Get the screenshots directory path
    pub fn screenshots_dir(&self) -> &Path {
        &self.screenshots_dir
    }

    /// Request a new screenshot capture via the configured adapter.
    ///
    /// Generates a UUID capture id, builds a [`CaptureContext`], and
    /// delegates to the adapter's `request_capture`.
    pub async fn request_capture(
        &self,
        url: Option<String>,
        viewport: Option<Viewport>,
        iteration_id: Option<String>,
    ) -> Result<CaptureRequest> {
        let capture_id = Uuid::new_v4().to_string();
        tracing::info!(capture_id = %capture_id, "Requesting capture");

        let ctx = CaptureContext {
            target_dir: self.screenshots_dir.clone(),
            capture_id,
            url,
            viewport,
            iteration_id,
        };

        self.adapter.request_capture(ctx).await
    }

    /// Index a newly-detected image file from the filesystem.
    ///
    /// Called by the FileWatcher when a new image appears in the watched
    /// directory.  Creates a [`Screenshot`] record with
    /// [`CaptureSource::Filesystem`] and adds it to the index.
    pub async fn index_file(&self, file_path: &Path) -> Result<Screenshot> {
        let id = Uuid::new_v4().to_string();
        let filename = file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Store absolute path so the frontend can use convertFileSrc()
        let absolute_path = if file_path.is_absolute() {
            file_path.to_path_buf()
        } else {
            self.screenshots_dir.join(file_path)
        };

        let screenshot = Screenshot {
            id: id.clone(),
            filename,
            path: absolute_path,
            timestamp: Utc::now().to_rfc3339(),
            session_id: None,
            iteration_id: None,
            source: CaptureSource::Filesystem,
            metadata: ScreenshotMetadata {
                url: None,
                viewport: None,
                project_type: None,
                label: None,
            },
        };

        self.screenshot_index.add(screenshot.clone()).await?;
        tracing::info!(id = %id, "Indexed new screenshot from filesystem");
        Ok(screenshot)
    }

    /// Return screenshots for the gallery view.
    ///
    /// If `session_id` is provided, returns only that session's
    /// screenshots; otherwise returns all indexed screenshots.
    pub async fn get_gallery(&self, session_id: Option<&str>) -> Result<Vec<Screenshot>> {
        match session_id {
            Some(sid) => self.screenshot_index.get_by_session(sid).await,
            None => self.screenshot_index.load().await,
        }
    }

    /// Delete a screenshot by id.
    ///
    /// Removes the backing file from disk (handling `NotFound` gracefully)
    /// and removes the entry from the index. Returns whether the entry
    /// was found.
    pub async fn delete_screenshot(&self, id: &str) -> Result<bool> {
        let screenshot = self.screenshot_index.get_by_id(id).await?;
        let Some(screenshot) = screenshot else {
            tracing::debug!(id = %id, "Screenshot not found for deletion");
            return Ok(false);
        };

        let file_path = screenshot.path.clone();

        // Validate path is within screenshots directory before deletion
        if let (Ok(canonical_file), Ok(canonical_dir)) = (
            file_path.canonicalize(),
            self.screenshots_dir.canonicalize(),
        ) {
            if !canonical_file.starts_with(&canonical_dir) {
                return Err(ScreenshotError::Io(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "Cannot delete file outside screenshots directory",
                )));
            }
        }

        match tokio::fs::remove_file(&file_path).await {
            Ok(()) => {
                tracing::info!(path = ?file_path, "Deleted screenshot file");
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::warn!(path = ?file_path, "Screenshot file already gone");
            }
            Err(e) => return Err(ScreenshotError::Io(e)),
        }

        self.screenshot_index.remove(id).await
    }

    /// Return all iterations from the iteration index.
    pub async fn get_iterations(&self) -> Result<Vec<Iteration>> {
        self.iteration_index.load().await
    }

    /// Start a new iteration in the capture-evaluate-fix loop.
    ///
    /// Creates an [`Iteration`] with status [`IterationStatus::Capturing`],
    /// persists it, and returns it.
    pub async fn start_iteration(
        &self,
        target_description: Option<String>,
    ) -> Result<Iteration> {
        let iteration = Iteration {
            id: Uuid::new_v4().to_string(),
            started_at: Utc::now().to_rfc3339(),
            status: IterationStatus::Capturing,
            screenshots: Vec::new(),
            findings: Vec::new(),
            target_description,
        };

        self.iteration_index.upsert(iteration.clone()).await?;
        tracing::info!(id = %iteration.id, "Started new iteration");
        Ok(iteration)
    }

    /// Update the status of an existing iteration.
    pub async fn update_iteration_status(
        &self,
        id: &str,
        status: IterationStatus,
    ) -> Result<()> {
        let mut iteration = self
            .iteration_index
            .get_by_id(id)
            .await?
            .ok_or(ScreenshotError::NotFound)?;

        iteration.status = status;
        self.iteration_index.upsert(iteration).await
    }

    /// Attach a screenshot to an existing iteration.
    pub async fn add_screenshot_to_iteration(
        &self,
        iteration_id: &str,
        screenshot_id: &str,
    ) -> Result<()> {
        let mut iteration = self
            .iteration_index
            .get_by_id(iteration_id)
            .await?
            .ok_or(ScreenshotError::NotFound)?;

        iteration.screenshots.push(screenshot_id.to_string());
        self.iteration_index.upsert(iteration).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screenshots::adapters::FilesystemAdapter;
    use tempfile::tempdir;

    fn make_orchestrator(dir: &Path) -> ScreenshotOrchestrator {
        ScreenshotOrchestrator::new(dir.to_path_buf(), Box::new(FilesystemAdapter))
    }

    #[tokio::test]
    async fn test_request_capture_returns_filesystem_pending() {
        let dir = tempdir().expect("tempdir");
        let orch = make_orchestrator(dir.path());

        let req = orch.request_capture(None, None, None).await.expect("capture");
        match req {
            CaptureRequest::FilesystemPending { expected_path } => {
                assert!(expected_path.starts_with(dir.path()));
                assert!(expected_path.to_string_lossy().ends_with(".png"));
            }
            _ => panic!("Expected FilesystemPending"),
        }
    }

    #[tokio::test]
    async fn test_index_file_and_get_gallery() {
        let dir = tempdir().expect("tempdir");
        let orch = make_orchestrator(dir.path());

        // Create a dummy file
        let file_path = dir.path().join("test.png");
        tokio::fs::write(&file_path, b"fake png").await.expect("write");

        let screenshot = orch.index_file(&file_path).await.expect("index_file");
        assert_eq!(screenshot.filename, "test.png");

        let gallery = orch.get_gallery(None).await.expect("gallery");
        assert_eq!(gallery.len(), 1);
        assert_eq!(gallery[0].id, screenshot.id);
    }

    #[tokio::test]
    async fn test_delete_screenshot() {
        let dir = tempdir().expect("tempdir");
        let orch = make_orchestrator(dir.path());

        let file_path = dir.path().join("to_delete.png");
        tokio::fs::write(&file_path, b"data").await.expect("write");

        let screenshot = orch.index_file(&file_path).await.expect("index");
        assert!(file_path.exists());

        let deleted = orch.delete_screenshot(&screenshot.id).await.expect("delete");
        assert!(deleted);
        assert!(!file_path.exists());

        // Deleting again returns false
        let again = orch.delete_screenshot(&screenshot.id).await.expect("delete");
        assert!(!again);
    }

    #[tokio::test]
    async fn test_delete_missing_file_graceful() {
        let dir = tempdir().expect("tempdir");
        let orch = make_orchestrator(dir.path());

        // Index a file, then delete the file manually before calling delete_screenshot
        let file_path = dir.path().join("gone.png");
        tokio::fs::write(&file_path, b"data").await.expect("write");
        let screenshot = orch.index_file(&file_path).await.expect("index");
        tokio::fs::remove_file(&file_path).await.expect("manual remove");

        // Should still succeed and remove from index
        let deleted = orch.delete_screenshot(&screenshot.id).await.expect("delete");
        assert!(deleted);
    }

    #[tokio::test]
    async fn test_iteration_lifecycle() {
        let dir = tempdir().expect("tempdir");
        let orch = make_orchestrator(dir.path());

        // Start
        let iter = orch
            .start_iteration(Some("Fix button color".to_string()))
            .await
            .expect("start");
        assert!(matches!(iter.status, IterationStatus::Capturing));
        assert_eq!(iter.target_description.as_deref(), Some("Fix button color"));

        // List
        let all = orch.get_iterations().await.expect("list");
        assert_eq!(all.len(), 1);

        // Update status
        orch.update_iteration_status(&iter.id, IterationStatus::Evaluating)
            .await
            .expect("update status");

        let updated = orch.get_iterations().await.expect("list");
        assert!(matches!(updated[0].status, IterationStatus::Evaluating));

        // Add screenshot
        orch.add_screenshot_to_iteration(&iter.id, "shot-1")
            .await
            .expect("add screenshot");

        let final_iter = orch.get_iterations().await.expect("list");
        assert_eq!(final_iter[0].screenshots, vec!["shot-1".to_string()]);
    }

    #[tokio::test]
    async fn test_update_nonexistent_iteration_fails() {
        let dir = tempdir().expect("tempdir");
        let orch = make_orchestrator(dir.path());

        let result = orch
            .update_iteration_status("nonexistent", IterationStatus::Done)
            .await;
        assert!(result.is_err());
    }
}
