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
    #[serde(default)]
    gate_ready: bool,
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

fn has_active_tasks(epic_content: &str) -> bool {
    let mut found_data_row = false;

    for line in epic_content.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with('|') || trimmed.contains("---") {
            continue;
        }

        // Split by | and check column count (header has "# | Task | Fichier(s) | Statut | Notes")
        let cells: Vec<&str> = trimmed.split('|').map(|c| c.trim()).collect();
        // A pipe-delimited row "| a | b | c | d | e |" splits into ["", "a", "b", "c", "d", "e", ""]
        // We need at least 6 segments (5 columns + empty edges) for a valid task row
        if cells.len() < 6 {
            continue;
        }

        // Skip header row (first data column is "#" or column name)
        let first_cell = cells[1];
        if first_cell == "#" || first_cell == "Task" || first_cell == "Statut" {
            continue;
        }

        found_data_row = true;

        // Check status column (index 4 = 4th column = "Statut")
        let status = cells[4].to_uppercase();
        if status.contains("TODO")
            || status.contains("EN COURS")
            || status.contains("A FAIRE")
            || status.contains("IN_PROGRESS")
            || status.contains("BLOQUE")
        {
            return true;
        }
    }

    // 0 data rows = plan not yet created = consider as active (don't block)
    if !found_data_row {
        return true;
    }

    false
}

