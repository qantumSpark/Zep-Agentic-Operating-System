use crate::events::types::CliEvent;
use std::io::Error as IoError;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::ChildStdout;
use tokio::sync::broadcast;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("IO error: {0}")]
    Io(#[from] IoError),

    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Channel send error")]
    #[allow(dead_code)]
    ChannelSend,
}

pub type Result<T> = std::result::Result<T, ParserError>;

/// Async JSONL parser: reads lines from ChildStdout, deserializes to CliEvent,
/// broadcasts via tokio broadcast channel
pub async fn parse_stream(
    stdout: ChildStdout,
    tx: broadcast::Sender<CliEvent>,
) -> Result<()> {
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        if line.is_empty() {
            continue;
        }

        match serde_json::from_str::<CliEvent>(&line) {
            Ok(event) => {
                // Log parse success for debugging
                tracing::debug!("Parsed CLI event: {:?}", event);

                // Send to broadcast channel, ignore if no receivers
                let _ = tx.send(event);
            }
            Err(e) => {
                let preview = &line[..500.min(line.len())];
                tracing::warn!("Parse error: {} on line: {}", e, preview);
                // Continue parsing on errors (forward compatibility)
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_deserialization() {
        let json = r#"{"type":"system","subtype":"init","session_id":"test","uuid":"uuid"}"#;
        let event: CliEvent = serde_json::from_str(json).unwrap();
        assert!(matches!(event, CliEvent::System(_)));
    }

    #[test]
    fn test_control_request_deserialization() {
        let json = r#"{"type":"control_request","request_id":"req_123","tool":"Write","input":{"file_path":"/foo/bar.ts"},"message":"Claude wants to write","session_id":"sess_1","uuid":"uuid_1"}"#;
        let event: CliEvent = serde_json::from_str(json).unwrap();
        match event {
            CliEvent::ControlRequest(req) => {
                assert_eq!(req.request_id, "req_123");
                assert_eq!(req.tool.as_deref(), Some("Write"));
            }
            _ => panic!("Expected ControlRequest variant"),
        }
    }
}
