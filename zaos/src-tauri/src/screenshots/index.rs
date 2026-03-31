use std::path::PathBuf;

use super::manager::Result;
use super::manager::ScreenshotError;
use super::types::{Iteration, Screenshot};

/// JSON index manager for screenshots, persisted to `.screenshots/index.json`.
pub struct ScreenshotIndex {
    index_path: PathBuf,
}

impl ScreenshotIndex {
    /// Create a new index manager pointing at `screenshots_dir/index.json`.
    pub fn new(screenshots_dir: PathBuf) -> Self {
        Self {
            index_path: screenshots_dir.join("index.json"),
        }
    }

    /// Load all screenshots from the index file.
    ///
    /// Returns an empty vec if the file does not exist (no TOCTOU: we
    /// attempt the read and handle `NotFound` from the result).
    pub async fn load(&self) -> Result<Vec<Screenshot>> {
        let content = match tokio::fs::read_to_string(&self.index_path).await {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::debug!("index.json not found, returning empty vec");
                return Ok(Vec::new());
            }
            Err(e) => return Err(ScreenshotError::Io(e)),
        };

        let screenshots: Vec<Screenshot> = serde_json::from_str(&content)?;
        tracing::debug!("Loaded {} screenshots from index", screenshots.len());
        Ok(screenshots)
    }

    /// Atomically save the screenshot list to index.json.
    ///
    /// Writes to a `.json.tmp` file first, then renames to avoid partial writes.
    pub async fn save(&self, screenshots: &[Screenshot]) -> Result<()> {
        let content = serde_json::to_string_pretty(screenshots)?;
        let tmp_path = self.index_path.with_extension("json.tmp");
        tokio::fs::write(&tmp_path, content).await?;
        tokio::fs::rename(&tmp_path, &self.index_path).await?;
        tracing::debug!("Saved {} screenshots to index", screenshots.len());
        Ok(())
    }

    /// Add a screenshot to the index (load, push, save).
    pub async fn add(&self, screenshot: Screenshot) -> Result<()> {
        let mut screenshots = self.load().await?;
        screenshots.push(screenshot);
        self.save(&screenshots).await
    }

    /// Remove a screenshot by id. Returns `true` if it was found and removed.
    pub async fn remove(&self, id: &str) -> Result<bool> {
        let mut screenshots = self.load().await?;
        let before_len = screenshots.len();
        screenshots.retain(|s| s.id != id);
        let removed = screenshots.len() < before_len;
        self.save(&screenshots).await?;
        Ok(removed)
    }

    /// Find a screenshot by id.
    pub async fn get_by_id(&self, id: &str) -> Result<Option<Screenshot>> {
        let screenshots = self.load().await?;
        Ok(screenshots.into_iter().find(|s| s.id == id))
    }

    /// Return all screenshots belonging to a given session.
    pub async fn get_by_session(&self, session_id: &str) -> Result<Vec<Screenshot>> {
        let screenshots = self.load().await?;
        Ok(screenshots
            .into_iter()
            .filter(|s| s.session_id.as_deref() == Some(session_id))
            .collect())
    }

    /// Return all screenshots belonging to a given iteration.
    pub async fn get_by_iteration(&self, iteration_id: &str) -> Result<Vec<Screenshot>> {
        let screenshots = self.load().await?;
        Ok(screenshots
            .into_iter()
            .filter(|s| s.iteration_id.as_deref() == Some(iteration_id))
            .collect())
    }
}

/// JSON index manager for iterations, persisted to `.screenshots/iterations.json`.
pub struct IterationIndex {
    index_path: PathBuf,
}

impl IterationIndex {
    /// Create a new index manager pointing at `screenshots_dir/iterations.json`.
    pub fn new(screenshots_dir: PathBuf) -> Self {
        Self {
            index_path: screenshots_dir.join("iterations.json"),
        }
    }

    /// Load all iterations from the index file.
    ///
    /// Returns an empty vec if the file does not exist.
    pub async fn load(&self) -> Result<Vec<Iteration>> {
        let content = match tokio::fs::read_to_string(&self.index_path).await {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::debug!("iterations.json not found, returning empty vec");
                return Ok(Vec::new());
            }
            Err(e) => return Err(ScreenshotError::Io(e)),
        };

