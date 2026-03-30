# Epic active : Phase 2 — Dashboard Temps Reel

> Milestone : 2 — Dashboard temps reel
> Date de debut : 2026-03-30
> Statut : EN COURS

## Objectif

Rendre le dashboard ZAOS vivant : remplacer toutes les donnees statiques/hardcodees par des informations temps reel provenant (a) des events CLI deja en place, (b) de file watchers sur `.workflow/state.json` et `.memory/`, et (c) d'une nouvelle commande IPC pour l'etat memoire.

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| 1 | Creer module `watchers/` Rust avec FileWatcherService (notify crate, debounce 300ms) | `watchers/mod.rs`, `watchers/service.rs` | VALIDATED | notify v6, RecommendedWatcher + mpsc bridge, debounce 300ms, Windows path fix |
| 2 | Enregistrer FileWatcherService dans AppState et spawn au setup Tauri | `commands.rs`, `main.rs` | VALIDATED | Arc<FileWatcherService> dans AppState. start(app_handle) dans setup closure |
| 3 | Emettre `workflow-change` depuis watcher quand state.json change | `watchers/service.rs` | VALIDATED | tokio::fs::read_to_string → serde parse → app.emit("workflow-change", &state) |
| 4 | Mettre a jour pipelineProgress dans workflowStore depuis workflow-change | `useTauriEvents.ts`, `workflowStore.ts` | VALIDATED | setFullState() + derivePipelineProgress(). Vite ignore fix |
| 5 | Implementer read_index() — parser INDEX.md | `memory/reader.rs` | VALIDATED | MemoryIndexEntry/Section structs, section grouping, 12 tests |
| 6 | Implementer read_state() — parser state.md milestones, epic, blocages | `memory/reader.rs` | VALIDATED | MilestoneEntry, state-machine parser, 12 tests |
| 7 | Implementer read_current_epic() — parser current-epic.md tasks table | `memory/reader.rs` | VALIDATED | EpicTask/CurrentEpic structs, backtick stripping, 4 tests |
| 8 | Creer commande IPC get_memory_state | `commands.rs`, `main.rs` | VALIDATED | MemoryStateResponse, graceful degradation, registered in invoke_handler |
| 9 | Creer memoryStore Zustand | `memoryStore.ts` | VALIDATED | 6 interfaces TS, applyMemoryResponse helper, setMemoryState/updateFromWatcher/reset |
| 10 | Emettre memory-change depuis watcher quand .memory/ change | `watchers/service.rs` | VALIDATED | MemoryReader + MemoryStateResponse, graceful error handling |
| 11 | Ajouter listener memory-change dans useTauriEvents | `useTauriEvents.ts` | VALIDATED | Same pattern as workflow-change → memoryStore.updateFromWatcher |
| 12 | Creer MemorySection.tsx dans le dashboard | `MemorySection.tsx`, `DashboardPanel.tsx` | VALIDATED | Milestones, epic, tasks table, blocages avec StatusBadge |
| 13 | Creer agentsStore Zustand | `agentsStore.ts` | VALIDATED | DelegationEntry, availableAgents, activeAgent, delegations, 5 actions |
| 14 | Detecter delegation agent dans useStreaming (tool_use "Agent" + parent_tool_use_id) | `useStreaming.ts` | VALIDATED | system/init → setAvailableAgents, tool_use Agent → addDelegation, tool_result → completeDelegation |
| 15 | Remplacer AgentsSection hardcodee par donnees live agentsStore | `AgentsSection.tsx` | VALIDATED | Live agents list, delegation history, status badges, duration |
| 16 | Appeler get_memory_state au startup pour etat initial | `App.tsx` | VALIDATED | invoke("get_memory_state") → memoryStore.setMemoryState |
| 17 | Capturer trace reference avec delegation Task et valider detection | `reference/traces/` | A FAIRE | Test manuel a faire par utilisateur : lancer app, declencher delegation, sauver trace |

## Streams de travail

- **Stream A (Watchers + Pipeline):** 1 → 2 → 3 → 4
- **Stream B (Memory):** 5,6,7 (parallel) → 8 + 9 (parallel) → 10 → 11 → 12 + 16
- **Stream C (Agents):** 13 → 14 → 15 + 17

## Workflow par tache

1. Agent Architect → plan d'implementation
2. Agent Codeur → implementation
3. Agent Reviewer → review + tests
4. Retour utilisateur → test manuel + validation
5. Mise a jour memoire → tache suivante
