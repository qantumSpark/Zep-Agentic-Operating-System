# Epic active : Completion du seam runtime

> Milestone : 24 — Chantier Codex (5/5)
> Statut : TERMINE

## Objectif

Completer le seam runtime pour qu'un futur CodexRuntime soit un ajout controle. Formaliser les interfaces, internaliser le mapping, nettoyer les hardcodes Claude, documenter la feuille de route.

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| 1 | Changer signature trait start_session() → Receiver ZaosEvent | `runtime/mod.rs` | DONE | Import + signature + doc |
| 2 | Internaliser mapping CliEvent→ZaosEvent dans ClaudeRuntime | `runtime/claude.rs` | DONE | Parsing inline + mapping |
| 3 | Propager ZaosEvent dans SessionManager | `session/manager.rs` | DONE | Import + signature |
| 4 | Simplifier forwarder dans commands.rs | `commands.rs` | DONE | CliEvent elimine, emit direct, cargo check OK |
| 5 | Factory create_runtime() | `runtime/mod.rs`, `session/manager.rs` | DONE | Factory + tests OK |
| 6 | RuntimeKind switchable Arc RwLock dans AppState | `commands.rs` | DONE | Arc RwLock + read().await |
| 7 | Renommer mapper.rs → claude_mapper.rs | `events/` | DONE | Rename + doc + 7 tests OK |
| 8 | Elargir type RuntimeInfo.kind frontend | `runtime.ts` | DONE | "claude" union string extensible |
| 9 | UI runtime-aware textes dynamiques | `ThinkingIndicator.tsx`, `StatusBar.tsx` | DONE | runtimeStore dynamique |
| 10 | Supprimer fallback .claude/agents dans agentsStore | `agentsStore.ts` | DONE | Fallback vide + commentaire |
| 11 | Feuille de route RUNTIME_SEAM.md | `RUNTIME_SEAM.md` | DONE | Architecture + checklist + backlog |
| 12 | S1 — Supprimer re-export parse_stream | `events/mod.rs` | DONE | Review suggestion |
| 13 | S2 — Supprimer glob re-export types::* | `events/mod.rs` | DONE | Review suggestion |
| 14 | S3 — Supprimer variants SessionError morts | `session/manager.rs` | DONE | CliNotFound, NotSpawned, ParseError supprimés |
| 15 | S4 — Supprimer champ runtime_kind mort | `session/manager.rs` | DONE | Champ struct supprimé, param new() gardé pour factory |
| 16 | S5 — Corriger doc comment new() | `session/manager.rs` | DONE | "default Claude" → "specified runtime kind" |
