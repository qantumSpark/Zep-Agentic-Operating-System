# Epic active : Phase 6 — Workflow Kit Integration

> Milestone : 6 — Workflow Kit natif
> Date de debut : 2026-03-31
> Statut : TERMINE

## Objectif

Integrer le workflow-kit (agents, hooks, rules) comme feature native de ZAOS. Au lancement dans un projet, l'app bootstrap tout le systeme. Les hooks sont un binaire Rust cross-platform. CRUD complet depuis le dashboard.

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| A1 | Config types .zaos/config.json | `deployer/config.rs` | DONE | Struct + read/write |
| A2 | Embed resources tauri.conf.json | `tauri.conf.json` | DONE | bundle.resources |
| A3 | Deployer sync engine | `deployer/sync.rs` | DONE | Hash compare, SyncOutcome enum |
| A4 | Deployer module + IPC commands | `deployer/mod.rs`, `commands.rs` | DONE | deploy, status, config |
| A5 | Bootstrap complet init.rs | `init.rs` | DONE | .zaos/, .claude/ |
| B1 | zaos-hooks binary scaffold | `bin/zaos_hooks.rs`, `Cargo.toml` | DONE | [[bin]] target |
| B2 | inject-context subcommand | meme fichier | DONE | JSON additionalContext |
| B3 | block-code subcommand | meme fichier | DONE | Exit 0 ou 2 |
| B4 | on-compact + welcome | meme fichier | DONE | Texte complet |
| C1 | workflowKitStore | `stores/workflowKitStore.ts` | DONE | agents, hooks, config |
| C2 | AgentsManager CRUD | `AgentsManager.tsx` | DONE | read_agent for edit |
| C3 | HooksManager | `HooksManager.tsx` | DONE | Shared Toggle component |
| C4 | RulesManager | `RulesManager.tsx` | DONE | Shared StatusDot + Toggle |
| C5 | Remplacer AgentsSection | `DashboardPanel.tsx` | DONE | Nouvelles sections |
| D1 | Auto-deploy dans start_session | `session/manager.rs` | DONE | Avant spawn CLI |
| D2 | Watcher .claude/ | `watchers/service.rs` | DONE | Filtered to agents/ |
| D3 | IPC agents CRUD | `commands.rs` | DONE | read/write/delete .md |

## Simplify Review Fixes

1. `invoke("create_agent")` → `invoke("save_agent")` (runtime bug)
2. Edit textarea fetches full content via `read_agent` (was truncating)
3. Deduplicated `newAgent` construction in handleCreate
4. `useTauriEvents` maps list_agents result properly (not raw cast)
5. Removed double deploy (init.rs no longer calls deployer::deploy)
6. Removed TOCTOU existence check in deployer/mod.rs
7. `sync_file` returns `SyncOutcome` enum + manifest fast-path
8. `list_agents` no longer sends full file content (lighter IPC)
9. Extracted shared `StatusDot` and `Toggle` to common/
