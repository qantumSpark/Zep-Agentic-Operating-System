# Etat courant ZAOS

> Derniere mise a jour : 2026-03-30
> ECRASE a chaque mise a jour. Max 50 lignes.

## Milestones

| # | Milestone | Statut | Epics |
|---|---|---|---|
| 1 | Chat fonctionnel avec Claude Code CLI | TERMINE | Chat complet, Interactive Permissions, Phase 1 Finition (6/6) |
| 2 | Dashboard temps reel | EN COURS | Stream A (4/4), Stream B (8/8), Stream C (0/4) |
| 3 | Screenshots & visuels | Non commence | Screenshot manager, Gallery, GoPeak |
| 4 | MCP Server zaos-ide | Non commence | Serveur local, Tools, .mcp.json |
| 5 | UX Polish | Non commence | Sessions, Metriques, Notifications |

## Epic active

Phase 2 — Dashboard Temps Reel EN COURS (13/17 taches — Streams A+B termines)

## Ce qui est fait

- **Stream A (File Watchers + Pipeline) TERMINE** : FileWatcherService (notify, debounce 300ms), workflow-change emit, pipelineProgress live
- **Stream B (Memory Reader) TERMINE** : Parsers (INDEX.md, state.md, current-epic.md, 34 tests), get_memory_state IPC, memoryStore, MemorySection dashboard, memory-change watcher
- Infrastructure : init.rs (ensure project dirs), Vite ignore .workflow/.memory, project_dir fix

## Bugs connus

- `list_sessions` retourne toujours vide (priorite basse)
- tsconfig.node.json reference issue (pre-existant, non bloquant)

## Technical debt

- Phase stringly-typed en Rust → devrait etre un enum
- Epic type mismatch Rust (String) vs TS (Epic object avec description/startTime vides)
- `as any` casts dans useTauriEvents.ts (5 occurrences)
- Messages/actions arrays unbounded (pas de cap)
- Dual event listeners (useStreaming + useTauriEvents sur meme channel)
- Status indicator patterns dupliques dans 4 composants dashboard (MemorySection, ActionsFeed, AgentsSection, PipelineSection)
- Event names stringly-typed eparpilles ("workflow-change", "memory-change", etc.) — pas de constantes partagees
- BackendWorkflowPayload.phase est string malgre enum Phase existant — cast unsafe

## Blocages

Aucun

## Prochaines priorites

1. Stream C : Agents tracking (tasks 13-15, 17)
