use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use tokio::fs;

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

        let state = parse_state_md(&content);
        tracing::debug!(
            "Parsed state: {} milestones, active_epic={:?}",
            state.milestones.len(),
            state.active_epic
        );
        Ok(state)
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

        let epic = parse_current_epic_md(&content);
        tracing::debug!(
            "Parsed current epic: name={:?}, {} tasks",
            epic.name,
            epic.tasks.len()
        );
        Ok(Some(epic))
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
}

/// Parse the full content of `state.md` into a `MemoryState`.
///
/// Uses a simple state-machine approach: track the current `## Heading` and
/// collect lines accordingly.  Missing sections gracefully produce empty/default
/// values.
fn parse_state_md(content: &str) -> MemoryState {
    let mut milestones: Vec<MilestoneEntry> = Vec::new();
    let mut active_epic = String::new();
    let mut blocages = String::new();

    enum Section {
        None,
        Milestones,
        ActiveEpic,
        Blocages,
        Other,
    }

    let mut section = Section::None;

    for line in content.lines() {
        let trimmed = line.trim();

        // Detect section headings (## ...)
        if trimmed.starts_with("## ") {
            let heading = trimmed[3..].trim().to_lowercase();
            section = if heading == "milestones" {
                Section::Milestones
            } else if heading.starts_with("epic active") || heading.starts_with("epic actif") {
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
                if let Some(entry) = parse_milestone_row(trimmed) {
                    milestones.push(entry);
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

    MemoryState {
        milestones,
        active_epic,
        blocages,
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
fn parse_current_epic_md(content: &str) -> CurrentEpic {
    let mut name = String::new();
    let mut milestone = String::new();
    let mut status = String::new();
    let mut objective = String::new();
    let mut tasks: Vec<EpicTask> = Vec::new();

    enum Section {
        Header,
        Objective,
        Tasks,
        Other,
    }

    let mut section = Section::Header;

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
            } else if let Some(val) = meta.strip_prefix("Statut :") {
                status = val.trim().to_string();
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
                if let Some(task) = parse_epic_task_row(trimmed) {
                    tasks.push(task);
                }
            }
            _ => {}
        }
    }

    CurrentEpic {
        name,
        milestone,
        status,
        objective,
        tasks,
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

#[cfg(test)]
mod tests {
    use super::*;

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
        let state = parse_state_md(md);

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
        let state = parse_state_md("");
        assert!(state.milestones.is_empty());
        assert!(state.active_epic.is_empty());
        assert!(state.blocages.is_empty());
    }

    #[test]
    fn test_parse_state_md_missing_sections() {
        let md = "## Milestones\n\n| # | Milestone | Statut | Epics |\n|---|---|---|---|\n| 1 | Only one | DONE | stuff |\n";
        let state = parse_state_md(md);

        assert_eq!(state.milestones.len(), 1);
        assert_eq!(state.milestones[0].name, "Only one");
        assert!(state.active_epic.is_empty());
        assert!(state.blocages.is_empty());
    }

    #[test]
    fn test_parse_state_md_multiline_blocages() {
        let md = "## Blocages\n\nPremier blocage\nDeuxieme blocage\n";
        let state = parse_state_md(md);
        assert_eq!(state.blocages, "Premier blocage\nDeuxieme blocage");
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
        let state = parse_state_md(md);
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
        let epic = parse_current_epic_md(md);
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
        let epic = parse_current_epic_md("");
        assert!(epic.name.is_empty());
        assert!(epic.tasks.is_empty());
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
}
