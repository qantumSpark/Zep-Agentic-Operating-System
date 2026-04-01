# Epic active : Phase 9.1 — Pipeline Mecanique (Hooks)

> Milestone : 9 — Field-Tested Corrections
> Statut : TERMINE

## Objectif

Rendre le pipeline mecanique : block-code bloque le code hors implementation/test, inject-context donne des DO/DON'T explicites par phase, rappel memoire apres chaque task.

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| 1 | Ajouter constante CODE_ALLOWED_PHASES | `zaos_hooks.rs` | DONE | ["implementation", "test"] |
| 2 | Check phase dans cmd_block_code | `zaos_hooks.rs` | DONE | exit 2 si phase hors liste en mode pipeline |
| 3 | Rewrite get_phase_instructions DO/DON'T | `zaos_hooks.rs` | DONE | 8 phases avec objectif, FAIS, NE FAIS PAS, agent, gate |
| 4 | Ajouter constante MEMORY_REMINDER | `zaos_hooks.rs` | DONE | Rappel maj current-epic.md apres chaque task |
| 5 | Injecter MEMORY_REMINDER dans inject-context | `zaos_hooks.rs` | DONE | Apres FORMAT_REMINDER |
| 6 | Injecter MEMORY_REMINDER dans on-compact | `zaos_hooks.rs` | DONE | Apres NON_NEGOTIABLE_RULES + FORMAT_REMINDER |
