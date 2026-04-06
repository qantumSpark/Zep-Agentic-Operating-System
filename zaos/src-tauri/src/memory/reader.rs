use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use tokio::fs;

use super::models::{
    AcceptanceCheck, AcceptanceChecks, ExperienceGoal, ExperienceGoals,
    Persona, ProductBrief, ProductContract, ReleaseChecklistItem, ReleaseReadiness,
    SessionInsights,
};

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("File not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, MemoryError>;

/// A structured warning emitted when a markdown field is missing or malformed
/// during parsing.  The parser still produces a best-effort value; warnings
/// are collected alongside it so that callers (e.g. memory-health checks) can
/// surface them without aborting.
#[derive(Debug, Clone, Serialize)]
pub struct ParseWarning {
    pub field: String,
    pub message: String,
}

/// Wraps a parsed value together with any warnings that were generated during
/// parsing.  The `value` is always usable (defaults are substituted for missing
/// fields); `warnings` lists every such substitution.
#[derive(Debug, Clone)]
pub struct ParseResult<T> {
    pub value: T,
    pub warnings: Vec<ParseWarning>,
}

/// A single entry parsed from INDEX.md
///
/// Represents a line like: `- [Title](filename.md) — Description text`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryIndexEntry {
    /// Display title extracted from `[Title]`
    pub title: String,
    /// Relative filename extracted from `(filename.md)`
    pub filename: String,
    /// Description text after the em-dash separator
    pub description: String,
}

/// A named section in the INDEX.md file (e.g. "Reference", "Decisions")
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryIndexSection {
    /// Section heading text (from `## Heading`)
    pub heading: String,
    /// Entries listed under this section
    pub entries: Vec<MemoryIndexEntry>,
}

/// Parsed memory index from `.memory/INDEX.md`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryIndex {
    /// All sections found in the index file
    pub sections: Vec<MemoryIndexSection>,
}

impl MemoryIndex {
    /// Returns a flat iterator over all entries across all sections.
    pub fn all_entries(&self) -> impl Iterator<Item = &MemoryIndexEntry> {
        self.sections.iter().flat_map(|s| s.entries.iter())
    }

    /// Find an entry by filename across all sections.
    #[allow(dead_code)]
    pub fn find_by_filename(&self, filename: &str) -> Option<&MemoryIndexEntry> {
        self.all_entries().find(|e| e.filename == filename)
    }
}

/// A single row from the Milestones table in state.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneEntry {
    pub number: u32,
    pub name: String,
    pub status: String,
    pub epics: String,
}

/// Parsed content of .memory/state.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryState {
    pub milestones: Vec<MilestoneEntry>,
    pub active_epic: String,
    pub blocages: String,
}

/// A single task row from the current epic's Tasks table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpicTask {
    pub number: u32,
    pub name: String,
    pub files: Vec<String>,
    pub status: String,
    pub notes: String,
}

/// Parsed representation of `.memory/current-epic.md`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentEpic {
    pub name: String,
    pub milestone: String,
    pub status: String,
    pub objective: String,
    pub tasks: Vec<EpicTask>,
}

/// Aggregated snapshot of all memory files.
#[derive(Debug, Clone, Serialize)]
pub struct MemoryStateResponse {
    pub index: MemoryIndex,
    pub state: Option<MemoryState>,
    pub current_epic: Option<CurrentEpic>,
}

/// Health status of a single memory file.
#[derive(Debug, Clone, Serialize)]
pub struct FileHealthEntry {
    /// File name, e.g. "current-epic.md", "state.md", "INDEX.md"
    pub name: String,
    /// Whether the file exists on disk
    pub present: bool,
    /// Whether the file parses without errors (true if not applicable or not present)
    pub parseable: bool,
    /// Warnings emitted during parsing
    pub warnings: Vec<ParseWarning>,
}

/// Aggregated health report for the `.memory/` directory.
#[derive(Debug, Clone, Serialize)]
pub struct MemoryHealthReport {
    /// Per-file health entries
    pub files: Vec<FileHealthEntry>,
    /// Whether the epic name in state.json matches the one in current-epic.md
    pub epic_name_match: bool,
    /// Epic name as stored in state.json (workflow engine)
    pub epic_name_workflow: String,
    /// Epic name as parsed from current-epic.md
    pub epic_name_memory: String,
    /// Global warnings (e.g. cross-file inconsistencies)
    pub warnings: Vec<String>,
}

/// MemoryReader reads files from .memory/ directory
pub struct MemoryReader {
    memory_dir: PathBuf,
}

impl MemoryReader {
    pub fn new(project_dir: PathBuf) -> Self {
        let memory_dir = project_dir.join(".memory");
        MemoryReader { memory_dir }
    }

    /// Read and parse `.memory/INDEX.md` into a structured `MemoryIndex`.
    ///
    /// The expected format is markdown with `## Section` headings and list items:
    /// ```text
    /// ## Reference
    /// - [Title](filename.md) — Description text
    /// ```
    ///
    /// Returns an empty index if the file does not exist.
    /// Malformed lines are skipped with a debug-level log.
    pub async fn read_index(&self) -> Result<MemoryIndex> {
        let index_path = self.memory_dir.join("INDEX.md");
        tracing::debug!("Reading memory index from: {:?}", index_path);

        let content = match fs::read_to_string(&index_path).await {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::debug!("INDEX.md not found, returning empty index");
                return Ok(MemoryIndex {
                    sections: Vec::new(),
                });
            }
            Err(e) => return Err(MemoryError::Io(e)),
        };

