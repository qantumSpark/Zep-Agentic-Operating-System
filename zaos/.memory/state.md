# Etat courant ZAOS

> Derniere mise a jour : 2026-03-31
> ECRASE a chaque mise a jour. Max 50 lignes.

## Milestones

| # | Milestone | Statut | Epics |
|---|---|---|---|
| 1 | Chat fonctionnel avec Claude Code CLI | TERMINE | Chat complet, Interactive Permissions, Phase 1 Finition (6/6) |
| 2 | Dashboard temps reel | TERMINE | Stream A (4/4), Stream B (8/8), Stream C (4/4) + trace reference |
| 3 | Screenshots & visuels | TERMINE | 13/13 taches implementees, test utilisateur valide |
| 4 | MCP Server zaos-ide | TERMINE | Serveur local, Tools, .mcp.json |
| 5 | UX Polish | EN COURS | 0/12 taches, 4 streams paralleles |

## Epic active

Phase 5 — UX Polish : Sessions, Metriques, Notifications, Raccourcis, Theme, Activity Feedback

## Ce qui est fait (Phase 5)

Rien encore — plan verifie, pret pour implementation.

## Bugs connus

- `list_sessions` retourne toujours vide (priorite basse — a investiguer en T1)
- tsconfig.node.json reference issue (pre-existant, non bloquant)
- Asset protocol scope pas encore configure (tauri.conf.json)

## Technical debt

- Phase stringly-typed en Rust → devrait etre un enum
- `as any` casts dans useTauriEvents.ts (5 occurrences)
- Messages/actions arrays unbounded (pas de cap)
- Event names stringly-typed sans constantes partagees

## Blocages

Aucun

## Prochaines priorites

1. T1-T4-T6-T8 en parallele (premiere vague)
2. T2-T5-T7-T9 en parallele (deuxieme vague)
3. T3-T10 en parallele (troisieme vague)
4. T11 puis T12 (sequential, Stream D fin)
