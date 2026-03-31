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
| 7 | Project Portability & Workflow Init | EN COURS | 0/15 taches, 4 streams |

## Epic active

Phase 7 — Project Portability : embedded content, project picker, workflow init

## Ce qui est fait (Phase 7)

Rien encore — plan valide, pret pour implementation.

## Bugs connus

- `list_sessions` retourne toujours vide (priorite basse)
- tsconfig.node.json reference issue (pre-existant, non bloquant)

## Technical debt

- Phase stringly-typed en Rust
- `as any` casts dans useTauriEvents.ts
- Event names stringly-typed

## Blocages

Aucun

## Prochaines priorites

1. Vague 1 : A1-A4 (embedded content — elimine dependance reference/)
2. Vague 2 : B1-B6 (dynamic project dir)
3. Vague 3 : C1-C6 (project picker UI)
4. Vague 4 : D1-D3 (workflow init)
