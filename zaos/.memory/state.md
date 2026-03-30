# Etat courant ZAOS

> Derniere mise a jour : 2026-03-30
> ECRASE a chaque mise a jour. Max 50 lignes.

## Milestones

| # | Milestone | Statut | Epics |
|---|---|---|---|
| 1 | Chat fonctionnel avec Claude Code CLI | En cours | Chat complet (VALIDATED), Interactive Permissions (VALIDATED) |
| 2 | Dashboard temps reel | Non commence | Actions feed, Agents, Pipeline, Memory |
| 3 | Screenshots & visuels | Non commence | Screenshot manager, Gallery, GoPeak |
| 4 | MCP Server zaos-ide | Non commence | Serveur local, Tools, .mcp.json |
| 5 | UX Polish | Non commence | Sessions, Metriques, Notifications |

## Epic active

Phase 1 — Finition (6 taches, Task 1-3 VALIDATED + 3 bugs fixes VALIDATED, 3 taches restantes)

## Ce qui est fait

- Backend Rust complet : Session Manager (long-lived process), Event Parser, Workflow Engine, 8 commandes IPC
- Frontend React en place : Chat (send/stream/display), Workflow (gate/mode), StatusBar, SplitPane
- Integration chat fonctionnelle : InputBar → CLI start_session → send_message via stdin → stream-json → useStreaming → affichage
- **Epic "Chat Complet" terminee** : tool_use blocks, ThinkingIndicator, interrupt reel, actionsStore nourri, streaming live
- **Epic "Interactive Permissions" VALIDATED 2026-03-30** : long-lived process model, control_request/control_response protocol, PermissionRequestBlock UI, permissionStore, respond_permission IPC command

## Bugs connus

- `list_sessions` retourne toujours vide (priorite basse)
- tsconfig.node.json reference issue (pre-existant, non bloquant)

## Bugs fixes (2026-03-30)

- Parse error `missing field tool_use_id` : UserContentBlock enum Rust + TS
- Text duplication streaming : seenBlockIds + addMessage/updateMessage
- Message disparait apres streaming : supprime branchement isUpdate pour text blocks

## Blocages

Aucun

## Prochaines priorites

1. Finir Phase 1 : 6 taches restantes (ThinkingIndicator, actionsStore, check_cli_auth, syntax highlighting, copy-to-clipboard, list_sessions)
2. Puis demarrer Phase 2 : dashboard temps reel
