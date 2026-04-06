// Project directory initialization
// Ensures .workflow/, .memory/, and .screenshots/ directories exist with default files at startup.

use crate::runtime::paths::{RuntimeKind, RuntimePaths};
use crate::workflow::state::WorkflowState;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

/// Ensure project directories and default files exist.
/// Called before AppState creation so watchers and engines have valid paths.
/// Uses std::fs (not tokio) since this runs before the async runtime.
pub fn ensure_project_dirs(project_dir: &Path) {
    let workflow_dir = project_dir.join(".workflow");
    let memory_dir = project_dir.join(".memory");

    // --- .workflow/ ---
    ensure_dir(&workflow_dir);
    ensure_file(
        &workflow_dir.join("state.json"),
        || {
            let state = WorkflowState::default();
            serde_json::to_string_pretty(&state).unwrap_or_else(|e| {
                tracing::warn!("Failed to serialize default WorkflowState: {}", e);
                "{}".to_string()
            })
        },
    );

    // --- .memory/ ---
    ensure_dir(&memory_dir);
    ensure_file(
        &memory_dir.join("INDEX.md"),
        || {
            concat!(
                "# Memory Index\n",
                "\n",
                "Central index of project memory files.\n",
                "\n",
                "## Entries\n",
                "\n",
                "- [State](state.md) — Current session state\n",
                "- [Current Epic](current-epic.md) — Active epic details\n",
                "- [Product Brief](product-brief.md) — Vision, public, contraintes, definition de succes\n",
                "- [Experience Goals](experience-goals.md) — Qualites UX cibles et anti-patterns\n",
                "- [Acceptance Checks](acceptance-checks.md) — Criteres de validation produit\n",
                "- [Release Readiness](release-readiness.md) — Etat de preparation a la livraison\n",
                "- [Session Insights](session-insights.md) — Resume actionnable de la derniere session\n",
            )
            .to_string()
        },
    );
    ensure_file(
        &memory_dir.join("state.md"),
        || {
            concat!(
                "# Etat courant\n",
                "\n",
                "## Milestones\n",
                "\n",
                "| # | Milestone | Statut | Epics |\n",
                "|---|-----------|--------|-------|\n",
                "\n",
                "## Epic active\n",
                "\n",
                "_Aucun epic en cours._\n",
                "\n",
                "## Blocages\n",
                "\n",
                "Aucun\n",
            )
            .to_string()
        },
    );
    ensure_file(
        &memory_dir.join("current-epic.md"),
        || {
            concat!(
                "# Epic active : _aucun_\n",
                "\n",
                "> Statut : IDLE\n",
                "\n",
                "## Objectif\n",
                "\n",
                "_Pas d'epic en cours._\n",
                "\n",
                "## Tasks\n",
                "\n",
                "| # | Task | Fichier(s) | Statut | Notes |\n",
                "|---|------|-----------|--------|-------|\n",
                "\n",
                "_En attente du plan._\n",
            )
            .to_string()
        },
    );
    ensure_file(
        &memory_dir.join("product-brief.md"),
        || {
            concat!(
                "# Product Brief\n",
                "\n",
                "## Vision\n",
                "\n",
                "_Ce qu'on construit en une phrase._\n",
                "\n",
                "## Pour qui\n",
                "\n",
                "_Public cible, contexte d'usage._\n",
                "\n",
                "## Pourquoi\n",
                "\n",
                "_Probleme resolu, valeur apportee._\n",
                "\n",
                "## Contraintes\n",
                "\n",
                "- _contrainte 1_\n",
                "\n",
                "## Hors-scope\n",
                "\n",
                "- _element explicitement exclu_\n",
                "\n",
                "## Definition de succes\n",
                "\n",
                "_Comment on sait que le produit a reussi._\n",
            )
            .to_string()
        },
    );
    ensure_file(
        &memory_dir.join("experience-goals.md"),
        || {
            concat!(
                "# Experience Goals\n",
                "\n",
                "## Qualites cibles\n",
                "\n",
                "| # | Qualite | Critere | Priorite |\n",
                "|---|---------|---------|----------|\n",
                "\n",
                "## Standards UX\n",
                "\n",
                "- _standard 1_\n",
                "\n",
                "## Anti-patterns\n",
                "\n",
                "- _ce qu'on veut eviter_\n",
            )
            .to_string()
        },
    );
    ensure_file(
        &memory_dir.join("acceptance-checks.md"),
        || {
            concat!(
                "# Acceptance Checks\n",
                "\n",
                "## Criteres\n",
                "\n",
                "| # | Check | Statut | Notes |\n",
                "|---|-------|--------|-------|\n",
                "\n",
                "## Validations manuelles\n",
                "\n",
                "- _validation 1_\n",
            )
            .to_string()
        },
    );
    ensure_file(
        &memory_dir.join("release-readiness.md"),
        || {
            concat!(
                "# Release Readiness\n",
                "\n",
                "## Etat general\n",
                "\n",
                "_Resume en 1-2 lignes._\n",
                "\n",
                "## Checklist\n",
                "\n",
                "| # | Item | Statut | Bloquant | Notes |\n",
                "|---|------|--------|----------|-------|\n",
                "\n",
                "## Risques ouverts\n",
                "\n",
                "- _risque 1_\n",
            )
            .to_string()
        },
    );
    ensure_file(
        &memory_dir.join("session-insights.md"),
        || {
            concat!(
                "# Session Insights\n",
                "\n",
                "> Date: YYYY-MM-DD\n",
                "> Epic: _aucun_\n",
                "> Phase: _inconnue_\n",
                "> Session-ID: _none_\n",
                "> Duration: 0s\n",
                "> Tokens: 0 in / 0 out\n",
                "> Agents: _aucun_\n",
                "> Suggested-Persona:\n",
                "\n",
                "## Decisions prises\n",
                "\n",
                "- _decision 1_\n",
                "\n",
                "## Ce qu'on a appris\n",
                "\n",
                "- _apprentissage 1_\n",
                "\n",
                "## Risques et points ouverts\n",
                "\n",
                "- _risque 1_\n",
                "\n",
                "## Prochaines validations\n",
                "\n",
                "- _validation 1_\n",
            )
            .to_string()
        },
    );

    // --- .memory/sessions/ ---
    ensure_dir(&memory_dir.join("sessions"));

    // --- .screenshots/ ---
    let screenshots_dir = project_dir.join(".screenshots");
    ensure_dir(&screenshots_dir);
    ensure_file(
        &screenshots_dir.join("index.json"),
        || "[]".to_string(),
    );

    // --- .zaos/ ---
    let zaos_dir = project_dir.join(".zaos");
    ensure_dir(&zaos_dir);
    ensure_file(
        &zaos_dir.join("config.json"),
        || {
            let config = crate::deployer::config::WorkflowKitConfig::default();
            serde_json::to_string_pretty(&config).unwrap_or_else(|_| "{}".to_string())
        },
    );

    // --- runtime directories (e.g. .claude/) ---
    let runtime_paths = RuntimePaths::for_kind(project_dir, &RuntimeKind::default());
    ensure_dir(&runtime_paths.base_dir);
    ensure_dir(&runtime_paths.agents_dir);
    ensure_dir(&runtime_paths.rules_dir);

    // --- Deploy workflow kit (agents, rules, settings) ---
    match crate::deployer::deploy(project_dir) {
        Ok(report) => {
            if !report.created.is_empty() || !report.updated.is_empty() {
                tracing::info!(
                    "Workflow kit deployed: {} created, {} updated",
                    report.created.len(),
                    report.updated.len()
                );
            }
        }
        Err(e) => {
            tracing::warn!("Workflow kit deploy failed (non-fatal): {}", e);
        }
    }

    tracing::info!("Project directories initialized");
}

/// Create a directory (no-op if it already exists).
fn ensure_dir(path: &Path) {
    if let Err(e) = fs::create_dir_all(path) {
        tracing::warn!("Failed to create directory {:?}: {}", path, e);
    }
}

/// Atomically create a file with default content if it doesn't already exist.
/// Uses `create_new(true)` to avoid TOCTOU races.
/// The content is produced lazily via a closure so we only build it when needed.
fn ensure_file<F: FnOnce() -> String>(path: &Path, content_fn: F) {
    match OpenOptions::new().create_new(true).write(true).open(path) {
        Ok(mut file) => {
            let content = content_fn();
            if let Err(e) = file.write_all(content.as_bytes()) {
                tracing::warn!("Failed to write default content to {:?}: {}", path, e);
            } else {
                tracing::info!("Created default file {:?}", path);
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            // File already exists — nothing to do
        }
        Err(e) => {
            tracing::warn!("Failed to create file {:?}: {}", path, e);
        }
    }
}
