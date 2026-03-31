use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SessionLogData {
    pub session_id: String,
    pub phase: String,
    pub tokens_input: u64,
    pub tokens_output: u64,
    pub duration_secs: u64,
    pub agent_timings: Vec<AgentTiming>,
}

#[derive(Debug, Deserialize)]
pub struct AgentTiming {
    pub name: String,
    pub duration_ms: u64,
}

pub async fn write_session_log(
    project_dir: &Path,
    data: &SessionLogData,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let sessions_dir = project_dir.join(".memory").join("sessions");
    tokio::fs::create_dir_all(&sessions_dir).await?;

    let now = chrono::Local::now();
    let filename = format!("{}.md", now.format("%Y-%m-%d_%H-%M-%S"));

    let mut content = String::new();
    content.push_str(&format!(
        "# Session {}\n\n",
        now.format("%Y-%m-%d %H:%M")
    ));
    content.push_str(&format!("> ID: {}\n", data.session_id));
    content.push_str(&format!("> Phase: {}\n", data.phase));
    content.push_str(&format!("> Duration: {}s\n\n", data.duration_secs));
    content.push_str("## Tokens\n\n");
    content.push_str(&format!("- Input: {}\n", data.tokens_input));
    content.push_str(&format!("- Output: {}\n\n", data.tokens_output));

    if !data.agent_timings.is_empty() {
        content.push_str("## Agents\n\n");
        for agent in &data.agent_timings {
            content.push_str(&format!("- {}: {}ms\n", agent.name, agent.duration_ms));
        }
    }

    tokio::fs::write(sessions_dir.join(&filename), content).await?;
    tracing::info!("Session log written: {}", filename);
    Ok(())
}
