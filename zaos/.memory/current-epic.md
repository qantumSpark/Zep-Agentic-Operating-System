# Epic active : Sync coherente workflow/runtime/permission

> Milestone : 25 — Vague 1 Stabilisation du socle (E3)
> Statut : TERMINE

## Objectif

Quand un processus externe modifie state.json (ex: hooks Claude), le cache permission_mode dans AppState doit etre sync automatiquement via le watcher.

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| 1 | Ajouter parametre permission_mode a start() | `watchers/service.rs` | DONE | Arc RwLock String |
| 2 | Sync permission_mode apres load_state dans handler Workflow | `watchers/service.rs` | DONE | Ecriture dans l'Arc |
| 3 | Passer permission_mode dans les appelants de start() | `main.rs`, `commands.rs` | DONE | 2 call sites adaptes |
| 4 | Verification build + tests | - | DONE | 160 tests OK, 0 nouveau warning |
| 5 | Review S1 — Deplacer permission_mode.write hors du engine lock | `commands.rs` | DONE | 160 tests OK |
