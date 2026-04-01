# Etat courant ZAOS

> Derniere mise a jour : 2026-03-31
> ECRASE a chaque mise a jour. Max 50 lignes.

## Milestones

| # | Milestone | Statut | Epics |
|---|---|---|---|
| 1 | Chat fonctionnel avec Claude Code CLI | TERMINE | 47 taches |
| 2 | Dashboard temps reel | TERMINE | 13 taches |
| 3 | Screenshots & visuels | TERMINE | 13 taches |
| 4 | MCP Server zaos-ide | TERMINE | 7 taches |
| 5 | UX Polish | TERMINE | 12 taches |
| 6 | Workflow Kit Integration | TERMINE | 17 taches |
| 7 | Project Portability & Workflow Init | TERMINE | 15 taches + CLAUDE.md deploy |
| 8 | Workflow Integration Bugs | TERMINE | 8 taches, 3 vagues |

## Epic active

Phase 8 — Workflow Integration Bugs : hooks display, format memoire, gate workflow

## Ce qui est fait (Phase 8)

- Vague 1 : hooks display fix (A1 + A2) + simplify applyKitStatus
- Vague 2 : format memoire (B1 + B2 + B3) — CLAUDE.md + init.rs + hooks
- Vague 3 : gate workflow (C1 + C2 + C3) — reset + notify + feedback

## Bugs connus

- `list_sessions` retourne toujours vide (priorite basse)
- `block-code` ne reconnait pas `A FAIRE` comme statut actif (pre-existant)
- tsconfig.node.json reference issue (pre-existant, non bloquant)

## Technical debt

- Phase stringly-typed en Rust
- `as any` casts dans useTauriEvents.ts
- Event names stringly-typed
- `welcome` hook command = dead code

## Blocages

Aucun

## Prochaines priorites

Phase 8 terminee. Prochaine priorite a definir par l'utilisateur.
