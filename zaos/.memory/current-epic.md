# Epic active : Phase 9.1.2+9.1.3 — Brainstorming Pipeline + Gate Enforcement

> Milestone : 9 — Field-Tested Corrections
> Statut : TERMINE

## Objectif

Guider la transition brainstorming → pipeline avec un parcours complet dans CLAUDE.md, et rendre les gates mecaniques via hooks (block-code, enforce-gate, inject-context).

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| 1 | Section "Parcours complet" dans CLAUDE.md | `reference/CLAUDE.md` | DONE | 6 etapes + exemple pushback |
| 2 | Enrichir instructions idle | `zaos_hooks.rs` | DONE | Brainstorming structure, demande explicite pipeline |
| 3 | Gate check dans block-code | `zaos_hooks.rs` | DONE | All tasks DONE + gate=false → exit 2 |
| 4 | Gate reminder dans inject-context | `zaos_hooks.rs` | DONE | Warning seulement quand toutes tasks DONE |
| 5 | Hook enforce-gate pour Bash | `zaos_hooks.rs`, `sync.rs`, `workflowKitStore.ts` | DONE | PreToolUse Bash + registration + frontend |
