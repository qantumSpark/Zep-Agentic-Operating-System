# Epic active : Phase 9.4.1+9.5.1 — Permissions UX + Coordination Agents

> Milestone : 9 — Field-Tested Corrections
> Statut : TERMINE

## Objectif

Toggle "Accept Edits" pour auto-approuver Write/Edit/WebSearch/WebFetch sans prompt. Serialiser les npm installs entre agents paralleles.

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| 1 | permission_mode dans WorkflowState | `state.rs` | DONE | serde default "strict" |
| 2 | Arc permission_mode dans AppState | `commands.rs` | DONE | Runtime cache + sync startup |
| 3 | IPC set_permission_mode | `commands.rs`, `main.rs` | DONE | Valide strict/accept-edits, persist via engine |
| 4 | Auto-approval interceptor | `commands.rs` | DONE | ControlRequest interception, skip emit |
| 5 | Frontend permissionMode store + sync | `workflowStore.ts` | DONE | BackendWorkflowPayload + setFullState |
| 6 | Toggle Accept Edits UI | `WorkflowSection.tsx` | DONE | togglePermissionMode, green/zinc styling |
| 7 | Regle npm serialization | `reference/CLAUDE.md` | DONE | Regle #6 non-negociable |
| 8 | Rappel npm inject-context | `zaos_hooks.rs` | DONE | Phase implementation DO list |
