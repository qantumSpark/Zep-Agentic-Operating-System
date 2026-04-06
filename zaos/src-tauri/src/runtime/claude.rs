use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::sync::broadcast;

use crate::events::types::CliEvent;
use crate::events::claude_mapper::map_cli_event;
use crate::events::zaos_events::ZaosEvent;
use super::{AgentRuntime, Result, RuntimeError, SessionRecord};

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len])
    } else {
        s.to_string()
    }
}

fn encode_project_path(path: &std::path::Path) -> String {
    let s = path.to_string_lossy();
    s.replace('\\', "-")
        .replace('/', "-")
        .replace(':', "-")
        .replace(' ', "-")
}

/// Claude Code CLI runtime implementation.
///
/// Spawns `claude` as a long-lived subprocess with `--output-format stream-json`
/// and communicates via stdin/stdout JSONL.
pub struct ClaudeRuntime {
    session_id: Option<String>,
    project_dir: PathBuf,
    child: Option<tokio::process::Child>,
    stdin: Option<tokio::process::ChildStdin>,
}

impl ClaudeRuntime {
    pub fn new(project_dir: PathBuf) -> Self {
        Self {
            session_id: None,
            project_dir,
            child: None,
            stdin: None,
        }
    }

    /// Write a JSON value as a single JSONL line to the CLI stdin.
    async fn write_jsonl(&mut self, value: serde_json::Value) -> Result<()> {
        let stdin = self.stdin.as_mut().ok_or(RuntimeError::NotStarted)?;
        let mut line = serde_json::to_string(&value)
            .map_err(|e| RuntimeError::Protocol(format!("Failed to serialize: {}", e)))?;
        line.push('\n');
        stdin.write_all(line.as_bytes()).await.map_err(RuntimeError::Io)?;
        stdin.flush().await.map_err(RuntimeError::Io)?;
        Ok(())
    }
}

#[async_trait]
impl AgentRuntime for ClaudeRuntime {
    fn name(&self) -> &str {
        "Claude Code CLI"
    }

    async fn check_auth(&self) -> Result<String> {
        let output = Command::new("claude")
            .arg("--version")
            .output()
            .await
            .map_err(|_| RuntimeError::CliNotFound)?;

        if !output.status.success() {
            return Err(RuntimeError::CliNotFound);
        }

        let version = String::from_utf8_lossy(&output.stdout);
        tracing::info!("Claude CLI version: {}", version);
        Ok(version.to_string())
    }

