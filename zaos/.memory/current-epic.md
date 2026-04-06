# Epic active : switch_project fiable

> Milestone : 25 — Vague 1 Stabilisation du socle (E1)
> Statut : TERMINE

## Objectif

Corriger switch_project pour qu'il recharge correctement le workflow state, le permission_mode et le policy_profile du projet cible.

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| 1 | Appeler load_state() apres remplacement du WorkflowEngine | `commands.rs` | DONE | Bloc 6b ajoute |
| 2 | Sync permission_mode depuis le state charge | `commands.rs` | DONE | Hardcode "strict" supprime |
| 3 | Sync policy_profile depuis le state charge | `commands.rs` | DONE | Inclus dans load_state() |
| 4 | Verification cargo test + scenario switch | `commands.rs` | DONE | 160 tests OK |
| 5 | Review B1 — Reset permission_mode dans branche Err | `commands.rs` | DONE | 160 tests OK |
