# Etat courant ZAOS

> Derniere mise a jour : 2026-03-31
> ECRASE a chaque mise a jour. Max 50 lignes.

## Milestones

| # | Milestone | Statut | Epics |
|---|---|---|---|
| 1 | Chat fonctionnel avec Claude Code CLI | TERMINE | Chat complet, Interactive Permissions, Phase 1 Finition (6/6) |
| 2 | Dashboard temps reel | TERMINE | Stream A (4/4), Stream B (8/8), Stream C (4/4) + trace reference |
| 3 | Screenshots & visuels | EN COURS | 13/13 taches implementees, en attente test utilisateur |
| 4 | MCP Server zaos-ide | Non commence | Serveur local, Tools, .mcp.json |
| 5 | UX Polish | Non commence | Sessions, Metriques, Notifications |

## Epic active

Phase 3 — Screenshots & Visuels (13/13 taches implementees, test utilisateur en attente)

## Ce qui est fait

- **Stream A (Fondations Rust) TERMINE** : types.rs (10 types), index.rs (ScreenshotIndex + IterationIndex, atomic write, 10 tests), FileWatcher Screenshot category (500ms debounce, image filter), init.rs (.screenshots/)
- **Stream B (Orchestration) TERMINE** : CaptureAdapter trait, FilesystemAdapter, CliMcpAdapter (prompt formatting), ScreenshotOrchestrator (6 tests), 4 commandes IPC (get_screenshots, add_screenshot, delete_screenshot, request_capture)
- **Stream C (Frontend) TERMINE** : screenshotStore Zustand (8 actions, cap 100), types TS, listeners screenshot-new + iteration-update, ScreenshotGallery (grille live, zoom modal, capture, delete), IterationTracker (StatusFlow, badges, history)
- **Stream D (Comparaison) TERMINE** : ComparisonView (side-by-side + slider mode)

## Bugs connus

- `list_sessions` retourne toujours vide (priorite basse)
- tsconfig.node.json reference issue (pre-existant, non bloquant)
- Asset protocol scope pas encore configure (tauri.conf.json) — images peuvent ne pas s'afficher

## Technical debt

- Phase stringly-typed en Rust → devrait etre un enum
- `as any` casts dans useTauriEvents.ts (5 occurrences)
- Messages/actions arrays unbounded (pas de cap)
- Dual event listeners (useStreaming + useTauriEvents sur meme channel)
- Status indicator patterns dupliques dans 4+ composants dashboard
- Event names stringly-typed sans constantes partagees
- manager.rs stub encore present (devrait etre nettoye)

## Blocages

Aucun

## Prochaines priorites

1. Test utilisateur Phase 3 — valider galerie, watcher, zoom
2. Configurer asset protocol scope pour servir les images
3. Phase 4 : MCP Server zaos-ide