        let iterations: Vec<Iteration> = serde_json::from_str(&content)?;
        tracing::debug!("Loaded {} iterations from index", iterations.len());
        Ok(iterations)
    }

    /// Atomically save the iteration list to iterations.json.
    pub async fn save(&self, iterations: &[Iteration]) -> Result<()> {
        let content = serde_json::to_string_pretty(iterations)?;
        let tmp_path = self.index_path.with_extension("json.tmp");
        tokio::fs::write(&tmp_path, content).await?;
        tokio::fs::rename(&tmp_path, &self.index_path).await?;
        tracing::debug!("Saved {} iterations to index", iterations.len());
        Ok(())
    }

    /// Insert or update an iteration by id (load, find-or-append, save).
    pub async fn upsert(&self, iteration: Iteration) -> Result<()> {
        let mut iterations = self.load().await?;
        if let Some(existing) = iterations.iter_mut().find(|i| i.id == iteration.id) {
            *existing = iteration;
        } else {
            iterations.push(iteration);
        }
        self.save(&iterations).await
    }

    /// Find an iteration by id.
    pub async fn get_by_id(&self, id: &str) -> Result<Option<Iteration>> {
        let iterations = self.load().await?;
        Ok(iterations.into_iter().find(|i| i.id == id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screenshots::types::{CaptureSource, IterationStatus, ScreenshotMetadata};
    use tempfile::tempdir;

    /// Helper to create a test screenshot with the given id.
    fn make_screenshot(id: &str) -> Screenshot {
        Screenshot {
            id: id.to_string(),
            filename: format!("{}.png", id),
            path: PathBuf::from(format!("/tmp/{}.png", id)),
            timestamp: "2026-03-31T12:00:00Z".to_string(),
            session_id: Some("session-1".to_string()),
            iteration_id: Some("iter-1".to_string()),
            source: CaptureSource::Manual,
            metadata: ScreenshotMetadata {
                url: None,
                viewport: None,
                project_type: None,
                label: None,
            },
        }
    }

    /// Helper to create a test iteration with the given id.
    fn make_iteration(id: &str) -> Iteration {
        Iteration {
            id: id.to_string(),
            started_at: "2026-03-31T12:00:00Z".to_string(),
            status: IterationStatus::Capturing,
            screenshots: Vec::new(),
            findings: Vec::new(),
            target_description: None,
        }
    }

    // ---- ScreenshotIndex tests ----

    #[tokio::test]
    async fn test_load_empty() {
        let dir = tempdir().expect("tempdir");
        let index = ScreenshotIndex::new(dir.path().to_path_buf());
        let result = index.load().await.expect("load should succeed");
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_add_and_load() {
        let dir = tempdir().expect("tempdir");
        let index = ScreenshotIndex::new(dir.path().to_path_buf());

        let shot = make_screenshot("s1");
        index.add(shot.clone()).await.expect("add should succeed");

        let loaded = index.load().await.expect("load should succeed");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "s1");
    }

    #[tokio::test]
    async fn test_remove() {
        let dir = tempdir().expect("tempdir");
        let index = ScreenshotIndex::new(dir.path().to_path_buf());

        index.add(make_screenshot("s1")).await.expect("add");
        index.add(make_screenshot("s2")).await.expect("add");

        let removed = index.remove("s1").await.expect("remove");
        assert!(removed);

        let remaining = index.load().await.expect("load");
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, "s2");

        // Removing a non-existent id returns false
        let not_removed = index.remove("nonexistent").await.expect("remove");
        assert!(!not_removed);
    }

    #[tokio::test]
    async fn test_get_by_id() {
        let dir = tempdir().expect("tempdir");
        let index = ScreenshotIndex::new(dir.path().to_path_buf());

        index.add(make_screenshot("s1")).await.expect("add");
        index.add(make_screenshot("s2")).await.expect("add");

        let found = index.get_by_id("s2").await.expect("get_by_id");
        assert!(found.is_some());
        assert_eq!(found.expect("exists").id, "s2");

        let missing = index.get_by_id("s99").await.expect("get_by_id");
        assert!(missing.is_none());
    }

    #[tokio::test]
    async fn test_get_by_session() {
        let dir = tempdir().expect("tempdir");
        let index = ScreenshotIndex::new(dir.path().to_path_buf());

        let mut s1 = make_screenshot("s1");
        s1.session_id = Some("sess-a".to_string());
        let mut s2 = make_screenshot("s2");
        s2.session_id = Some("sess-b".to_string());
        let mut s3 = make_screenshot("s3");
        s3.session_id = Some("sess-a".to_string());

        index.add(s1).await.expect("add");
        index.add(s2).await.expect("add");
        index.add(s3).await.expect("add");

        let results = index.get_by_session("sess-a").await.expect("get_by_session");
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|s| s.session_id.as_deref() == Some("sess-a")));
    }

    #[tokio::test]
    async fn test_get_by_iteration() {
        let dir = tempdir().expect("tempdir");
        let index = ScreenshotIndex::new(dir.path().to_path_buf());

        let mut s1 = make_screenshot("s1");
        s1.iteration_id = Some("iter-x".to_string());
        let mut s2 = make_screenshot("s2");
        s2.iteration_id = Some("iter-y".to_string());

        index.add(s1).await.expect("add");
        index.add(s2).await.expect("add");

        let results = index
            .get_by_iteration("iter-x")
            .await
            .expect("get_by_iteration");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "s1");
    }

    // ---- IterationIndex tests ----

    #[tokio::test]
    async fn test_iteration_load_empty() {
        let dir = tempdir().expect("tempdir");
        let index = IterationIndex::new(dir.path().to_path_buf());
        let result = index.load().await.expect("load");
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_iteration_upsert_insert() {
        let dir = tempdir().expect("tempdir");
        let index = IterationIndex::new(dir.path().to_path_buf());

        index
            .upsert(make_iteration("i1"))
            .await
            .expect("upsert");

        let loaded = index.load().await.expect("load");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "i1");
    }

    #[tokio::test]
    async fn test_iteration_upsert_update() {
        let dir = tempdir().expect("tempdir");
        let index = IterationIndex::new(dir.path().to_path_buf());

        index
            .upsert(make_iteration("i1"))
            .await
            .expect("upsert");

        let mut updated = make_iteration("i1");
        updated.status = IterationStatus::Done;
        updated.findings = vec!["found something".to_string()];
        index.upsert(updated).await.expect("upsert update");

        let loaded = index.load().await.expect("load");
        assert_eq!(loaded.len(), 1);
        assert!(matches!(loaded[0].status, IterationStatus::Done));
        assert_eq!(loaded[0].findings.len(), 1);
    }

    #[tokio::test]
    async fn test_iteration_get_by_id() {
        let dir = tempdir().expect("tempdir");
        let index = IterationIndex::new(dir.path().to_path_buf());

        index
            .upsert(make_iteration("i1"))
            .await
            .expect("upsert");
        index
            .upsert(make_iteration("i2"))
            .await
            .expect("upsert");

        let found = index.get_by_id("i2").await.expect("get_by_id");
        assert!(found.is_some());
        assert_eq!(found.expect("exists").id, "i2");

        let missing = index.get_by_id("i99").await.expect("get_by_id");
        assert!(missing.is_none());
    }
}
