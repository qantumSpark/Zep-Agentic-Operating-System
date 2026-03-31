# Epic active : Phase 3 — Screenshots & Visuels

> Milestone : 3 — Screenshots & visuels
> Date de debut : 2026-03-31
> Statut : EN COURS — implementation terminee, en attente de test utilisateur

## Objectif

Implementer une boucle iterative de capture/evaluation/correction de screenshots. Le Rust orchestre via des adaptateurs (filesystem passif + CLI-MCP actif), le FileWatcher detecte les fichiers, et le frontend affiche la galerie live avec suivi des iterations.

## Architecture

```
ScreenshotOrchestrator (choisit l'adaptateur selon ProjectType)
  ├── FilesystemAdapter (passif: attend le fichier)
  └── CliMcpAdapter (actif: envoie prompt au CLI → CLI utilise MCP)
          ↓
FileWatcher (.screenshots/) → index.json → emit "screenshot-new" → screenshotStore → Gallery
```

Boucle iterative : Capture → Evaluation → Rapport → Correction → Re-capture → Comparaison

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| A1 | Types enrichis : Screenshot, Iteration, ProjectType, CaptureContext | `screenshots/types.rs` | DONE | 10 types, serde camelCase |
| A2 | Index JSON persistant (load/save/add/get_by) | `screenshots/index.rs` | DONE | Atomic write, 10 tests |
| A3 | Etendre FileWatcher avec WatchCategory::Screenshot | `watchers/service.rs` | DONE | 500ms debounce, image filter |
| A4 | Creer `.screenshots/` + `index.json` au demarrage | `init.rs` | DONE | OpenOptions::create_new |
| B1 | Trait CaptureAdapter + FilesystemAdapter (passif) | `screenshots/adapters.rs` | DONE | async_trait, 2 tests |
| B2 | CliMcpAdapter — prompt formate au CLI via SessionManager | `screenshots/adapters.rs` | DONE | 3 tests prompt formatting |
| B3 | ScreenshotOrchestrator — remplace le stub manager.rs | `screenshots/orchestrator.rs` | DONE | 6 tests |
| B4 | 4 commandes IPC + enregistrement main.rs | `commands.rs` + `main.rs` | DONE | get_screenshots, add_screenshot, delete_screenshot, request_capture |
| C1 | screenshotStore Zustand + types TS | `screenshotStore.ts` + `types/screenshots.ts` | DONE | 8 actions, cap 100 |
| C2 | Listeners screenshot-new + iteration-update | `useTauriEvents.ts` + `App.tsx` | DONE | + initial load |
| C3 | Reecrire ScreenshotGallery (grille live, zoom, capture, delete) | `ScreenshotGallery.tsx` | DONE | convertFileSrc, zoom modal, Escape |
| C4 | IterationTracker — section dashboard boucle iterative | `IterationTracker.tsx` + `DashboardPanel.tsx` | DONE | StatusFlow, badges, history |
| D1 | ComparisonView — before/after cote a cote avec slider | `ComparisonView.tsx` | DONE | Side-by-side + slider mode |

## Streams de travail

- **Stream A (Fondations Rust):** A1 + A2 + A4 (parallele) → A3 — TERMINE
- **Stream B (Orchestration):** B1 → B2 → B3 → B4 — TERMINE
- **Stream C (Frontend):** C1 → C2 → C3 → C4 — TERMINE
- **Stream D (Comparaison):** D1 — TERMINE

## Tests

22 tests Rust passent (index: 10, adapters: 5, orchestrator: 6, manager: 1)

## Defere a Phase 3.5

- Diff visuel pixel-a-pixel
- Detection automatique ProjectType
- Lighthouse scoring par capture
- Annotations sur screenshots
- Cleanup automatique par age
- Adaptateurs mobile (ADB, iOS Simulator)
- Multi-viewport
- Asset protocol scope (tauri.conf.json + capabilities) — a tester manuellement