        let index = parse_index_md(&content);
        tracing::debug!(
            "Parsed index: {} sections, {} total entries",
            index.sections.len(),
            index.all_entries().count()
        );
        Ok(index)
    }

    /// Read and parse .memory/state.md into structured MemoryState.
    ///
    /// Extracts the milestones table, active epic, and blocages sections
    /// from the markdown file using line-by-line parsing.
    pub async fn read_state(&self) -> Result<MemoryState> {
        let state_path = self.memory_dir.join("state.md");
        tracing::debug!("Reading state from: {:?}", state_path);

        let content = match fs::read_to_string(&state_path).await {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(MemoryError::NotFound(state_path.display().to_string()));
            }
            Err(e) => return Err(MemoryError::Io(e)),
        };

        let result = parse_state_md(&content);
        for w in &result.warnings {
            tracing::debug!("state.md parse warning [{}]: {}", w.field, w.message);
        }
        tracing::debug!(
            "Parsed state: {} milestones, active_epic={:?}",
            result.value.milestones.len(),
            result.value.active_epic
        );
        Ok(result.value)
    }

    /// Read and parse `.memory/current-epic.md` into a structured `CurrentEpic`.
    ///
    /// Extracts the epic name, milestone, status, objective, and tasks table
    /// from the markdown file using line-by-line parsing.
    pub async fn read_current_epic(&self) -> Result<Option<CurrentEpic>> {
        let epic_path = self.memory_dir.join("current-epic.md");
        tracing::debug!("Reading current epic from: {:?}", epic_path);

        let content = match fs::read_to_string(&epic_path).await {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::debug!("current-epic.md not found, returning None");
                return Ok(None);
            }
            Err(e) => return Err(MemoryError::Io(e)),
        };

        let result = parse_current_epic_md(&content);
        for w in &result.warnings {
            tracing::debug!("current-epic.md parse warning [{}]: {}", w.field, w.message);
        }
        tracing::debug!(
            "Parsed current epic: name={:?}, {} tasks",
            result.value.name,
            result.value.tasks.len()
        );
        Ok(Some(result.value))
    }

    /// Read all memory files concurrently and return an aggregated snapshot.
    ///
    /// - Index error → propagated (hard failure).
    /// - State / epic errors → silently mapped to `None`.
    pub async fn read_all(&self) -> std::result::Result<MemoryStateResponse, String> {
        let (index_res, state_res, epic_res) = tokio::join!(
            self.read_index(),
            self.read_state(),
            self.read_current_epic(),
        );

        let index = index_res.map_err(|e| format!("Failed to read memory index: {}", e))?;

        let state = match state_res {
            Ok(s) => Some(s),
            Err(e) => {
                tracing::debug!("Memory state not available: {}", e);
                None
            }
        };

        let current_epic = match epic_res {
            Ok(epic) => epic,
            Err(e) => {
                tracing::debug!("Current epic not available: {}", e);
                None
            }
        };

        Ok(MemoryStateResponse {
            index,
            state,
            current_epic,
        })
    }

    /// Read and parse `.memory/product-brief.md`.
    pub async fn read_product_brief(&self) -> Result<Option<ProductBrief>> {
        let path = self.memory_dir.join("product-brief.md");
        match fs::read_to_string(&path).await {
            Ok(content) => Ok(Some(parse_product_brief_md(&content))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(MemoryError::Io(e)),
        }
    }

    /// Read and parse `.memory/experience-goals.md`.
    pub async fn read_experience_goals(&self) -> Result<Option<ExperienceGoals>> {
        let path = self.memory_dir.join("experience-goals.md");
        match fs::read_to_string(&path).await {
            Ok(content) => Ok(Some(parse_experience_goals_md(&content))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(MemoryError::Io(e)),
        }
    }

    /// Read and parse `.memory/acceptance-checks.md`.
    pub async fn read_acceptance_checks(&self) -> Result<Option<AcceptanceChecks>> {
        let path = self.memory_dir.join("acceptance-checks.md");
        match fs::read_to_string(&path).await {
            Ok(content) => Ok(Some(parse_acceptance_checks_md(&content))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(MemoryError::Io(e)),
        }
    }

    /// Read and parse `.memory/release-readiness.md`.
    pub async fn read_release_readiness(&self) -> Result<Option<ReleaseReadiness>> {
        let path = self.memory_dir.join("release-readiness.md");
        match fs::read_to_string(&path).await {
            Ok(content) => Ok(Some(parse_release_readiness_md(&content))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(MemoryError::Io(e)),
        }
    }

    /// Read and parse `.memory/session-insights.md`.
    pub async fn read_session_insights(&self) -> Result<Option<SessionInsights>> {
        let path = self.memory_dir.join("session-insights.md");
        match fs::read_to_string(&path).await {
            Ok(content) => Ok(Some(parse_session_insights_md(&content))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(MemoryError::Io(e)),
        }
    }

    /// Read all product contract artifacts concurrently.
    /// Missing files produce `None`, errors are logged and mapped to `None`.
    pub async fn read_product_contract(&self) -> ProductContract {
        let (brief, goals, checks, readiness, insights) = tokio::join!(
            self.read_product_brief(),
            self.read_experience_goals(),
            self.read_acceptance_checks(),
            self.read_release_readiness(),
            self.read_session_insights(),
        );

        ProductContract {
            brief: brief.unwrap_or_else(|e| {
                tracing::warn!("Failed to read product-brief: {}", e);
                None
            }),
            experience_goals: goals.unwrap_or_else(|e| {
                tracing::warn!("Failed to read experience-goals: {}", e);
                None
            }),
            acceptance_checks: checks.unwrap_or_else(|e| {
                tracing::warn!("Failed to read acceptance-checks: {}", e);
                None
            }),
            release_readiness: readiness.unwrap_or_else(|e| {
                tracing::warn!("Failed to read release-readiness: {}", e);
                None
            }),
            session_insights: insights.unwrap_or_else(|e| {
                tracing::warn!("Failed to read session-insights: {}", e);
                None
            }),
        }
    }

    /// Read all persona files from `.zaos/personas/`.
    pub async fn read_personas(&self) -> Vec<Persona> {
        // self.memory_dir points to .memory; parent gives us project_dir
        let personas_dir = self.memory_dir.parent()
            .unwrap_or(&self.memory_dir)
            .join(".zaos")
            .join("personas");

        let mut personas = Vec::new();
        let mut entries = match fs::read_dir(&personas_dir).await {
            Ok(entries) => entries,
            Err(_) => return personas,
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                if let Ok(content) = fs::read_to_string(&path).await {
                    personas.push(parse_persona_md(&content));
                }
            }
        }

        personas.sort_by(|a, b| a.name.cmp(&b.name));
        personas
    }

    /// Run a lightweight health check on the `.memory/` directory.
    ///
    /// Checks file presence, parsability, and cross-file consistency
    /// (epic name in `state.json` vs `current-epic.md`).
    pub async fn check_health(&self, workflow_epic: &str) -> MemoryHealthReport {
        let mut files = Vec::new();
        let mut global_warnings: Vec<String> = Vec::new();
        let mut epic_name_memory = String::new();

        // --- INDEX.md: presence only (no structured parse) ---
        let index_path = self.memory_dir.join("INDEX.md");
        let index_present = fs::metadata(&index_path).await.is_ok();
        files.push(FileHealthEntry {
            name: "INDEX.md".to_string(),
            present: index_present,
            parseable: true, // no structured parse for INDEX
            warnings: Vec::new(),
        });
        if !index_present {
            global_warnings.push("INDEX.md is missing".to_string());
        }

        // --- state.md: presence + parse ---
        let state_path = self.memory_dir.join("state.md");
        match fs::read_to_string(&state_path).await {
            Ok(content) => {
                let result = parse_state_md(&content);
                let parseable = result.warnings.is_empty();
                files.push(FileHealthEntry {
                    name: "state.md".to_string(),
                    present: true,
                    parseable,
                    warnings: result.warnings,
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                files.push(FileHealthEntry {
                    name: "state.md".to_string(),
                    present: false,
                    parseable: true, // N/A — parsability is irrelevant when file is absent
                    warnings: Vec::new(),
                });
                global_warnings.push("state.md is missing".to_string());
            }
            Err(e) => {
                files.push(FileHealthEntry {
                    name: "state.md".to_string(),
                    present: true,
                    parseable: false,
                    warnings: vec![ParseWarning {
                        field: "io".to_string(),
                        message: format!("Failed to read file: {}", e),
                    }],
                });
            }
        }

        // --- current-epic.md: presence + parse + extract epic name ---
        let epic_path = self.memory_dir.join("current-epic.md");
        match fs::read_to_string(&epic_path).await {
            Ok(content) => {
                let result = parse_current_epic_md(&content);
                let parseable = result.warnings.is_empty();
                epic_name_memory = result.value.name.clone();
                files.push(FileHealthEntry {
                    name: "current-epic.md".to_string(),
                    present: true,
                    parseable,
                    warnings: result.warnings,
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                files.push(FileHealthEntry {
                    name: "current-epic.md".to_string(),
                    present: false,
                    parseable: true, // N/A — parsability is irrelevant when file is absent
                    warnings: Vec::new(),
                });
                // Not necessarily a warning — no epic may be active
            }
            Err(e) => {
                files.push(FileHealthEntry {
                    name: "current-epic.md".to_string(),
                    present: true,
                    parseable: false,
                    warnings: vec![ParseWarning {
                        field: "io".to_string(),
                        message: format!("Failed to read file: {}", e),
                    }],
                });
            }
        }

        // --- Cross-file consistency: epic name ---
        let epic_name_workflow = workflow_epic.to_string();
        let epic_name_match = if epic_name_workflow.is_empty() && epic_name_memory.is_empty() {
            true // both empty — consistent
        } else {
            epic_name_workflow == epic_name_memory
        };

        if !epic_name_match
            && !epic_name_workflow.is_empty()
            && !epic_name_memory.is_empty()
        {
            global_warnings.push(format!(
                "Epic name mismatch: state.json has '{}', current-epic.md has '{}'",
                epic_name_workflow, epic_name_memory
            ));
        }

        MemoryHealthReport {
            files,
            epic_name_match,
            epic_name_workflow,
            epic_name_memory,
            warnings: global_warnings,
        }
    }
}

/// Parse the full content of `state.md` into a `MemoryState`.
///
/// Uses a simple state-machine approach: track the current `## Heading` and
/// collect lines accordingly.  Missing sections gracefully produce empty/default
/// values.
fn parse_state_md(content: &str) -> ParseResult<MemoryState> {
    let mut milestones: Vec<MilestoneEntry> = Vec::new();
    let mut active_epic = String::new();
    let mut blocages = String::new();
    let mut warnings: Vec<ParseWarning> = Vec::new();

    enum Section {
        None,
        Milestones,
        ActiveEpic,
        Blocages,
        Other,
    }

    let mut section = Section::None;
    let mut found_milestones_section = false;
    let mut found_active_epic_section = false;
    let mut milestone_row_index: usize = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Detect section headings (## ...)
        if trimmed.starts_with("## ") {
            let heading = trimmed[3..].trim().to_lowercase();
            section = if heading == "milestones" {
                found_milestones_section = true;
                Section::Milestones
            } else if heading.starts_with("epic active") || heading.starts_with("epic actif") {
                found_active_epic_section = true;
                Section::ActiveEpic
            } else if heading.starts_with("blocage") {
                Section::Blocages
            } else {
                Section::Other
            };
            continue;
        }

        match section {
            Section::Milestones => {
                // Only consider lines that look like table rows (start with '|')
                if trimmed.starts_with('|') {
                    // Skip header and separator rows silently
                    let is_separator = trimmed.trim_matches('|').trim().chars().all(|c| c == '-' || c == '|' || c == ' ');
                    let is_header = trimmed.contains("# |") || trimmed.contains("| # |");
                    if !is_separator && !is_header {
                        milestone_row_index += 1;
                        match parse_milestone_row(trimmed) {
                            Some(entry) => milestones.push(entry),
                            None => warnings.push(ParseWarning {
                                field: "milestones".to_string(),
                                message: format!("milestone row {} parse failed", milestone_row_index),
                            }),
                        }
                    }
                }
            }
            Section::ActiveEpic => {
                if !trimmed.is_empty() {
                    if !active_epic.is_empty() {
                        active_epic.push('\n');
                    }
                    active_epic.push_str(trimmed);
                }
            }
            Section::Blocages => {
                if !trimmed.is_empty() {
                    if !blocages.is_empty() {
                        blocages.push('\n');
                    }
                    blocages.push_str(trimmed);
                }
            }
            _ => {}
        }
    }

    if !found_milestones_section {
        warnings.push(ParseWarning {
            field: "milestones".to_string(),
            message: "milestones section missing".to_string(),
        });
    }

    if !found_active_epic_section {
        warnings.push(ParseWarning {
            field: "active_epic".to_string(),
            message: "active_epic section missing".to_string(),
        });
    }

    ParseResult {
        value: MemoryState {
            milestones,
            active_epic,
            blocages,
        },
        warnings,
    }
}

/// Parse a markdown table row into a numbered row id and remaining cell strings.
///
/// Returns `None` for non-table lines, separator rows (`|---|---|`), and header
/// rows where the first cell is `#` or otherwise non-numeric.
///
/// `min_cells` is the minimum number of *real* cells required (excluding the
/// leading/trailing empty strings produced by splitting on `|`).
fn parse_md_table_row(line: &str, min_cells: usize) -> Option<(u32, Vec<String>)> {
    if !line.starts_with('|') {
        return None;
    }

    let raw: Vec<&str> = line.split('|').map(|c| c.trim()).collect();
    // Remove leading and trailing empty strings from split('|') on "|...|" format
    let cells: Vec<&str> = if raw.len() >= 2 && raw[0].is_empty() && raw[raw.len() - 1].is_empty() {
        raw[1..raw.len() - 1].to_vec()
    } else if raw.len() >= 1 && raw[0].is_empty() {
        raw[1..].to_vec()
    } else {
        raw
    };

    if cells.len() < min_cells {
        return None;
    }

    let num_str = cells[0];

    // Skip separator rows (all dashes) and header rows (first cell is "#")
    if num_str.chars().all(|c| c == '-') || num_str == "#" {
        return None;
    }

    let number: u32 = match num_str.parse() {
        Ok(n) => n,
        Err(_) => return None,
    };

    let rest = cells[1..].iter().map(|c| c.to_string()).collect();
    Some((number, rest))
}

/// Try to parse a single markdown table row into a `MilestoneEntry`.
///
/// Expected format: `| 1 | Milestone name | TERMINE | epic1, epic2 |`
fn parse_milestone_row(line: &str) -> Option<MilestoneEntry> {
    let (number, cells) = parse_md_table_row(line, 4)?;

    Some(MilestoneEntry {
        number,
        name: cells[0].clone(),
        status: cells[1].clone(),
        epics: cells[2].clone(),
    })
}

/// Parse the full content of `current-epic.md` into a `CurrentEpic`.
///
/// Extracts metadata from the blockquote header (`> Milestone : ...`, `> Statut : ...`),
/// the epic name from the `# Epic active : ...` heading, the objective paragraph, and
/// the tasks table.
fn parse_current_epic_md(content: &str) -> ParseResult<CurrentEpic> {
    let mut name = String::new();
    let mut milestone = String::new();
    let mut status = String::new();
    let mut objective = String::new();
    let mut tasks: Vec<EpicTask> = Vec::new();
    let mut warnings: Vec<ParseWarning> = Vec::new();

    enum Section {
        Header,
        Objective,
        Tasks,
        Other,
    }

    let mut section = Section::Header;
    let mut found_milestone_blockquote = false;
    let mut found_status_blockquote = false;
    let mut task_row_index: usize = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Top-level heading: # Epic active : <name>
        if trimmed.starts_with("# ") && !trimmed.starts_with("## ") {
            if let Some(after_colon) = trimmed.splitn(2, ':').nth(1) {
                name = after_colon.trim().to_string();
            } else {
                name = trimmed[2..].trim().to_string();
            }
            continue;
        }

        // Blockquote metadata
        if trimmed.starts_with("> ") {
            let meta = &trimmed[2..];
            if let Some(val) = meta.strip_prefix("Milestone :") {
                milestone = val.trim().to_string();
                found_milestone_blockquote = true;
            } else if let Some(val) = meta.strip_prefix("Statut :") {
                status = val.trim().to_string();
                found_status_blockquote = true;
            }
            continue;
        }

        // Section headings
        if trimmed.starts_with("## ") {
            let heading = trimmed[3..].trim().to_lowercase();
            section = if heading == "objectif" {
                Section::Objective
            } else if heading == "tasks" {
                Section::Tasks
            } else {
                Section::Other
            };
            continue;
        }

        match section {
            Section::Objective => {
                if !trimmed.is_empty() {
                    if !objective.is_empty() {
                        objective.push('\n');
                    }
                    objective.push_str(trimmed);
                }
            }
            Section::Tasks => {
                // Only consider lines that look like table rows (start with '|')
                if trimmed.starts_with('|') {
                    let is_separator = trimmed.trim_matches('|').trim().chars().all(|c| c == '-' || c == '|' || c == ' ');
                    let is_header = trimmed.contains("# |") || trimmed.contains("| # |");
                    if !is_separator && !is_header {
                        task_row_index += 1;
                        match parse_epic_task_row(trimmed) {
                            Some(task) => tasks.push(task),
                            None => warnings.push(ParseWarning {
                                field: "tasks".to_string(),
                                message: format!("task row {} parse failed", task_row_index),
                            }),
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if name.is_empty() {
        warnings.push(ParseWarning {
            field: "name".to_string(),
            message: "epic name missing".to_string(),
        });
    }

    if !found_milestone_blockquote {
        warnings.push(ParseWarning {
            field: "milestone".to_string(),
            message: "milestone blockquote missing".to_string(),
        });
    }

    if !found_status_blockquote {
        warnings.push(ParseWarning {
            field: "status".to_string(),
            message: "status blockquote missing".to_string(),
        });
    }

    ParseResult {
        value: CurrentEpic {
            name,
            milestone,
            status,
            objective,
            tasks,
        },
        warnings,
    }
}

/// Try to parse a single markdown table row into an `EpicTask`.
///
/// Expected format: `| 1 | Task name | `file.rs` | DONE | notes |`
fn parse_epic_task_row(line: &str) -> Option<EpicTask> {
    let (number, cells) = parse_md_table_row(line, 5)?;

    let files_str = &cells[1];
    let notes = if cells.len() > 3 { &cells[3] } else { "" };

    // Parse files: split by comma, strip backticks
    let files: Vec<String> = files_str
        .split(',')
        .map(|f| f.trim().trim_matches('`').to_string())
        .filter(|f| !f.is_empty())
        .collect();

    Some(EpicTask {
        number,
        name: cells[0].clone(),
        files,
        status: cells[2].clone(),
        notes: notes.to_string(),
    })
}

/// Parse a single markdown list entry line into a `MemoryIndexEntry`.
///
/// Expected format: `- [Title](filename.md) — Description`
/// Accepts both em-dash (U+2014) and double-hyphen (--) as separators.
/// Returns `None` if the line doesn't match the expected format.
fn parse_index_entry(line: &str) -> Option<MemoryIndexEntry> {
    let trimmed = line.trim();

    // Must start with a list marker
    let after_marker = trimmed.strip_prefix("- ")?;

    // Extract [Title]
    let open_bracket = after_marker.find('[')?;
    let close_bracket = after_marker.find(']')?;
    if close_bracket <= open_bracket + 1 {
        return None; // Empty title like `[]`
    }
    let title = after_marker[open_bracket + 1..close_bracket].to_string();

    // Extract (filename.md) — must immediately follow the `]`
    let rest = &after_marker[close_bracket + 1..];
    let open_paren = rest.find('(')?;
    let close_paren = rest.find(')')?;
    if open_paren != 0 || close_paren <= 1 {
        return None;
    }
    let filename = rest[open_paren + 1..close_paren].to_string();

    // Extract description after separator (em-dash or double-hyphen)
    let after_link = &rest[close_paren + 1..];
    let description = if let Some(pos) = after_link.find('\u{2014}') {
        // Em-dash "\u{2014}"
        after_link[pos + '\u{2014}'.len_utf8()..].trim().to_string()
    } else if let Some(pos) = after_link.find("--") {
        // Double-hyphen fallback
        after_link[pos + 2..].trim().to_string()
    } else {
        // No separator — use whatever remains (trimmed), may be empty
        after_link.trim().to_string()
    };

    Some(MemoryIndexEntry {
        title,
        filename,
        description,
    })
}

/// Parse the full contents of INDEX.md into a `MemoryIndex`.
///
/// Groups entries by `## Section` headings. Entries appearing before any
/// section heading are placed in an "Uncategorized" section.
fn parse_index_md(content: &str) -> MemoryIndex {
    let mut sections: Vec<MemoryIndexSection> = Vec::new();
    let mut current_heading: Option<String> = None;
    let mut current_entries: Vec<MemoryIndexEntry> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Detect section headings (## Heading)
        if let Some(heading_text) = trimmed.strip_prefix("## ") {
            // Flush the previous section
            if let Some(heading) = current_heading.take() {
                sections.push(MemoryIndexSection {
                    heading,
                    entries: std::mem::take(&mut current_entries),
                });
            } else if !current_entries.is_empty() {
                // Entries that appeared before any heading
                sections.push(MemoryIndexSection {
                    heading: "Uncategorized".to_string(),
                    entries: std::mem::take(&mut current_entries),
                });
            }
            current_heading = Some(heading_text.trim().to_string());
            continue;
        }

        // Try to parse as a list entry
        if trimmed.starts_with("- [") {
            match parse_index_entry(trimmed) {
                Some(entry) => current_entries.push(entry),
                None => {
                    tracing::debug!("Skipping malformed index entry: {}", trimmed);
                }
            }
        }
        // All other lines (comments, blank lines, blockquotes) are ignored
    }

    // Flush the final section
    if let Some(heading) = current_heading {
        sections.push(MemoryIndexSection {
            heading,
            entries: current_entries,
        });
    } else if !current_entries.is_empty() {
        sections.push(MemoryIndexSection {
            heading: "Uncategorized".to_string(),
            entries: current_entries,
        });
    }

    MemoryIndex { sections }
}

// ── Helpers for product contract parsers ──

/// Append non-empty text to a string, adding a newline separator if needed.
fn append_text(target: &mut String, text: &str) {
    if !target.is_empty() {
        target.push('\n');
    }
    target.push_str(text);
}

/// Parse a markdown list item, skipping placeholder lines like `_italic placeholder_`.
fn parse_list_item(line: &str) -> Option<String> {
    let text = line.strip_prefix("- ")?.trim();
    if text.is_empty() {
        return None;
    }
    // Skip italic placeholders: _some text_
    if text.starts_with('_') && text.ends_with('_') && text.len() > 2 {
        return None;
    }
    Some(text.to_string())
}

// ── Product contract parsers ──

/// Parse `product-brief.md` into a `ProductBrief`.
/// Sections: Vision, Pour qui, Pourquoi, Contraintes, Hors-scope, Definition de succes.
fn parse_product_brief_md(content: &str) -> ProductBrief {
    let mut vision = String::new();
    let mut audience = String::new();
    let mut rationale = String::new();
    let mut constraints: Vec<String> = Vec::new();
    let mut out_of_scope: Vec<String> = Vec::new();
    let mut success_definition = String::new();

    enum Section { None, Vision, Audience, Rationale, Constraints, OutOfScope, SuccessDef }
    let mut section = Section::None;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("## ") {
            let heading = trimmed[3..].trim().to_lowercase();
            section = match heading.as_str() {
                "vision" => Section::Vision,
                "pour qui" => Section::Audience,
                "pourquoi" => Section::Rationale,
                "contraintes" => Section::Constraints,
                "hors-scope" => Section::OutOfScope,
                "definition de succes" => Section::SuccessDef,
                _ => Section::None,
            };
            continue;
        }

        if trimmed.is_empty() || trimmed.starts_with("# ") {
            continue;
        }

        match section {
            Section::Vision => append_text(&mut vision, trimmed),
            Section::Audience => append_text(&mut audience, trimmed),
            Section::Rationale => append_text(&mut rationale, trimmed),
            Section::Constraints => {
                if let Some(item) = parse_list_item(trimmed) {
                    constraints.push(item);
                }
            }
            Section::OutOfScope => {
                if let Some(item) = parse_list_item(trimmed) {
                    out_of_scope.push(item);
                }
            }
            Section::SuccessDef => append_text(&mut success_definition, trimmed),
            Section::None => {}
        }
    }

    ProductBrief { vision, audience, rationale, constraints, out_of_scope, success_definition }
}

/// Parse `experience-goals.md` into `ExperienceGoals`.
/// Sections: Qualites cibles (table), Standards UX (list), Anti-patterns (list).
fn parse_experience_goals_md(content: &str) -> ExperienceGoals {
    let mut goals: Vec<ExperienceGoal> = Vec::new();
    let mut ux_standards: Vec<String> = Vec::new();
    let mut anti_patterns: Vec<String> = Vec::new();

    enum Section { None, Goals, Standards, AntiPatterns }
    let mut section = Section::None;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("## ") {
            let heading = trimmed[3..].trim().to_lowercase();
            section = if heading == "qualites cibles" {
                Section::Goals
            } else if heading == "standards ux" {
                Section::Standards
            } else if heading.starts_with("anti-pattern") {
                Section::AntiPatterns
            } else {
                Section::None
            };
            continue;
        }

        if trimmed.is_empty() || trimmed.starts_with("# ") { continue; }

        match section {
            Section::Goals => {
                // Table: | # | Qualite | Critere | Priorite |
                if let Some((number, cells)) = parse_md_table_row(trimmed, 4) {
                    goals.push(ExperienceGoal {
                        number,
                        quality: cells[0].clone(),
                        criterion: cells[1].clone(),
                        priority: cells[2].clone(),
                    });
                }
            }
            Section::Standards => {
                if let Some(item) = parse_list_item(trimmed) {
                    ux_standards.push(item);
                }
            }
            Section::AntiPatterns => {
                if let Some(item) = parse_list_item(trimmed) {
                    anti_patterns.push(item);
                }
            }
            Section::None => {}
        }
    }

    ExperienceGoals { goals, ux_standards, anti_patterns }
}

/// Parse `acceptance-checks.md` into `AcceptanceChecks`.
/// Sections: Criteres (table), Validations manuelles (list).
fn parse_acceptance_checks_md(content: &str) -> AcceptanceChecks {
    let mut checks: Vec<AcceptanceCheck> = Vec::new();
    let mut manual_validations: Vec<String> = Vec::new();

    enum Section { None, Checks, Manual }
    let mut section = Section::None;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("## ") {
            let heading = trimmed[3..].trim().to_lowercase();
            section = if heading.starts_with("critere") || heading == "criteres" {
                Section::Checks
            } else if heading.starts_with("validations manuelles") {
                Section::Manual
            } else {
                Section::None
            };
            continue;
        }

        if trimmed.is_empty() || trimmed.starts_with("# ") { continue; }

        match section {
            Section::Checks => {
                // Table: | # | Check | Statut | Notes |
                if let Some((number, cells)) = parse_md_table_row(trimmed, 4) {
                    checks.push(AcceptanceCheck {
                        number,
                        check: cells[0].clone(),
                        status: cells[1].clone(),
                        notes: cells[2].clone(),
                    });
                }
            }
            Section::Manual => {
                if let Some(item) = parse_list_item(trimmed) {
                    manual_validations.push(item);
                }
            }
            Section::None => {}
        }
    }

    AcceptanceChecks { checks, manual_validations }
}

/// Parse `release-readiness.md` into `ReleaseReadiness`.
/// Sections: Etat general (text), Checklist (table), Risques ouverts (list).
fn parse_release_readiness_md(content: &str) -> ReleaseReadiness {
    let mut overall_state_summary = String::new();
    let mut checklist: Vec<ReleaseChecklistItem> = Vec::new();
    let mut open_risks: Vec<String> = Vec::new();

    enum Section { None, Overall, Checklist, Risks }
    let mut section = Section::None;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("## ") {
            let heading = trimmed[3..].trim().to_lowercase();
            section = if heading.starts_with("etat general") || heading.starts_with("état général") {
                Section::Overall
            } else if heading == "checklist" {
                Section::Checklist
            } else if heading.starts_with("risques ouvert") {
                Section::Risks
            } else {
                Section::None
            };
            continue;
        }

        if trimmed.is_empty() || trimmed.starts_with("# ") { continue; }

        match section {
            Section::Overall => {
                if !trimmed.starts_with('_') || !trimmed.ends_with('_') {
                    append_text(&mut overall_state_summary, trimmed);
                }
            }
            Section::Checklist => {
                // Table: | # | Item | Statut | Bloquant | Notes |
                if let Some((number, cells)) = parse_md_table_row(trimmed, 5) {
                    checklist.push(ReleaseChecklistItem {
                        number,
                        item: cells[0].clone(),
                        status: cells[1].clone(),
                        blocking: cells[2].clone(),
                        notes: cells[3].clone(),
                    });
                }
            }
            Section::Risks => {
                if let Some(item) = parse_list_item(trimmed) {
                    open_risks.push(item);
                }
            }
            Section::None => {}
        }
    }

    ReleaseReadiness { overall_state_summary, checklist, open_risks }
}

/// Parse `session-insights.md` into `SessionInsights`.
/// Metadata blockquotes (Date:, Epic:, Phase:), then 4 list sections.
fn parse_session_insights_md(content: &str) -> SessionInsights {
    let mut date = String::from("unknown");
    let mut epic = String::from("unknown");
    let mut phase = String::from("unknown");
    let mut session_id = String::new();
    let mut duration_secs: u64 = 0;
    let mut tokens_input: u64 = 0;
    let mut tokens_output: u64 = 0;
    let mut agents_used: Vec<String> = Vec::new();
    let mut decisions: Vec<String> = Vec::new();
    let mut learnings: Vec<String> = Vec::new();
    let mut risks: Vec<String> = Vec::new();
    let mut next_validations: Vec<String> = Vec::new();
    let mut suggested_next_persona = String::new();

    enum Section { None, Decisions, Learnings, Risks, NextValidations }
    let mut section = Section::None;

    for line in content.lines() {
        let trimmed = line.trim();

        // Blockquote metadata: "> Date: value" (no space before colon)
        if trimmed.starts_with("> ") {
            let meta = &trimmed[2..];
            if let Some(val) = meta.strip_prefix("Date:") {
                let v = val.trim();
                if !v.is_empty() && v != "YYYY-MM-DD" { date = v.to_string(); }
            } else if let Some(val) = meta.strip_prefix("Epic:") {
                let v = val.trim();
                if !v.is_empty() && !v.starts_with('_') { epic = v.to_string(); }
            } else if let Some(val) = meta.strip_prefix("Phase:") {
                let v = val.trim();
                if !v.is_empty() && !v.starts_with('_') { phase = v.to_string(); }
            } else if let Some(val) = meta.strip_prefix("Session-ID:") {
                let v = val.trim();
                if !v.is_empty() && !v.starts_with('_') { session_id = v.to_string(); }
            } else if let Some(val) = meta.strip_prefix("Duration:") {
                let v = val.trim().trim_end_matches('s');
                duration_secs = v.parse().unwrap_or(0);
            } else if let Some(val) = meta.strip_prefix("Tokens:") {
                // Format: "1234 in / 5678 out"
                let parts: Vec<&str> = val.split('/').collect();
                if parts.len() == 2 {
                    tokens_input = parts[0].trim().trim_end_matches(" in").trim().parse().unwrap_or(0);
                    tokens_output = parts[1].trim().trim_end_matches(" out").trim().parse().unwrap_or(0);
                }
            } else if let Some(val) = meta.strip_prefix("Agents:") {
                agents_used = val.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            } else if let Some(val) = meta.strip_prefix("Suggested-Persona:") {
                let v = val.trim();
                if !v.is_empty() { suggested_next_persona = v.to_string(); }
            }
            continue;
        }

        if trimmed.starts_with("## ") {
            let heading = trimmed[3..].trim().to_lowercase();
            section = if heading == "decisions prises" {
                Section::Decisions
            } else if heading.starts_with("ce qu") {
                Section::Learnings
            } else if heading.starts_with("risques") {
                Section::Risks
            } else if heading.starts_with("prochaines") {
                Section::NextValidations
            } else {
                Section::None
            };
            continue;
        }

        if trimmed.is_empty() || trimmed.starts_with("# ") { continue; }

        match section {
            Section::Decisions => {
                if let Some(item) = parse_list_item(trimmed) { decisions.push(item); }
            }
            Section::Learnings => {
                if let Some(item) = parse_list_item(trimmed) { learnings.push(item); }
            }
            Section::Risks => {
                if let Some(item) = parse_list_item(trimmed) { risks.push(item); }
            }
            Section::NextValidations => {
                if let Some(item) = parse_list_item(trimmed) { next_validations.push(item); }
            }
            Section::None => {}
        }
    }

    SessionInsights { date, epic, phase, session_id, duration_secs, tokens_input, tokens_output, agents_used, decisions, learnings, risks, next_validations, suggested_next_persona }
}

/// Parse a persona `.md` file.
/// Format: H1 title, blockquote metadata (Role:, Maps-to:), H2 sections with lists.
fn parse_persona_md(content: &str) -> Persona {
    let mut name = String::new();
    let mut role = String::new();
    let mut maps_to = String::from("none");
    let mut description = String::new();
    let mut responsibilities: Vec<String> = Vec::new();
    let mut when_active: Vec<String> = Vec::new();

    enum Section { None, Description, Responsibilities, WhenActive }
    let mut section = Section::None;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("# ") && !trimmed.starts_with("## ") {
            name = trimmed[2..].trim().to_string();
            continue;
        }

        if trimmed.starts_with("> ") {
            let meta = &trimmed[2..];
            if let Some(val) = meta.strip_prefix("Role:") {
                role = val.trim().to_string();
            } else if let Some(val) = meta.strip_prefix("Maps-to:") {
                let v = val.trim();
                if !v.is_empty() { maps_to = v.to_string(); }
            }
            continue;
        }

        if trimmed.starts_with("## ") {
            let heading = trimmed[3..].trim().to_lowercase();
            section = if heading == "description" {
                Section::Description
            } else if heading.starts_with("responsabilit") {
                Section::Responsibilities
            } else if heading.starts_with("quand") {
                Section::WhenActive
            } else {
                Section::None
            };
            continue;
        }

        if trimmed.is_empty() { continue; }

        match section {
            Section::Description => {
                if !description.is_empty() { description.push(' '); }
                description.push_str(trimmed);
            }
            Section::Responsibilities => {
                if let Some(item) = parse_list_item(trimmed) { responsibilities.push(item); }
            }
            Section::WhenActive => {
                if let Some(item) = parse_list_item(trimmed) { when_active.push(item); }
            }
            Section::None => {}
        }
    }

    Persona { name, role, maps_to, description, responsibilities, when_active }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use super::super::models::*;

    #[test]
    fn test_memory_reader_creation() {
        let reader = MemoryReader::new(PathBuf::from("/tmp"));
        assert!(!reader.memory_dir.as_os_str().is_empty());
    }

    // --- parse_index_entry tests ---

    #[test]
    fn test_parse_entry_em_dash() {
        let line = "- [architecture.md](architecture.md) \u{2014} Stack Tauri v2 + React/TS + Rust";
        let entry = parse_index_entry(line).expect("should parse");
        assert_eq!(entry.title, "architecture.md");
        assert_eq!(entry.filename, "architecture.md");
        assert_eq!(entry.description, "Stack Tauri v2 + React/TS + Rust");
    }

    #[test]
    fn test_parse_entry_double_hyphen() {
        let line = "- [State](state.md) -- Current session state";
        let entry = parse_index_entry(line).expect("should parse");
        assert_eq!(entry.title, "State");
        assert_eq!(entry.filename, "state.md");
        assert_eq!(entry.description, "Current session state");
    }

    #[test]
    fn test_parse_entry_with_subdirectory_path() {
        let line =
            "- [sessions/2026-03-29.md](sessions/2026-03-29.md) \u{2014} Session initiale";
        let entry = parse_index_entry(line).expect("should parse");
        assert_eq!(entry.title, "sessions/2026-03-29.md");
        assert_eq!(entry.filename, "sessions/2026-03-29.md");
        assert_eq!(entry.description, "Session initiale");
    }

    #[test]
    fn test_parse_entry_no_description() {
        let line = "- [Readme](readme.md)";
        let entry = parse_index_entry(line).expect("should parse");
        assert_eq!(entry.title, "Readme");
        assert_eq!(entry.filename, "readme.md");
        assert_eq!(entry.description, "");
    }

    #[test]
    fn test_parse_entry_malformed_returns_none() {
        assert!(parse_index_entry("not a list item").is_none());
        assert!(parse_index_entry("- no brackets here").is_none());
        assert!(parse_index_entry("- [](empty.md)").is_none());
        assert!(parse_index_entry("- [Title] no parens").is_none());
    }

    // --- parse_index_md tests ---

    #[test]
    fn test_parse_full_index_matches_real_format() {
        let content = r#"# Memoire Projet ZAOS — Index

> Point d'entree de la memoire.

## Reference (charger selon besoin)

- [architecture.md](architecture.md) — Stack Tauri v2 + React/TS + Rust
- [conventions.md](conventions.md) — Standards TypeScript/React/Rust pour ZAOS
- [state.md](state.md) — Phase 1 a 68%, backend Rust complet

## Decisions

<!-- Format : - [decisions/NNN-titre.md](decisions/NNN-titre.md) — Resume 1 ligne -->

## Sessions recentes

- [sessions/2026-03-29.md](sessions/2026-03-29.md) — Session initiale
"#;

        let index = parse_index_md(content);
        assert_eq!(index.sections.len(), 3);

        // First section: Reference
        assert_eq!(
            index.sections[0].heading,
            "Reference (charger selon besoin)"
        );
        assert_eq!(index.sections[0].entries.len(), 3);
        assert_eq!(index.sections[0].entries[0].title, "architecture.md");
        assert_eq!(index.sections[0].entries[0].filename, "architecture.md");
        assert_eq!(
            index.sections[0].entries[0].description,
            "Stack Tauri v2 + React/TS + Rust"
        );

        // Second section: Decisions (empty, only HTML comments)
        assert_eq!(index.sections[1].heading, "Decisions");
        assert_eq!(index.sections[1].entries.len(), 0);

        // Third section: Sessions
        assert_eq!(index.sections[2].heading, "Sessions recentes");
        assert_eq!(index.sections[2].entries.len(), 1);
        assert_eq!(
            index.sections[2].entries[0].filename,
            "sessions/2026-03-29.md"
        );
    }

    #[test]
    fn test_parse_empty_content() {
        let index = parse_index_md("");
        assert!(index.sections.is_empty());
    }

    #[test]
    fn test_parse_entries_before_any_heading() {
        let content = "- [orphan.md](orphan.md) \u{2014} No section above\n";
        let index = parse_index_md(content);
        assert_eq!(index.sections.len(), 1);
        assert_eq!(index.sections[0].heading, "Uncategorized");
        assert_eq!(index.sections[0].entries.len(), 1);
    }

    #[test]
    fn test_find_by_filename() {
        let index =
            parse_index_md("## Ref\n- [State](state.md) \u{2014} Current state\n");
        let found = index.find_by_filename("state.md");
        assert!(found.is_some());
        assert_eq!(found.expect("entry exists").title, "State");
        assert!(index.find_by_filename("nonexistent.md").is_none());
    }

    #[test]
    fn test_all_entries_spans_sections() {
        let content =
            "## A\n- [X](x.md) \u{2014} X desc\n## B\n- [Y](y.md) \u{2014} Y desc\n- [Z](z.md) \u{2014} Z desc\n";
        let index = parse_index_md(content);
        let all: Vec<_> = index.all_entries().collect();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].title, "X");
        assert_eq!(all[1].title, "Y");
        assert_eq!(all[2].title, "Z");
    }

    #[tokio::test]
    async fn test_read_index_missing_file_returns_empty() {
        let reader = MemoryReader::new(PathBuf::from("/nonexistent/path"));
        let result = reader.read_index().await;
        assert!(result.is_ok());
        let index = result.expect("should be Ok");
        assert!(index.sections.is_empty());
    }

    // --- parse_state_md tests ---

    #[test]
    fn test_parse_state_md_full() {
        let md = r#"# Etat courant ZAOS

> Derniere mise a jour : 2026-03-30

## Milestones

| # | Milestone | Statut | Epics |
|---|---|---|---|
| 1 | Chat fonctionnel avec Claude Code CLI | TERMINE | Chat complet, Interactive Permissions, Phase 1 Finition (6/6) |
| 2 | Dashboard temps reel | Non commence | Actions feed, Agents, Pipeline, Memory |
| 3 | Screenshots & visuels | Non commence | Screenshot manager, Gallery, GoPeak |

## Epic active

Phase 2 — Dashboard Temps Reel EN COURS (0/17 taches)

## Blocages

Aucun
"#;
        let result = parse_state_md(md);
        assert!(result.warnings.is_empty(), "full state should have no warnings");
        let state = result.value;

        assert_eq!(state.milestones.len(), 3);

        assert_eq!(state.milestones[0].number, 1);
        assert_eq!(
            state.milestones[0].name,
            "Chat fonctionnel avec Claude Code CLI"
        );
        assert_eq!(state.milestones[0].status, "TERMINE");
        assert!(state.milestones[0].epics.contains("Chat complet"));

        assert_eq!(state.milestones[1].number, 2);
        assert_eq!(state.milestones[1].status, "Non commence");

        assert_eq!(state.milestones[2].number, 3);

        assert!(state.active_epic.contains("Phase 2"));
        assert!(state.active_epic.contains("EN COURS"));
        assert_eq!(state.blocages, "Aucun");
    }

    #[test]
    fn test_parse_state_md_empty() {
        let result = parse_state_md("");
        let state = result.value;
        assert!(state.milestones.is_empty());
        assert!(state.active_epic.is_empty());
        assert!(state.blocages.is_empty());
        // Empty content should warn about missing sections
        assert!(result.warnings.iter().any(|w| w.message == "milestones section missing"));
        assert!(result.warnings.iter().any(|w| w.message == "active_epic section missing"));
    }

    #[test]
    fn test_parse_state_md_missing_sections() {
        let md = "## Milestones\n\n| # | Milestone | Statut | Epics |\n|---|---|---|---|\n| 1 | Only one | DONE | stuff |\n";
        let result = parse_state_md(md);
        let state = result.value;

        assert_eq!(state.milestones.len(), 1);
        assert_eq!(state.milestones[0].name, "Only one");
        assert!(state.active_epic.is_empty());
        assert!(state.blocages.is_empty());
        // Missing epic active section should produce a warning
        assert!(result.warnings.iter().any(|w| w.message == "active_epic section missing"));
    }

    #[test]
    fn test_parse_state_md_multiline_blocages() {
        let md = "## Blocages\n\nPremier blocage\nDeuxieme blocage\n";
        let result = parse_state_md(md);
        assert_eq!(result.value.blocages, "Premier blocage\nDeuxieme blocage");
    }

    #[test]
    fn test_parse_state_md_other_sections_ignored() {
        let md = r#"## Milestones

| # | Milestone | Statut | Epics |
|---|---|---|---|
| 1 | M1 | DONE | e1 |

## Ce qui est fait

- some stuff done

## Epic active

Active now

## Bugs connus

- a bug

## Blocages

None
"#;
        let result = parse_state_md(md);
        let state = result.value;
        assert_eq!(state.milestones.len(), 1);
        assert_eq!(state.active_epic, "Active now");
        assert_eq!(state.blocages, "None");
    }

    // --- parse_milestone_row tests ---

    #[test]
    fn test_parse_milestone_row_valid() {
        let entry = parse_milestone_row("| 1 | Chat | TERMINE | Epics list |");
        assert!(entry.is_some());
        let e = entry.unwrap();
        assert_eq!(e.number, 1);
        assert_eq!(e.name, "Chat");
        assert_eq!(e.status, "TERMINE");
        assert_eq!(e.epics, "Epics list");
    }

    #[test]
    fn test_parse_milestone_row_separator() {
        assert!(parse_milestone_row("|---|---|---|---|").is_none());
    }

    #[test]
    fn test_parse_milestone_row_header() {
        assert!(parse_milestone_row("| # | Milestone | Statut | Epics |").is_none());
    }

    #[test]
    fn test_parse_milestone_row_not_a_table() {
        assert!(parse_milestone_row("Just some text").is_none());
    }

    // --- parse_current_epic_md tests ---

    #[test]
    fn test_parse_current_epic_md_full() {
        let md = r#"# Epic active : Phase 2 — Dashboard Temps Reel

> Milestone : 2 — Dashboard temps reel
> Date de debut : 2026-03-30
> Statut : EN COURS

## Objectif

Rendre le dashboard ZAOS vivant.

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| 1 | Creer module watchers | `watchers/mod.rs`, `watchers/service.rs` | VALIDATED | done |
| 2 | Enregistrer service | `commands.rs`, `main.rs` | A FAIRE | pending |

## Streams de travail

- Stream A
"#;
        let result = parse_current_epic_md(md);
        assert!(result.warnings.is_empty(), "full epic should have no warnings");
        let epic = result.value;
        assert_eq!(epic.name, "Phase 2 — Dashboard Temps Reel");
        assert_eq!(epic.milestone, "2 — Dashboard temps reel");
        assert_eq!(epic.status, "EN COURS");
        assert!(epic.objective.contains("dashboard ZAOS vivant"));
        assert_eq!(epic.tasks.len(), 2);

        assert_eq!(epic.tasks[0].number, 1);
        assert_eq!(epic.tasks[0].name, "Creer module watchers");
        assert_eq!(
            epic.tasks[0].files,
            vec!["watchers/mod.rs", "watchers/service.rs"]
        );
        assert_eq!(epic.tasks[0].status, "VALIDATED");

        assert_eq!(epic.tasks[1].number, 2);
        assert_eq!(epic.tasks[1].status, "A FAIRE");
    }

    #[test]
    fn test_parse_current_epic_md_empty() {
        let result = parse_current_epic_md("");
        let epic = result.value;
        assert!(epic.name.is_empty());
        assert!(epic.tasks.is_empty());
        // Empty content should warn about missing name, milestone, status
        assert!(result.warnings.iter().any(|w| w.message == "epic name missing"));
        assert!(result.warnings.iter().any(|w| w.message == "milestone blockquote missing"));
        assert!(result.warnings.iter().any(|w| w.message == "status blockquote missing"));
    }

    #[test]
    fn test_parse_epic_task_row_valid() {
        let task = parse_epic_task_row("| 3 | Do thing | `file.rs` | DONE | all good |");
        assert!(task.is_some());
        let t = task.unwrap();
        assert_eq!(t.number, 3);
        assert_eq!(t.name, "Do thing");
        assert_eq!(t.files, vec!["file.rs"]);
        assert_eq!(t.status, "DONE");
        assert_eq!(t.notes, "all good");
    }

    #[test]
    fn test_parse_epic_task_row_separator() {
        assert!(parse_epic_task_row("|---|------|-----------|--------|-------|").is_none());
    }

    // --- parse_md_table_row empty cells tests ---

    #[test]
    fn test_parse_md_table_row_empty_notes() {
        // Notes column is empty — should still parse as 5 cells
        let result = parse_md_table_row("| 1 | Task name | `file.rs` | DONE | |", 5);
        assert!(result.is_some(), "row with empty Notes should parse");
        let (num, cells) = result.unwrap();
        assert_eq!(num, 1);
        assert_eq!(cells.len(), 4); // Task, Fichier, Statut, Notes
        assert_eq!(cells[0], "Task name");
        assert_eq!(cells[1], "`file.rs`");
        assert_eq!(cells[2], "DONE");
        assert_eq!(cells[3], ""); // empty Notes preserved
    }

    #[test]
    fn test_parse_md_table_row_empty_middle_column() {
        // Middle column empty — should preserve it
        let result = parse_md_table_row("| 2 | Task | | EN COURS | notes |", 5);
        assert!(result.is_some(), "row with empty middle column should parse");
        let (num, cells) = result.unwrap();
        assert_eq!(num, 2);
        assert_eq!(cells[0], "Task");
        assert_eq!(cells[1], ""); // empty Fichier(s) preserved
        assert_eq!(cells[2], "EN COURS");
        assert_eq!(cells[3], "notes");
    }

    #[test]
    fn test_parse_md_table_row_normal_row() {
        let result = parse_md_table_row("| 3 | Do thing | `file.rs` | DONE | all good |", 5);
        assert!(result.is_some());
        let (num, cells) = result.unwrap();
        assert_eq!(num, 3);
        assert_eq!(cells[0], "Do thing");
        assert_eq!(cells[3], "all good");
    }

    #[test]
    fn test_parse_md_table_row_separator_line() {
        assert!(parse_md_table_row("|---|------|-----------|--------|-------|", 5).is_none());
    }

    #[test]
    fn test_parse_md_table_row_header_line() {
        assert!(parse_md_table_row("| # | Task | Fichier(s) | Statut | Notes |", 5).is_none());
    }

    #[test]
    fn test_parse_md_table_row_too_few_cells() {
        assert!(parse_md_table_row("| 1 | only two |", 5).is_none());
    }

    #[tokio::test]
    async fn test_read_state_missing_file_returns_error() {
        let reader = MemoryReader::new(PathBuf::from("/nonexistent/path"));
        let result = reader.read_state().await;
        assert!(result.is_err());
    }

    // --- parse_list_item tests ---

    #[test]
    fn test_parse_list_item_valid() {
        assert_eq!(parse_list_item("- real item"), Some("real item".to_string()));
    }

    #[test]
    fn test_parse_list_item_placeholder_skipped() {
        assert_eq!(parse_list_item("- _placeholder text_"), None);
    }

    #[test]
    fn test_parse_list_item_not_a_list() {
        assert_eq!(parse_list_item("just text"), None);
    }

    #[test]
    fn test_parse_list_item_empty() {
        assert_eq!(parse_list_item("- "), None);
    }

    // --- append_text tests ---

    #[test]
    fn test_append_text_to_empty() {
        let mut s = String::new();
        append_text(&mut s, "hello");
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_append_text_multiline() {
        let mut s = String::from("line1");
        append_text(&mut s, "line2");
        assert_eq!(s, "line1\nline2");
    }

    // --- parse_product_brief_md tests ---

    #[test]
    fn test_parse_product_brief_md_full() {
        let content = "\
# Product Brief

## Vision

A productivity dashboard for developers.

## Pour qui

Senior engineers working remotely.

## Pourquoi

Reduce context switching between tools.

## Contraintes

- Must work offline
- Under 50MB binary

## Hors-scope

- Mobile app
- Cloud sync

## Definition de succes

Users save 30 minutes per day.
";
        let brief = parse_product_brief_md(content);
        assert_eq!(brief.vision, "A productivity dashboard for developers.");
        assert_eq!(brief.audience, "Senior engineers working remotely.");
        assert_eq!(brief.rationale, "Reduce context switching between tools.");
        assert_eq!(brief.constraints, vec!["Must work offline", "Under 50MB binary"]);
        assert_eq!(brief.out_of_scope, vec!["Mobile app", "Cloud sync"]);
        assert_eq!(brief.success_definition, "Users save 30 minutes per day.");
    }

    #[test]
    fn test_parse_product_brief_md_empty() {
        let brief = parse_product_brief_md("");
        assert_eq!(brief.vision, "");
        assert_eq!(brief.constraints.len(), 0);
    }

    #[test]
    fn test_parse_product_brief_md_template_placeholders_ignored() {
        let content = "\
# Product Brief

## Contraintes

- _contrainte 1_

## Hors-scope

- _element explicitement exclu_
";
        let brief = parse_product_brief_md(content);
        assert!(brief.constraints.is_empty());
        assert!(brief.out_of_scope.is_empty());
    }

    // --- parse_experience_goals_md tests ---

    #[test]
    fn test_parse_experience_goals_md_full() {
        let content = "\
# Experience Goals

## Qualites cibles

| # | Qualite | Critere | Priorite |
|---|---------|---------|----------|
| 1 | Fast | < 200ms response | P0 |
| 2 | Intuitive | No manual needed | P1 |

## Standards UX

- Keyboard-first navigation
- Dark mode default

## Anti-patterns

- Modal dialogs for simple actions
";
        let goals = parse_experience_goals_md(content);
        assert_eq!(goals.goals.len(), 2);
        assert_eq!(goals.goals[0].quality, "Fast");
        assert_eq!(goals.goals[0].priority, "P0");
        assert_eq!(goals.goals[1].number, 2);
        assert_eq!(goals.ux_standards, vec!["Keyboard-first navigation", "Dark mode default"]);
        assert_eq!(goals.anti_patterns, vec!["Modal dialogs for simple actions"]);
    }

    // --- parse_acceptance_checks_md tests ---

    #[test]
    fn test_parse_acceptance_checks_md_full() {
        let content = "\
# Acceptance Checks

## Criteres

| # | Check | Statut | Notes |
|---|-------|--------|-------|
| 1 | Chat works | PASS | tested manually |
| 2 | Dashboard loads | TODO | |

## Validations manuelles

- Cross-browser test on Firefox
";
        let checks = parse_acceptance_checks_md(content);
        assert_eq!(checks.checks.len(), 2);
        assert_eq!(checks.checks[0].status, "PASS");
        assert_eq!(checks.checks[1].status, "TODO");
        assert_eq!(checks.checks[1].notes, "");
        assert_eq!(checks.manual_validations, vec!["Cross-browser test on Firefox"]);
    }

    // --- parse_release_readiness_md tests ---

    #[test]
    fn test_parse_release_readiness_md_full() {
        let content = "\
# Release Readiness

## Etat general

Ready for beta testing.

## Checklist

| # | Item | Statut | Bloquant | Notes |
|---|------|--------|----------|-------|
| 1 | All tests pass | DONE | YES | 72/72 |
| 2 | Docs updated | TODO | NO | |

## Risques ouverts

- Performance on large projects untested
";
        let rr = parse_release_readiness_md(content);
        assert_eq!(rr.overall_state_summary, "Ready for beta testing.");
        assert_eq!(rr.checklist.len(), 2);
        assert_eq!(rr.checklist[0].blocking, "YES");
        assert_eq!(rr.checklist[1].status, "TODO");
        assert_eq!(rr.open_risks, vec!["Performance on large projects untested"]);
    }

    // --- parse_session_insights_md tests ---

    #[test]
    fn test_parse_session_insights_md_full() {
        let content = "\
# Session Insights

> Date: 2026-04-03
> Epic: Backend Memory
> Phase: implementation
> Session-ID: abc12345
> Duration: 3600s
> Tokens: 1234 in / 5678 out
> Agents: architect, coder, reviewer
> Suggested-Persona: Builder

## Decisions prises

- Use models.rs for shared types
- Separate command from get_memory_state

## Ce qu'on a appris

- tokio::join! scales well for parallel reads

## Risques et points ouverts

- No tests for new parsers yet

## Prochaines validations

- Run full test suite after Epic 3
";
        let si = parse_session_insights_md(content);
        assert_eq!(si.date, "2026-04-03");
        assert_eq!(si.epic, "Backend Memory");
        assert_eq!(si.phase, "implementation");
        assert_eq!(si.session_id, "abc12345");
        assert_eq!(si.duration_secs, 3600);
        assert_eq!(si.tokens_input, 1234);
        assert_eq!(si.tokens_output, 5678);
        assert_eq!(si.agents_used, vec!["architect", "coder", "reviewer"]);
        assert_eq!(si.decisions.len(), 2);
        assert_eq!(si.learnings, vec!["tokio::join! scales well for parallel reads"]);
        assert_eq!(si.risks, vec!["No tests for new parsers yet"]);
        assert_eq!(si.next_validations, vec!["Run full test suite after Epic 3"]);
        assert_eq!(si.suggested_next_persona, "Builder");
    }

    #[test]
    fn test_parse_session_insights_md_missing_metadata() {
        let content = "\
# Session Insights

## Decisions prises

- One decision
";
        let si = parse_session_insights_md(content);
        assert_eq!(si.date, "unknown");
        assert_eq!(si.epic, "unknown");
        assert_eq!(si.phase, "unknown");
        assert_eq!(si.decisions, vec!["One decision"]);
        assert_eq!(si.suggested_next_persona, "");
    }

    #[test]
    fn test_parse_session_insights_md_template_defaults() {
        let content = "\
# Session Insights

> Date: YYYY-MM-DD
> Epic: _aucun_
> Phase: _inconnue_

## Decisions prises

- _decision 1_
";
        let si = parse_session_insights_md(content);
        assert_eq!(si.date, "unknown");
        assert_eq!(si.epic, "unknown");
        assert_eq!(si.phase, "unknown");
        assert!(si.decisions.is_empty());
        assert_eq!(si.suggested_next_persona, "");
    }

    #[test]
    fn test_parse_persona_md_full() {
        let content = "\
# Builder

> Role: Implementation du code
> Maps-to: coder

## Description

Implemente le code selon le plan valide.

## Responsabilites

- Implementer les taches du plan
- Ecrire du code lisible

## Quand ce role intervient

- Phase implementation
- Apres review
";
        let p = parse_persona_md(content);
        assert_eq!(p.name, "Builder");
        assert_eq!(p.role, "Implementation du code");
        assert_eq!(p.maps_to, "coder");
        assert_eq!(p.description, "Implemente le code selon le plan valide.");
        assert_eq!(p.responsibilities.len(), 2);
        assert_eq!(p.responsibilities[0], "Implementer les taches du plan");
        assert_eq!(p.when_active.len(), 2);
        assert_eq!(p.when_active[0], "Phase implementation");
    }

    #[test]
    fn test_parse_persona_md_empty() {
        let content = "";
        let p = parse_persona_md(content);
        assert_eq!(p.name, "");
        assert_eq!(p.role, "");
        assert_eq!(p.maps_to, "none");
        assert_eq!(p.description, "");
        assert!(p.responsibilities.is_empty());
        assert!(p.when_active.is_empty());
    }
}
