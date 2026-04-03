use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScreenshotError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Screenshot not found")]
    #[allow(dead_code)]
    NotFound,

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Directory watch error: {0}")]
    #[allow(dead_code)]
    WatchError(String),

    #[error("Capture error: {0}")]
    #[allow(dead_code)]
    CaptureError(String),
}

pub type Result<T> = std::result::Result<T, ScreenshotError>;
