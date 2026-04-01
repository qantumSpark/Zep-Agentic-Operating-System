# Epic active : Phase 11 — V1.5 Stabilisation (Audit Sprints)

> Milestone : 11 — V1.5 Stabilisation
> Statut : TERMINE

## Objectif

3 sprints issus d'un audit de code. Stabiliser la base technique, decoupler le runtime Claude, normaliser le modele d'evenements frontend. Principe : "Claude-first but agnostic-ready."

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| 1 | Path traversal prevention agent CRUD | `commands.rs` | DONE | validate_safe_name() |
| 2 | Screenshot path validation | `commands.rs`, `orchestrator.rs` | DONE | canonicalize + starts_with |
| 3 | CSP headers | `tauri.conf.json` | DONE | default-src 'self' |
| 4 | Asset protocol scope restriction | `tauri.conf.json` | DONE | .screenshots/** + $APPDATA/** |
| 5 | Retrait shell:allow-execute | `capabilities/default.json` | DONE | |
| 6 | Extensions bloquees elargies | `zaos_hooks.rs` | DONE | +.js, .jsx, .css |
| 7 | Remplacement as any | `useTauriEvents.ts` | DONE | Types corrects |
| 8 | Gate bypass prevention | `engine.rs` | DONE | validate_gate verifie gate_ready |
| 9 | Trait AgentRuntime | `runtime/mod.rs` | DONE | Send + Sync, async-trait |
| 10 | ClaudeRuntime implementation | `runtime/claude.rs` | DONE | Extrait de SessionManager |
| 11 | SessionManager thin wrapper | `session/manager.rs` | DONE | Box<dyn AgentRuntime> |
| 12 | ZaosEvent types | `types/zaosEvents.ts` | DONE | 15 types discriminated union |
| 13 | Mapper Claude → ZAOS | `adapters/claudeMapper.ts` | DONE | mapClaudeEvent() |
| 14 | Refactor useStreaming | `useStreaming.ts` | DONE | for/switch sur ZaosEvent |
| 15 | Refactor useTauriEvents | `useTauriEvents.ts` | DONE | Events normalises |
| 16 | Normaliser permissionStore | `permissionStore.ts` | DONE | ApprovalRequestedEvent |
| 17 | Normaliser PermissionRequestBlock | `PermissionRequestBlock.tsx` | DONE | Champs directs |
| 18 | Message.permissionRequest type | `events.ts` | DONE | ControlRequest → ApprovalRequestedEvent |
