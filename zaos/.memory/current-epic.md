# Epic active : Dashboard Fixes (React/TS + Rust)

> Milestone : 12 — Test Run Baseline Fixes
> Statut : EN COURS

## Objectif

Corriger les findings #2, #3, #5, #6, #8 du test run baseline : propagation workflow state, gate_ready robuste, parser markdown, ToolCallStarted precoce, mapping agent par scoring.

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| 1.1 | Hydratation initiale workflow state au montage | `useTauriEvents.ts`, `workflowStore.ts` | DONE | invoke get_workflow_state au bootstrap |
| 1.2 | Reload workflow sur project-changed | `useTauriEvents.ts` | DONE | ajoute dans Promise.all |
| 1.3 | Tracing leger cote watcher | `watchers/service.rs` | DONE | log phase/gate_ready/gate_validated avant emit |
| 1.4 | Diagnostic mismatch temporaire | `useTauriEvents.ts` | DONE | log transitions de phase |
| 2.1 | Extraire logique metier gate_ready | `engine.rs` | DONE | methode try_mark_gate_ready_after_turn() |
| 2.2 | Deplacer condition depuis commands.rs | `commands.rs` | DONE | 14 lignes → 4 lignes |
| 2.3 | Exposer commande Tauri set_gate_ready | `commands.rs`, `main.rs` | DONE | fallback + debug |
| 2.4 | Tracing structure gate_ready | `engine.rs`, `commands.rs` | DONE | log raison skip |
| 2.5 | Tests unitaires gate_ready | `engine.rs` | DONE | 4 tests, 63 total pass |
| 3.1 | Corriger parse_md_table_row | `reader.rs` | DONE | retirer filter empty |
| 3.2 | Preserver colonnes vides legitimes | `reader.rs` | DONE | cellules bord |
| 3.3 | Tests regression parser | `reader.rs` | DONE | 6 cas |
| 4.1 | Emettre ToolCallStarted des ContentBlockStart | `mapper.rs` | DONE | emission precoce |
| 4.2 | Dedupliquer avec emission actuelle | `mapper.rs` | DONE | frontend seenBlockIds |
| 4.3 | Test idempotence | `mapper.rs` | DONE | 2 tests |
| 5.1 | Scoring mapAgentName | `agentsStore.ts` | DONE | remplacer first-match |
| 5.2 | Tie-break stable | `agentsStore.ts` | DONE | bonus subagentType +2 |
| 5.3 | Tests mapping | `agentsStore.ts` | DONE | JSDoc 5 cas documentes |
