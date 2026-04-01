# Epic active : Phase 10 — Test Run #2 Fixes

> Milestone : 10 — Test Run #2 Fixes
> Statut : TERMINE

## Objectif

Corriger les 8 findings du test run #2 : hooks trop restrictifs (review/test bloques), gate desync + multi-clic, dashboard empty state, delegation fantome. Ordre : 10.1 (hooks) → 10.2 (gate) → 10.3 (dashboard) → 10.4 (delegation).

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| 1 | Ajouter "review" a CODE_ALLOWED_PHASES pour autoriser Write/Edit en review | `zaos_hooks.rs` | DONE | L229 |
| 2 | Ajouter GATE_ENFORCED_PHASES et restreindre enforce-gate a implementation seule | `zaos_hooks.rs` | DONE | Nouvelle const, skip si phase pas dans list |
| 3 | Corriger has_active_tasks pour retourner true quand 0 data rows | `zaos_hooks.rs` | DONE | 0 data rows = plan pas cree = actif |
| 4 | Restreindre warning ATTENTE GATE a phase implementation dans inject-context et on-compact | `zaos_hooks.rs` | DONE | L280 et L491 |
| 5 | Ajouter champ gate_ready: bool au struct WorkflowState avec serde default | `state.rs` | DONE | #[serde(default)] |
| 6 | Reset gate_ready dans validate_gate, next_phase, start_epic, set_phase + set_gate_ready | `engine.rs` | DONE | 4 methods + 1 nouvelle |
| 7 | Ajouter gate_ready a WorkflowStateResponse et get_workflow_state | `commands.rs` | DONE | Struct + mapping |
| 8 | Detecter fin de turn reussie et set gate_ready = true dans event forwarder | `commands.rs` | DONE | CliEvent::Result success en pipeline |
| 9 | Reset gate_ready = false au debut de send_prompt | `commands.rs` | DONE | Avant send_message |
| 10 | Ajouter gate_ready au struct WorkflowState local du hook binary | `zaos_hooks.rs` | DONE | Compat deserialization |
| 11 | Ajouter gateReady au store + BackendWorkflowPayload et setFullState | `workflowStore.ts` | DONE | Map gate_ready |
| 12 | Bouton gate disabled quand !gateReady, anti-double-clic, styles conditionnels | `WorkflowSection.tsx` | DONE | Gris disabled, vert ready |
| 13 | Afficher "Plan en attente..." quand 0 tasks, strip compteur du activeEpic | `MemorySection.tsx` | DONE | Placeholder + regex strip |
| 14 | Fallback result event : marquer delegations RUNNING comme completed | `useTauriEvents.ts` | DONE | Cleanup au result event |
| 15 | Style "stale" pour delegations RUNNING > 10 min | `UnifiedAgentsSection.tsx` | DONE | Opacity + badge stale |
