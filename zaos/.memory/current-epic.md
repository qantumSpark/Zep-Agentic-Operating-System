# Epic active : Interactive Permission Approvals

> Milestone : 1 — Chat fonctionnel avec Claude Code CLI
> Date de debut : 2026-03-29
> Statut : VALIDATED ✅

## Objectif

Remplacer `--dangerously-skip-permissions` par des approbations interactives inline dans le chat. Passer du modele spawn-per-prompt a un process CLI long-lived avec communication bidirectionnelle via stdin/stdout.

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|---|---|---|---|
| 1 | Types Rust control_request/control_response | `events/types.rs` | done | + CliEvent::ControlRequest variant |
| 2 | Parser control_request | `events/parser.rs` | done | Auto via serde, test ajoute |
| 3 | Refactor SessionManager long-lived process | `session/manager.rs` | done | start_session + send_message + send_permission_response |
| 4 | Adapter commands.rs + main.rs | `commands.rs`, `main.rs` | done | respond_permission command |
| 5 | Types TypeScript | `types/events.ts` | done | ControlRequest, ControlResponse, Message.permissionRequest |
| 6 | permissionStore Zustand | `stores/permissionStore.ts` | done | Nouveau fichier |
| 7 | useStreaming control_request handler | `hooks/useStreaming.ts` | done | Feed permissionStore + chatStore |
| 8 | PermissionRequestBlock composant | `components/chat/PermissionRequestBlock.tsx` | done | Approve/Deny inline, amber theme |
| 9 | Integration MessageBubble + InputBar | `MessageBubble.tsx`, `InputBar.tsx` | done | Rendu conditionnel |

## Changements cles

- CLI flags: `--input-format stream-json --output-format stream-json --verbose --include-partial-messages --permission-prompt-tool stdio`
- Supprime: `-p <prompt>`, `--dangerously-skip-permissions`, `--resume` (gere via stdin)
- SessionManager stocke `child` + `stdin` au lieu de `child_pid`
- Nouveau IPC: `respond_permission(id, allow)`

## Validation

> Tested on 2026-03-30 — VALIDATED ✅

- Both **Approve** and **Deny** flows confirmed working in live app
- File creation via Approve confirmed (CLI executes the tool after approval)
- Deny rejection confirmed (CLI receives denial and skips execution)
- **Key fix discovered during testing:** `control_response` format needed `subtype: "success"` and double-nested `response.response` to match the CLI's Zod schema

## Prochaine action

COMPLETED — Epic validated and closed on 2026-03-30.
