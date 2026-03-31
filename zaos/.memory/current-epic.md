# Epic active : Phase 8 — Workflow Integration Bugs

> Milestone : 8 — Workflow fiable end-to-end
> Date de debut : 2026-03-31
> Statut : EN COURS

## Objectif

Corriger 3 bugs identifies lors du premier test reel sur un projet vierge : hooks affiches inactifs, format memoire non specifie (epic invisible dans le dashboard), et validate_gate deconnecte de Claude.

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| A1 | Brancher hooks_active dans le frontend | `App.tsx`, `useTauriEvents.ts` | A FAIRE | setHooks() apres get_workflow_kit_status |
| A2 | Deriver active par hook depuis hooks_active | `workflowKitStore.ts` | A FAIRE | Map hooks_active boolean sur chaque hook |
| B1 | Ajouter specs format memoire dans CLAUDE.md | `reference/CLAUDE.md` | A FAIRE | Template exact current-epic.md + state.md |
| B2 | Fixer templates init.rs | `init.rs` | A FAIRE | Headings compatibles parser (# Epic active, ## Objectif, ## Tasks) |
| B3 | Ajouter specs format dans inject-context | `zaos_hooks.rs` | A FAIRE | Rappeler le format dans le contexte injecte |
| C1 | Reset gate_validated dans next_phase | `engine.rs` | A FAIRE | gate_validated = false apres avancement |
| C2 | Envoyer message a Claude quand gate valide | `commands.rs` | A FAIRE | Auto-inject prompt via session.send_message |
| C3 | Feedback visuel apres validate_gate | `WorkflowSection.tsx` | A FAIRE | Toast ou indication de la nouvelle phase |

## Vagues

1. A1 + A2 (hooks display fix — frontend only)
2. B1 + B2 + B3 (format memoire — CLAUDE.md + init + hooks)
3. C1 + C2 + C3 (gate workflow — engine + commands + frontend)
