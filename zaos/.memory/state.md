# Etat courant ZAOS

> Derniere mise a jour : 2026-03-29
> ECRASE a chaque mise a jour. Max 50 lignes.

## Milestones

| # | Milestone | Statut | Epics |
|---|---|---|---|
| 1 | Chat fonctionnel avec Claude Code CLI | En cours | Chat complet, Streaming, Integration |
| 2 | Dashboard temps reel | Non commence | Actions feed, Agents, Pipeline, Memory |
| 3 | Screenshots & visuels | Non commence | Screenshot manager, Gallery, GoPeak |
| 4 | MCP Server zaos-ide | Non commence | Serveur local, Tools, .mcp.json |
| 5 | UX Polish | Non commence | Sessions, Metriques, Notifications |

## Epic active

Aucune — en attente de definition de la prochaine epic

## Ce qui est fait

- Backend Rust complet : Session Manager, Event Parser, Workflow Engine, 7 commandes IPC
- Frontend React en place : Chat (send/stream/display), Workflow (gate/mode), StatusBar, SplitPane
- Integration chat fonctionnelle : InputBar → CLI spawn → stream-json → useStreaming → affichage

## Bugs connus

- `--include-partial-messages` manquant au spawn CLI (streaming sous-optimal)
- `interrupt_session` ne tue pas le process CLI
- `list_sessions` retourne toujours vide

## Blocages

Aucun

## Prochaines priorites

1. Finir Phase 1 : tool_use blocks, ThinkingIndicator, interrupt, actionsStore
2. Demarrer Phase 2 : dashboard temps reel