    async fn start_session(&mut self) -> Result<broadcast::Receiver<ZaosEvent>> {
        let (tx, rx) = broadcast::channel(512);

        let mut cmd = Command::new("claude");
        cmd.arg("-p")
            .arg("--input-format")
            .arg("stream-json")
            .arg("--output-format")
            .arg("stream-json")
            .arg("--verbose")
            .arg("--include-partial-messages")
            .arg("--permission-prompt-tool")
            .arg("stdio");

        cmd.current_dir(&self.project_dir);
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            tracing::error!("Failed to spawn Claude CLI: {}", e);
            RuntimeError::Io(e)
        })?;
        tracing::info!("Claude CLI session started (long-lived)");

        // Take stdin handle for sending messages
        let stdin = child.stdin.take().ok_or(RuntimeError::Io(
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to pipe stdin"),
        ))?;
        self.stdin = Some(stdin);

        // Take stdout for event parsing
        let stdout = child.stdout.take().ok_or(RuntimeError::Io(
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to pipe stdout"),
        ))?;

        // Spawn stderr reader
        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                let reader = tokio::io::BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::warn!("[CLI stderr] {}", line);
                }
            });
        }

        // Store child for lifecycle management
        self.child = Some(child);

        // Spawn JSONL parser task — parse CliEvent then map to ZaosEvent
        tokio::spawn(async move {
            let reader = tokio::io::BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if line.is_empty() { continue; }
                match serde_json::from_str::<CliEvent>(&line) {
                    Ok(cli_event) => {
                        tracing::debug!("Parsed CLI event: {:?}", cli_event);
                        for zaos_event in map_cli_event(&cli_event) {
                            let _ = tx.send(zaos_event);
                        }
                    }
                    Err(e) => {
                        let preview = &line[..500.min(line.len())];
                        tracing::warn!("Parse error: {} on line: {}", e, preview);
                    }
                }
            }
        });

        Ok(rx)
    }

    async fn send_message(&mut self, prompt: &str) -> Result<()> {
        let mut msg = serde_json::json!({
            "type": "user",
            "message": {
                "role": "user",
                "content": prompt
            },
            "parent_tool_use_id": null
        });

        // Include session_id for resume if we have one
        if let Some(ref sid) = self.session_id {
            msg["session_id"] = serde_json::Value::String(sid.clone());
        }

        self.write_jsonl(msg).await?;
        tracing::info!("Sent user message to CLI");
        Ok(())
    }

    async fn send_permission_response(
        &mut self,
        id: &str,
        allow: bool,
        tool_input: Option<serde_json::Value>,
    ) -> Result<()> {
        let result_json = if allow {
            serde_json::json!({
                "behavior": "allow",
                "updatedInput": tool_input.unwrap_or(serde_json::json!({}))
            })
        } else {
            serde_json::json!({
                "behavior": "deny",
                "message": "User denied this action"
            })
        };

        let response = serde_json::json!({
            "type": "control_response",
            "response": {
                "subtype": "success",
                "request_id": id,
                "response": result_json
            }
        });

        tracing::info!("Sent permission response: id={}, allow={}", id, allow);
        self.write_jsonl(response).await?;
        Ok(())
    }

    async fn interrupt(&mut self) -> Result<()> {
        if let Some(ref child) = self.child {
            if let Some(pid) = child.id() {
                tracing::info!("Interrupting CLI process with PID: {}", pid);
                #[cfg(target_os = "windows")]
                {
                    let _ = Command::new("taskkill")
                        .args(["/PID", &pid.to_string(), "/T", "/F"])
                        .output()
                        .await;
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let pid_i32 = i32::try_from(pid).map_err(|_| {
                        RuntimeError::Protocol(format!("PID {} exceeds i32::MAX", pid))
                    })?;
                    // SAFETY: pid is a valid process ID obtained from Child::id().
                    let ret = unsafe { libc::kill(pid_i32, libc::SIGTERM) };
                    if ret != 0 {
                        tracing::warn!(
                            "libc::kill returned {}, process may have already exited",
                            ret
                        );
                    }
                }
            }
        }
        self.child = None;
        self.stdin = None;
        Ok(())
    }

    fn is_session_active(&self) -> bool {
        self.child.is_some()
    }

    fn set_session_id(&mut self, id: String) {
        self.session_id = Some(id);
    }

    fn get_session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    async fn list_sessions(&self) -> Result<Vec<SessionRecord>> {
        let home = dirs::home_dir().ok_or_else(|| {
            RuntimeError::Protocol("Could not find home directory".to_string())
        })?;

        let encoded = encode_project_path(&self.project_dir);
        let sessions_dir = home.join(".claude").join("projects").join(&encoded);

        let mut read_dir = match tokio::fs::read_dir(&sessions_dir).await {
            Ok(rd) => rd,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::debug!("Sessions dir does not exist: {:?}", sessions_dir);
                return Ok(Vec::new());
            }
            Err(e) => {
                tracing::warn!("Failed to read sessions dir: {}", e);
                return Err(RuntimeError::Io(e));
            }
        };

        let mut sessions = Vec::new();

        while let Ok(Some(entry)) = read_dir.next_entry().await {
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }

            let session_id = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };

            let file = match tokio::fs::File::open(&path).await {
                Ok(f) => f,
                Err(e) => {
                    tracing::warn!("Failed to open session file {:?}: {}", path, e);
                    continue;
                }
            };

            let reader = tokio::io::BufReader::new(file);
            let mut lines = reader.lines();

            let first_line = match lines.next_line().await {
                Ok(Some(line)) => line,
                _ => continue,
            };

            let mut last_line = first_line.clone();
            while let Ok(Some(line)) = lines.next_line().await {
                last_line = line;
            }

            let mut first_prompt = None;
            let mut timestamp = None;
            let mut last_prompt = None;

            if let Ok(first) = serde_json::from_str::<serde_json::Value>(&first_line) {
                if first.get("type").and_then(|t| t.as_str()) == Some("queue-operation") {
                    timestamp = first.get("timestamp").and_then(|t| t.as_str()).map(|s| s.to_string());
                    first_prompt = first.get("content").and_then(|c| c.as_str()).map(|s| truncate_str(s, 100));
                }
            }

            if let Ok(last) = serde_json::from_str::<serde_json::Value>(&last_line) {
                if last.get("type").and_then(|t| t.as_str()) == Some("last-prompt") {
                    last_prompt = last.get("lastPrompt").and_then(|c| c.as_str()).map(|s| truncate_str(s, 100));
                }
            }

            let last_modified = tokio::fs::metadata(&path)
                .await
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    let datetime: chrono::DateTime<chrono::Utc> = t.into();
                    datetime.to_rfc3339()
                });

            sessions.push(SessionRecord {
                session_id,
                first_prompt,
                last_prompt,
                timestamp,
                last_modified,
            });
        }

        sessions.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
        tracing::info!("Found {} sessions", sessions.len());
        Ok(sessions)
    }
}
