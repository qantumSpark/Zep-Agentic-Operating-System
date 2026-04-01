use serde::Deserialize;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process;

// ---------------------------------------------------------------------------
// Types (standalone — no dependency on the main zaos crate / Tauri)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct WorkflowState {
    phase: String,
    epic: String,
    task: String,
    mode: String,
    gate_validated: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Config {
    blocked_extensions: Vec<String>,
    hooks_enabled: bool,
    disabled_rules: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            blocked_extensions: vec![".rs".into(), ".ts".into(), ".tsx".into()],
            hooks_enabled: true,
            disabled_rules: vec![],
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn get_project_dir() -> PathBuf {
    std::env::var("CLAUDE_PROJECT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn load_state(project_dir: &Path) -> Option<WorkflowState> {
    let path = project_dir.join(".workflow").join("state.json");
    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

fn load_config(project_dir: &Path) -> Config {
    let path = project_dir.join(".zaos").join("config.json");
    fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

fn load_file_head(path: &Path, max_lines: usize) -> String {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .take(max_lines)
        .collect::<Vec<_>>()
        .join("\n")
}

fn load_file_full(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

fn get_phase_instructions(phase: &str) -> &'static str {
    match phase {
        "comprehension" => "Phase COMPREHENSION: Avant de planifier, clarifie le besoin avec l'utilisateur (scope, stack, contraintes, priorites). Sur un projet vierge, pose des questions. Sur un projet existant, analyse le code. Delegue au Researcher si besoin d'infos externes.",
        "specification" => "Phase SPECIFICATION: Redige les specifications. Delegue a l'Architect.",
        "architecture" => "Phase ARCHITECTURE: Concois l'architecture. Delegue a l'Architect.",
        "implementation" => {
            "Phase IMPLEMENTATION: Code les taches du plan. Delegue au Coder agent."
        }
        "review" => "Phase REVIEW: Revise le code produit. Delegue au Reviewer.",
        "test" => "Phase TEST: Teste le code produit. Delegue au Tester.",
        "closure" => "Phase CLOSURE: Mets a jour la memoire et ferme l'epic.",
        "idle" => "Phase IDLE: Aucun epic actif. Quand l'utilisateur demande une feature, pose des questions de cadrage AVANT de lancer le pipeline (stack technique, scope, contraintes, public cible). Ne pas improviser.",
        _ => "Phase inconnue. Demande clarification a l'utilisateur.",
    }
}

const NON_NEGOTIABLE_RULES: &str = "\
REGLES NON-NEGOCIABLES:
- JAMAIS coder sans plan valide dans current-epic.md
- JAMAIS skip un gate sans validation utilisateur
- TOUJOURS deleguer via subagent Task (tu ne codes JAMAIS directement)
- JAMAIS inventer des APIs/classes/methodes — verifie toujours la doc";

const ROLE_REMINDER: &str = "Tu es l'Orchestrateur ZAOS. \
Tu ne codes JAMAIS directement. \
Tu delegues TOUJOURS aux agents specialises.";

const FORMAT_REMINDER: &str = "\
FORMAT MEMOIRE (obligatoire pour le dashboard):\
\n- current-epic.md: heading '# Epic active : <nom>', blockquotes '> Milestone :', '> Statut :', sections '## Objectif', '## Tasks' avec tableau 5 colonnes (# | Task | Fichier(s) | Statut | Notes)\
\n- state.md: sections '## Milestones' (tableau 4 col), '## Epic active', '## Blocages'\
\n- Statuts tasks: A FAIRE, EN COURS, TODO, DONE, VALIDATED, BLOQUE";

// ---------------------------------------------------------------------------
// B2: inject-context — called on every UserPromptSubmit
// ---------------------------------------------------------------------------

fn cmd_inject_context() {
    let project_dir = get_project_dir();
    let state = load_state(&project_dir);
    let _config = load_config(&project_dir);

    let mut context = String::new();

    // 1. Role reminder
    context.push_str(ROLE_REMINDER);
    context.push_str("\n\n");

    // 2. Current state
    if let Some(ref st) = state {
        context.push_str(&format!(
            "ETAT WORKFLOW: phase={}, epic={}, task={}, mode={}, gate_validated={}\n\n",
            st.phase, st.epic, st.task, st.mode, st.gate_validated
        ));

        // 4. Phase-specific instructions
        context.push_str(get_phase_instructions(&st.phase));
        context.push_str("\n\n");
    } else {
        context.push_str("ETAT WORKFLOW: state.json introuvable — mode libre.\n\n");
    }

    // 3. Non-negotiable rules
    context.push_str(NON_NEGOTIABLE_RULES);
    context.push_str("\n\n");

    // 3b. Format reminder for memory files
    context.push_str(FORMAT_REMINDER);
    context.push_str("\n\n");

    // 5. First 20 lines of current-epic.md
    let epic_path = project_dir.join(".memory").join("current-epic.md");
    let epic_head = load_file_head(&epic_path, 20);
    if !epic_head.is_empty() {
        context.push_str("--- EPIC EN COURS (20 premieres lignes) ---\n");
        context.push_str(&epic_head);
        context.push('\n');
    }

    // Output as JSON
    let output = serde_json::json!({ "additionalContext": context });
    println!("{}", output);
}

// ---------------------------------------------------------------------------
// B3: block-code — called on PreToolUse for Write|Edit
// ---------------------------------------------------------------------------

fn cmd_block_code() {
    let project_dir = get_project_dir();
    let state = load_state(&project_dir);
    let config = load_config(&project_dir);

    // Read stdin as JSON to get tool_input.file_path
    let mut stdin_buf = String::new();
    if std::io::stdin().read_to_string(&mut stdin_buf).is_err() {
        eprintln!("zaos-hooks block-code: impossible de lire stdin");
        process::exit(0); // allow on read error — don't block the user
    }

    let file_path = match serde_json::from_str::<serde_json::Value>(&stdin_buf) {
        Ok(val) => val
            .get("tool_input")
            .and_then(|ti| ti.get("file_path"))
            .and_then(|fp| fp.as_str())
            .unwrap_or("")
            .to_string(),
        Err(_) => {
            eprintln!("zaos-hooks block-code: JSON stdin invalide");
            process::exit(0);
        }
    };

    // Get file extension
    let extension = Path::new(&file_path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e))
        .unwrap_or_default();

    // If extension NOT in blocked_extensions → exit 0 (allow non-code files)
    if !config.blocked_extensions.contains(&extension) {
        process::exit(0);
    }

    // If workflow mode is "free" → exit 0
    if let Some(ref st) = state {
        if st.mode == "free" {
            process::exit(0);
        }
    } else {
        // No state file → can't enforce, allow
        process::exit(0);
    }

    // If current-epic.md doesn't exist → exit 2
    let epic_path = project_dir.join(".memory").join("current-epic.md");
    let epic_content = match fs::read_to_string(&epic_path) {
        Ok(content) => content,
        Err(_) => {
            eprintln!(
                "BLOQUE: Ecriture de code interdite sans plan valide. \
                 Creez un plan dans current-epic.md d'abord."
            );
            process::exit(2);
        }
    };

    // If current-epic.md has no active tasks → exit 2
    let has_active_tasks = epic_content.contains("TODO")
        || epic_content.contains("EN COURS")
        || epic_content.contains("A FAIRE")
        || epic_content.contains("in_progress");

    if !has_active_tasks {
        eprintln!(
            "BLOQUE: Ecriture de code interdite sans plan valide. \
             Creez un plan dans current-epic.md d'abord."
        );
        process::exit(2);
    }

    // All checks passed → allow
    process::exit(0);
}

// ---------------------------------------------------------------------------
// B4: on-compact — called after session compaction
// ---------------------------------------------------------------------------

fn cmd_on_compact() {
    let project_dir = get_project_dir();
    let state = load_state(&project_dir);

    println!("=== ZAOS CONTEXTE POST-COMPACTION ===\n");

    // 1. Role reminder
    println!("{}\n", ROLE_REMINDER);

    // 2. Current workflow state
    if let Some(ref st) = state {
        println!("ETAT WORKFLOW:");
        println!("  Phase          : {}", st.phase);
        println!("  Epic           : {}", st.epic);
        println!("  Task           : {}", st.task);
        println!("  Mode           : {}", st.mode);
        println!("  Gate validated : {}", st.gate_validated);
        println!();

        // Phase-specific instruction
        println!("{}\n", get_phase_instructions(&st.phase));
    } else {
        println!("ETAT WORKFLOW: state.json introuvable — mode libre.\n");
    }

    // 3. Non-negotiable rules
    println!("{}\n", NON_NEGOTIABLE_RULES);

    // 4. Full content of current-epic.md
    let epic_path = project_dir.join(".memory").join("current-epic.md");
    let epic_content = load_file_full(&epic_path);
    if !epic_content.is_empty() {
        println!("=== current-epic.md (complet) ===");
        println!("{}\n", epic_content);
    } else {
        println!("=== current-epic.md : INTROUVABLE ===\n");
    }

    // 5. First 50 lines of state.md
    let state_md_path = project_dir.join(".memory").join("state.md");
    let state_md_content = load_file_head(&state_md_path, 50);
    if !state_md_content.is_empty() {
        println!("=== state.md (50 premieres lignes) ===");
        println!("{}\n", state_md_content);
    } else {
        println!("=== state.md : INTROUVABLE ===\n");
    }
}

// ---------------------------------------------------------------------------
// B4: welcome — diagnostic output at session start
// ---------------------------------------------------------------------------

fn cmd_welcome() {
    let project_dir = get_project_dir();
    let state = load_state(&project_dir);

    println!("=== ZAOS Diagnostic ===\n");

    // Check existence of key files
    let checks: Vec<(&str, PathBuf)> = vec![
        (
            ".workflow/state.json",
            project_dir.join(".workflow").join("state.json"),
        ),
        (
            ".memory/INDEX.md",
            project_dir.join(".memory").join("INDEX.md"),
        ),
        (
            ".memory/state.md",
            project_dir.join(".memory").join("state.md"),
        ),
        (
            ".memory/current-epic.md",
            project_dir.join(".memory").join("current-epic.md"),
        ),
        (
            ".zaos/config.json",
            project_dir.join(".zaos").join("config.json"),
        ),
    ];

    for (label, path) in &checks {
        let marker = if path.exists() { "[OK]" } else { "[--]" };
        println!("  {} {}", marker, label);
    }

    // Count files in .claude/agents/
    let agents_dir = project_dir.join(".claude").join("agents");
    let agent_count = count_files_in_dir(&agents_dir);
    let marker = if agent_count > 0 { "[OK]" } else { "[--]" };
    println!(
        "  {} .claude/agents/ ({} fichier{})",
        marker,
        agent_count,
        if agent_count != 1 { "s" } else { "" }
    );

    // Count files in .claude/rules/
    let rules_dir = project_dir.join(".claude").join("rules");
    let rules_count = count_files_in_dir(&rules_dir);
    let marker = if rules_count > 0 { "[OK]" } else { "[--]" };
    println!(
        "  {} .claude/rules/ ({} fichier{})",
        marker,
        rules_count,
        if rules_count != 1 { "s" } else { "" }
    );

    // Count settings.json
    let settings_path = project_dir.join(".claude").join("settings.json");
    let marker = if settings_path.exists() {
        "[OK]"
    } else {
        "[--]"
    };
    println!("  {} .claude/settings.json", marker);

    println!();

    // Current state summary
    if let Some(ref st) = state {
        println!("Etat courant:");
        println!("  Phase : {}", st.phase);
        println!("  Epic  : {}", if st.epic.is_empty() { "(aucun)" } else { &st.epic });
        println!("  Task  : {}", if st.task.is_empty() { "(aucune)" } else { &st.task });
        println!("  Mode  : {}", st.mode);
    } else {
        println!("Etat courant: state.json introuvable.");
    }

    println!("\n=== Fin diagnostic ===");
}

fn count_files_in_dir(dir: &Path) -> usize {
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
                .count()
        })
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match command {
        "inject-context" => cmd_inject_context(),
        "block-code" => cmd_block_code(),
        "on-compact" => cmd_on_compact(),
        "welcome" => cmd_welcome(),
        _ => {
            eprintln!("Usage: zaos-hooks <inject-context|block-code|on-compact|welcome>");
            process::exit(1);
        }
    }
}
