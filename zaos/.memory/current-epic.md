# Epic active : Post-V1.5 — Preparation Codex

> Milestone : 19 — Post-V1.5 — Preparation Codex
> Statut : TERMINE

## Objectif

Preparer ZAOS a accueillir un futur second runtime (Codex) en auditant les zones Claude-specific, ajoutant un runtime registry minimal, et documentant ce qui reste a faire.

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| 1 | Creer RuntimeKind enum + RuntimePaths struct | `runtime/paths.rs`, `runtime/mod.rs` | DONE | Copy, Eq, Default derives. serde lowercase |
| 2 | Enrichir AppState avec runtime_kind + runtime_paths | `commands.rs` | DONE | Source de verite unique |
| 3 | Remplacer chemins hardcodes dans commands.rs | `commands.rs` | DONE | 7 remplacements, agent CRUD + workflow_kit_status |
| 4 | Remplacer chemins hardcodes dans init.rs | `init.rs` | DONE | Creation dirs via RuntimePaths |
| 5 | Centraliser chemins dans deployer/sync.rs | `deployer/sync.rs` | DONE | sync_embedded, sync_settings, sync_all via RuntimePaths |
| 6 | Centraliser + renommer event watchers | `watchers/service.rs` | DONE | runtime-dir-change, WatchCategory::Runtime |
| 7 | Parametrer SessionManager par RuntimeKind | `session/manager.rs`, `commands.rs` | DONE | Stocke kind, 2 call sites maj |
| 8 | Ajouter IPC get_runtime_info | `commands.rs`, `main.rs` | DONE | RuntimeInfo: kind + name + paths |
| 9 | Types + store frontend | `types/runtime.ts`, `stores/runtimeStore.ts` | DONE | RuntimeInfo avec name lisible |
| 10 | Integration frontend | `App.tsx`, `useTauriEvents.ts` | DONE | runtime-dir-change, runtimeStore boot + reset |
| 11 | Documentation inline des seams | `runtime/paths.rs` | DONE | Seams documentes: ajout runtime, agnostic vs specific |
