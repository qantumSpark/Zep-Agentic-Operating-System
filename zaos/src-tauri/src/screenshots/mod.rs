pub mod adapters;
pub mod index;
pub mod manager;
pub mod orchestrator;
pub mod types;

pub use adapters::{CaptureAdapter, CliMcpAdapter, FilesystemAdapter};
pub use index::{IterationIndex, ScreenshotIndex};
pub use manager::ScreenshotError;
pub use orchestrator::ScreenshotOrchestrator;
pub use types::*;
