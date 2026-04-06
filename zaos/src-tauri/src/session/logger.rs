use std::path::Path;

use crate::memory::models::SessionInsightsEditorial;
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

/// Write (or overwrite) `.memory/session-insights.md` with auto-metadata + editorial content.
/// Archives the previous snapshot if it contains meaningful data.
pub async fn write_session_insights(
    project_dir: &Path,
    session_id: &str,
    phase: &str,
    epic: &str,
    duration_secs: u64,
    tokens_input: u64,
    tokens_output: u64,
    agents_used: &[String],
    editorial: &SessionInsightsEditorial,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let memory_dir = project_dir.join(".memory");
    let insights_path = memory_dir.join("session-insights.md");
    let sessions_dir = memory_dir.join("sessions");
    tokio::fs::create_dir_all(&sessions_dir).await?;

    // Archive existing snapshot if it contains meaningful data
    if let Ok(existing) = tokio::fs::read_to_string(&insights_path).await {
        if is_meaningful_insights(&existing) {
            let now = chrono::Local::now();
            let sid_short = if session_id.len() > 8 { &session_id[..8] } else { session_id };
            let archive_name = format!("{}_{}_insights.md", now.format("%Y-%m-%d_%H-%M-%S"), sid_short);
            tokio::fs::write(sessions_dir.join(&archive_name), &existing).await?;
            tracing::info!("Archived previous session insights: {}", archive_name);
        }
    }

    // Build new snapshot
    let now = chrono::Local::now();
    let date_str = now.format("%Y-%m-%d").to_string();
    let agents_str = if agents_used.is_empty() {
        String::new()
    } else {
        agents_used.join(", ")
    };

    let mut content = String::new();
    content.push_str("# Session Insights\n\n");
    content.push_str(&format!("> Date: {}\n", date_str));
    content.push_str(&format!("> Epic: {}\n", if epic.is_empty() { "unknown" } else { epic }));
    content.push_str(&format!("> Phase: {}\n", if phase.is_empty() { "unknown" } else { phase }));
    content.push_str(&format!("> Session-ID: {}\n", session_id));
    content.push_str(&format!("> Duration: {}s\n", duration_secs));
    content.push_str(&format!("> Tokens: {} in / {} out\n", tokens_input, tokens_output));
    content.push_str(&format!("> Agents: {}\n", agents_str));
    let persona = suggested_persona_for_phase(phase);
    if !persona.is_empty() {
        content.push_str(&format!("> Suggested-Persona: {}\n", persona));
    }

    content.push_str("\n## Decisions prises\n\n");
    if editorial.decisions.is_empty() {
        content.push_str("- _aucune_\n");
    } else {
        for d in &editorial.decisions {
            content.push_str(&format!("- {}\n", d));
        }
    }

    content.push_str("\n## Ce qu'on a appris\n\n");
    if editorial.learnings.is_empty() {
        content.push_str("- _aucun apprentissage_\n");
    } else {
        for l in &editorial.learnings {
            content.push_str(&format!("- {}\n", l));
        }
    }

    content.push_str("\n## Risques et points ouverts\n\n");
    if editorial.risks.is_empty() {
        content.push_str("- _aucun risque identifie_\n");
    } else {
        for r in &editorial.risks {
            content.push_str(&format!("- {}\n", r));
        }
    }

    content.push_str("\n## Prochaines validations\n\n");
    if editorial.next_validations.is_empty() {
        content.push_str("- _aucune validation prevue_\n");
    } else {
        for v in &editorial.next_validations {
            content.push_str(&format!("- {}\n", v));
        }
    }

    tokio::fs::write(&insights_path, content).await?;
    tracing::info!("Session insights written to session-insights.md");
    Ok(())
}

/// Update only the editorial sections of an existing `session-insights.md`,
/// preserving the original metadata blockquote lines.
/// Returns an error if the file does not exist yet.
pub async fn update_editorial_only(
    project_dir: &Path,
    editorial: &SessionInsightsEditorial,
) -> Result<(), String> {
    let insights_path = project_dir.join(".memory").join("session-insights.md");

    let existing = tokio::fs::read_to_string(&insights_path)
        .await
        .map_err(|_| "session-insights.md does not exist yet".to_string())?;

    // Parse: keep H1 heading and blockquote metadata lines from the header only.
    // Stop collecting blockquotes once a section heading (## ) is encountered.
    let mut heading = String::new();
    let mut metadata_lines: Vec<String> = Vec::new();
    let mut in_header = true;

    for line in existing.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") && !trimmed.starts_with("## ") && heading.is_empty() {
            heading = line.to_string();
        } else if trimmed.starts_with("## ") {
            in_header = false;
        } else if in_header && trimmed.starts_with("> ") {
            metadata_lines.push(line.to_string());
        }
    }

    if heading.is_empty() {
        heading = "# Session Insights".to_string();
    }

    // Rebuild the file
    let mut content = String::new();
    content.push_str(&heading);
    content.push_str("\n\n");
    for meta in &metadata_lines {
        content.push_str(meta);
        content.push('\n');
    }

    content.push_str("\n## Decisions prises\n\n");
    if editorial.decisions.is_empty() {
        content.push_str("- _aucune_\n");
    } else {
        for d in &editorial.decisions {
            content.push_str(&format!("- {}\n", d));
        }
    }

    content.push_str("\n## Ce qu'on a appris\n\n");
    if editorial.learnings.is_empty() {
        content.push_str("- _aucun apprentissage_\n");
    } else {
        for l in &editorial.learnings {
            content.push_str(&format!("- {}\n", l));
        }
    }

    content.push_str("\n## Risques et points ouverts\n\n");
    if editorial.risks.is_empty() {
        content.push_str("- _aucun risque identifie_\n");
    } else {
        for r in &editorial.risks {
            content.push_str(&format!("- {}\n", r));
        }
    }

    content.push_str("\n## Prochaines validations\n\n");
    if editorial.next_validations.is_empty() {
        content.push_str("- _aucune validation prevue_\n");
    } else {
        for v in &editorial.next_validations {
            content.push_str(&format!("- {}\n", v));
        }
    }

    tokio::fs::write(&insights_path, content)
        .await
        .map_err(|e| format!("Failed to write session-insights.md: {}", e))?;

    tracing::info!("Session insights editorial updated (metadata preserved)");
    Ok(())
}

/// Map a pipeline phase to the most relevant persona.
/// Names must match the frontend `PHASE_PERSONA_MAP` in `personaMapping.ts`.
fn suggested_persona_for_phase(phase: &str) -> &'static str {
    match phase {
        "comprehension" | "specification" | "architecture" => "Product Architect",
        "implementation" => "Builder",
        "review" => "Reviewer",
        "test" => "Tester",
        "closure" => "Release Manager",
        _ => "",
    }
}

/// Check if an existing session-insights.md has meaningful content.
/// Returns false for empty files, template files, or files with only placeholder content.
fn is_meaningful_insights(content: &str) -> bool {
    // Must have at least one non-template metadata value
    let has_real_metadata = content.lines().any(|l| {
        let t = l.trim();
        if let Some(rest) = t.strip_prefix("> Date:") {
            let v = rest.trim();
            !v.is_empty() && v != "YYYY-MM-DD"
        } else {
            false
        }
    });

    // Must have at least one non-placeholder list item
    let has_real_item = content.lines().any(|l| {
        let t = l.trim();
        t.starts_with("- ") && !t.starts_with("- _") && t.len() > 2
    });

    has_real_metadata || has_real_item
}
