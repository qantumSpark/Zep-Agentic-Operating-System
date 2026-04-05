//! Product Contract data models.
//! Shared types used by the memory reader and exposed via Tauri commands.

use serde::{Deserialize, Serialize};

// ── product-brief.md ──

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProductBrief {
    pub vision: String,
    pub audience: String,
    pub rationale: String,
    pub constraints: Vec<String>,
    pub out_of_scope: Vec<String>,
    pub success_definition: String,
}

// ── experience-goals.md ──

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExperienceGoal {
    pub number: u32,
    pub quality: String,
    pub criterion: String,
    pub priority: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExperienceGoals {
    pub goals: Vec<ExperienceGoal>,
    pub ux_standards: Vec<String>,
    pub anti_patterns: Vec<String>,
}

// ── acceptance-checks.md ──

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AcceptanceCheck {
    pub number: u32,
    pub check: String,
    pub status: String,
    pub notes: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AcceptanceChecks {
    pub checks: Vec<AcceptanceCheck>,
    pub manual_validations: Vec<String>,
}

// ── release-readiness.md ──

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReleaseChecklistItem {
    pub number: u32,
    pub item: String,
    pub status: String,
    pub blocking: String,
    pub notes: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReleaseReadiness {
    pub overall_state_summary: String,
    pub checklist: Vec<ReleaseChecklistItem>,
    pub open_risks: Vec<String>,
}

// ── session-insights.md ──

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionInsights {
    pub date: String,
    pub epic: String,
    pub phase: String,
    pub session_id: String,
    pub duration_secs: u64,
    pub tokens_input: u64,
    pub tokens_output: u64,
    pub agents_used: Vec<String>,
    pub decisions: Vec<String>,
    pub learnings: Vec<String>,
    pub risks: Vec<String>,
    pub next_validations: Vec<String>,
}

// ── Persona (.zaos/personas/*.md) ──

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Persona {
    pub name: String,
    pub role: String,
    pub maps_to: String,
    pub description: String,
    pub responsibilities: Vec<String>,
    pub when_active: Vec<String>,
}

// ── Session insights editorial payload ──

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SessionInsightsEditorial {
    pub decisions: Vec<String>,
    pub learnings: Vec<String>,
    pub risks: Vec<String>,
    pub next_validations: Vec<String>,
}

// ── Aggregate ──

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProductContract {
    pub brief: Option<ProductBrief>,
    pub experience_goals: Option<ExperienceGoals>,
    pub acceptance_checks: Option<AcceptanceChecks>,
    pub release_readiness: Option<ReleaseReadiness>,
    pub session_insights: Option<SessionInsights>,
}