fn get_phase_instructions(phase: &str) -> &'static str {
    match phase {
        "idle" => "\
Phase IDLE\n\
Objectif: Cadrage et brainstorming avant pipeline.\n\
FAIS:\n\
- Poser des questions de cadrage (stack, scope, contraintes, public, existant)\n\
- Brainstormer les milestones avec l'utilisateur (proposer, challenger, iterer)\n\
- Challenger le scope : identifier ce qui est MVP vs nice-to-have\n\
- Identifier les incertitudes et les signaler\n\
- Lire le code existant pour comprendre le contexte\n\
- Quand les milestones sont valides, demander EXPLICITEMENT : 'On passe en mode pipeline ?'\n\
NE FAIS PAS:\n\
- Coder quoi que ce soit\n\
- Creer des plans ou des fichiers\n\
- Lancer le pipeline sans avoir brainstorme les milestones\n\
- Passer en pipeline automatiquement sans confirmation de l'utilisateur\n\
Gate: L'utilisateur confirme les milestones et lance start_epic.",

        "comprehension" => "\
Phase COMPREHENSION\n\
Objectif: Comprendre le besoin et l'existant.\n\
FAIS:\n\
- Clarifier les requirements avec l'utilisateur\n\
- Lire le code existant en profondeur\n\
- Deleguer au Researcher pour infos externes\n\
NE FAIS PAS:\n\
- Ecrire du code\n\
- Creer une architecture\n\
- Creer des fichiers source\n\
Agent: Researcher\n\
Gate: L'utilisateur valide que la comprehension est complete.",

        "specification" => "\
Phase SPECIFICATION\n\
Objectif: Rediger les specs fonctionnelles.\n\
FAIS:\n\
- Documenter les requirements precis\n\
- Identifier les cas limites\n\
- Definir les criteres d'acceptation\n\
- Deleguer a l'Architect\n\
NE FAIS PAS:\n\
- Ecrire du code\n\
- Creer des fichiers source\n\
Agent: Architect\n\
Gate: L'utilisateur valide les specs.",

        "architecture" => "\
Phase ARCHITECTURE\n\
Objectif: Concevoir l'architecture et le plan de tasks.\n\
FAIS:\n\
- Designer l'architecture technique\n\
- Creer le plan de tasks dans current-epic.md\n\
- Deleguer a l'Architect\n\
- Verifier le plan avec le Researcher\n\
NE FAIS PAS:\n\
- Ecrire du code\n\
- Creer des fichiers source\n\
- Commencer l'implementation\n\
Agent: Architect\n\
Gate: L'utilisateur valide l'architecture et le plan.",

        "implementation" => "\
Phase IMPLEMENTATION\n\
Objectif: Coder les tasks du plan.\n\
FAIS:\n\
- Implementer les tasks de current-epic.md une par une\n\
- Deleguer au Coder\n\
- Mettre a jour current-epic.md apres CHAQUE task (marquer DONE)\n\
- Suivre l'ordre du plan\n\
- DO serialiser les npm install : un seul agent a la fois pour eviter les conflits ERESOLVE\n\
NE FAIS PAS:\n\
- Sauter des tasks\n\
- Ajouter des features hors plan\n\
- Passer en review sans terminer toutes les tasks\n\
Agent: Coder\n\
Gate: Toutes les tasks sont DONE, l'utilisateur valide.",

        "review" => "\
Phase REVIEW\n\
Objectif: Reviser le code produit.\n\
FAIS:\n\
- Verifier la qualite, lisibilite, conventions\n\
- Deleguer au Reviewer\n\
- Signaler les problemes trouves\n\
NE FAIS PAS:\n\
- Ecrire de nouvelles features\n\
- Modifier l'architecture\n\
- Passer a la suite sans review complete\n\
Agent: Reviewer\n\
Gate: L'utilisateur valide que la review est complete.",

        "test" => "\
Phase TEST\n\
Objectif: Tester le code produit.\n\
FAIS:\n\
- Ecrire et executer les tests\n\
- Deleguer au Tester\n\
- Verifier les cas limites\n\
NE FAIS PAS:\n\
- Ecrire de nouvelles features\n\
- Modifier l'architecture\n\
Agent: Tester\n\
Gate: L'utilisateur valide que les tests passent.",

        "closure" => "\
Phase CLOSURE\n\
Objectif: Mettre a jour la memoire et fermer l'epic.\n\
FAIS:\n\
- Mettre a jour state.md, current-epic.md, ROADMAP.md\n\
- Resumer ce qui a ete fait\n\
NE FAIS PAS:\n\
- Coder quoi que ce soit\n\
- Ajouter des features\n\
- Ouvrir une nouvelle epic sans validation\n\
Gate: L'utilisateur valide la closure.",

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

const CODE_ALLOWED_PHASES: &[&str] = &["implementation", "review", "test"];

const GATE_ENFORCED_PHASES: &[&str] = &["implementation"];

const MEMORY_REMINDER: &str = "\
RAPPEL MEMOIRE (apres chaque task terminee):\
\n1. Mettre a jour current-epic.md IMMEDIATEMENT (marquer la task DONE dans le tableau)\
\n2. Ne PAS attendre la fin de session pour mettre a jour\
\n3. Le dashboard ZAOS lit current-epic.md en temps reel — les infos doivent etre a jour";

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

    // 3c. Memory reminder
    context.push_str(MEMORY_REMINDER);
    context.push_str("\n\n");

    // 3d-5. Load current-epic.md once for both gate check and head display
    let epic_path = project_dir.join(".memory").join("current-epic.md");
    let epic_content = fs::read_to_string(&epic_path).unwrap_or_default();

    // Gate enforcement reminder (pipeline mode, implementation phase, all tasks done)
    if let Some(ref st) = state {
        if st.mode == "pipeline" && st.phase == "implementation" && !st.gate_validated {
            if !epic_content.is_empty() && !has_active_tasks(&epic_content) {
                context.push_str(
                    "⛔ ATTENTE GATE — Toutes les tasks sont terminees. \
                     L'utilisateur n'a PAS encore valide le gate. \
                     Tu NE DOIS PAS avancer a la phase suivante. \
                     Resume ce qui a ete fait et demande a l'utilisateur de valider le gate.\n\n"
                );
            }
        }
    }

    // First 20 lines of current-epic.md (reuse already-loaded content)
    if !epic_content.is_empty() {
        let epic_head: String = epic_content.lines().take(20).collect::<Vec<_>>().join("\n");
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

    // Phase enforcement (pipeline mode only)
    let st = state.as_ref().unwrap(); // safe: we exited above if state is None
    if !CODE_ALLOWED_PHASES.contains(&st.phase.as_str()) {
        eprintln!(
            "BLOQUE: Phase actuelle = '{}'. L'ecriture de code (.rs/.ts/.tsx) \
             n'est autorisee qu'en phase implementation ou test. \
             Termine la phase {} d'abord.",
            st.phase, st.phase
        );
        process::exit(2);
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
    if !has_active_tasks(&epic_content) {
        // All tasks done but gate not validated → block (gate enforcement)
        if !st.gate_validated {
            eprintln!(
                "BLOQUE: Toutes les tasks sont terminees mais le gate n'est pas valide. \
                 Attends que l'utilisateur valide le gate avant de continuer. \
                 Ne passe PAS a la phase suivante sans validation."
            );
            process::exit(2);
        }
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
// B3b: enforce-gate — called on PreToolUse for Bash (gate enforcement)
// ---------------------------------------------------------------------------

fn cmd_enforce_gate() {
    let project_dir = get_project_dir();
    let state = load_state(&project_dir);

    // Only enforce in pipeline mode
    let st = match state {
        Some(ref s) if s.mode == "pipeline" => s,
        _ => process::exit(0), // free mode or no state → allow
    };

    // Only enforce gate in specific phases (implementation)
    if !GATE_ENFORCED_PHASES.contains(&st.phase.as_str()) {
        process::exit(0);
    }

    // If gate is validated → allow
    if st.gate_validated {
        process::exit(0);
    }

    // Check if all tasks are done in current-epic.md
    let epic_path = project_dir.join(".memory").join("current-epic.md");
    let epic_content = match fs::read_to_string(&epic_path) {
        Ok(c) => c,
        Err(_) => process::exit(0), // no epic file → allow
    };

    // If tasks are still active → allow (work in progress)
    if has_active_tasks(&epic_content) {
        process::exit(0);
    }

    // All tasks done + gate not validated → block
    eprintln!(
        "BLOQUE: Toutes les tasks sont terminees mais le gate n'est pas valide. \
         Attends que l'utilisateur valide le gate avant d'executer des commandes. \
         Ne passe PAS a la phase suivante sans validation."
    );
    process::exit(2);
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

    // 3b. Format reminder
    println!("{}\n", FORMAT_REMINDER);

    // 3c. Memory reminder
    println!("{}\n", MEMORY_REMINDER);

    // 3d. Gate enforcement reminder
    if let Some(ref st) = state {
        if st.mode == "pipeline" && st.phase == "implementation" && !st.gate_validated {
            let epic_for_gate = load_file_full(&project_dir.join(".memory").join("current-epic.md"));
            if !epic_for_gate.is_empty() && !has_active_tasks(&epic_for_gate) {
                println!(
                    "⛔ ATTENTE GATE — Toutes les tasks sont terminees. \
                     L'utilisateur n'a PAS encore valide le gate. \
                     Tu NE DOIS PAS avancer a la phase suivante.\n"
                );
            }
        }
    }

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
        "enforce-gate" => cmd_enforce_gate(),
        "on-compact" => cmd_on_compact(),
        "welcome" => cmd_welcome(),
        _ => {
            eprintln!("Usage: zaos-hooks <inject-context|block-code|enforce-gate|on-compact|welcome>");
            process::exit(1);
        }
    }
}
