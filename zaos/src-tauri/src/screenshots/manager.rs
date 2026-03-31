use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScreenshotError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Screenshot not found")]
    NotFound,

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Directory watch error: {0}")]
    WatchError(String),

    #[error("Capture error: {0}")]
    CaptureError(String),
}

pub type Result<T> = std::result::Result<T, ScreenshotError>;
