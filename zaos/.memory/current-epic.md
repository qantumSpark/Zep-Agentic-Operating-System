# Epic active : Phase 8 — Workflow Integration Bugs

> Milestone : 8 — Workflow fiable end-to-end
> Date de debut : 2026-03-31
> Statut : TERMINE

## Objectif

Corriger 3 bugs identifies lors du premier test reel sur un projet vierge : hooks affiches inactifs, format memoire non specifie (epic invisible dans le dashboard), et validate_gate deconnecte de Claude.

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| A1 | Brancher hooks_active dans le frontend | `App.tsx`, `useTauriEvents.ts` | DONE | setHooks() apres get_workflow_kit_status |
| A2 | Deriver active par hook depuis hooks_active | `workflowKitStore.ts` | DONE | Supprime welcome dead code, garde 3 hooks |
| B1 | Ajouter specs format memoire dans CLAUDE.md | `reference/CLAUDE.md` | DONE | Section STRICT avec templates exacts |
| B2 | Fixer templates init.rs | `init.rs` | DONE | Headings compatibles parser |
| B3 | Ajouter specs format dans inject-context | `zaos_hooks.rs` | DONE | FORMAT_REMINDER injecte a chaque prompt |
| C1 | Reset gate_validated dans next_phase | `engine.rs` | DONE | gate_validated = false avant set_phase |
| C2 | Envoyer message a Claude quand gate valide | `commands.rs` | DONE | send_message apres drop workflow lock |
| C3 | Feedback visuel apres validate_gate | `WorkflowSection.tsx` | DONE | Message temporaire 2s "Phase avancee a..." |

## Vagues

1. A1 + A2 (hooks display fix — frontend only)
2. B1 + B2 + B3 (format memoire — CLAUDE.md + init + hooks)
3. C1 + C2 + C3 (gate workflow — engine + commands + frontend)
