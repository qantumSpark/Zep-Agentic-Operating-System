# Etat courant ZAOS

> Derniere mise a jour : 2026-04-01
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
| 7 | Project Portability & Workflow Init | TERMINE | 15 taches |
| 8 | Workflow Integration Bugs | TERMINE | 8 taches |
| 9 | Field-Tested Corrections | A FAIRE | ~28 taches, 5 milestones, 8 epics |

## Epic active

Epic 9.1.2 + 9.1.3 TERMINE (5/5 tasks). Prochains : 9.2.1 (agents unifies) + 9.3.1 (dashboard temps reel).

## Bugs connus

- `list_sessions` retourne toujours vide (priorite basse)
- tsconfig.node.json reference issue (pre-existant, non bloquant)

## Technical debt

- Phase stringly-typed en Rust
- `as any` casts dans useTauriEvents.ts
- Event names stringly-typed
- `welcome` hook command = dead code

## Blocages

Aucun

## Prochaines priorites

Phase 9 — Field-Tested Corrections (17 findings, voir `.memory/test-findings.md`)
Sprint 1 : 9.1.1 + 9.1.4 (hooks) TERMINE
Sprint 2 : 9.1.2 + 9.1.3 (pipeline + gate) TERMINE
Sprint 3 : 9.2.1 (agents unifies) || 9.3.1 (parser)
Sprint 4 : 9.4.1 (permissions) || 9.5.1 (coordination)
