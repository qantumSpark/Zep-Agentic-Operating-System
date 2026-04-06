# Runtime Seam — Architecture & Roadmap

> Last updated: 2026-04-06
> Status: Seam complete. No second runtime implemented yet.

## Architecture overview

```
                 +-----------------+
                 |   Frontend      |
                 |  (React / TS)   |
                 |                 |
                 | Listens to:     |
                 |  "agent-event"  |
                 |  (ZaosEvent)    |
                 +--------+--------+
                          |
                   Tauri emit
                          |
                 +--------+--------+
                 |  Forwarder      |
                 |  (commands.rs)  |
                 |                 |
                 | Receives:       |
                 |  ZaosEvent      |
                 | Policy Engine   |
                 | Gate detection  |
                 +--------+--------+
                          |
               broadcast::Receiver<ZaosEvent>
                          |
            +-------------+-------------+
            |                           |
   +--------+--------+       +---------+--------+
   | ClaudeRuntime    |       | (future)         |
   | (claude.rs)      |       | CodexRuntime     |
   |                  |       |                  |
   | CliEvent parsing |       | Codex parsing    |
   | claude_mapper    |       | codex_mapper     |
   +------------------+       +------------------+
```

## What is runtime-agnostic (shared)

| Layer | Files | Notes |
|-------|-------|-------|
| AgentRuntime trait | `runtime/mod.rs` | 10 methods, returns `ZaosEvent` |
| RuntimeKind enum | `runtime/paths.rs` | Serde-compatible, `Default = Claude` |
| RuntimePaths | `runtime/paths.rs` | `for_kind()` method, match on RuntimeKind |
| create_runtime factory | `runtime/mod.rs` | Single entry-point for runtime construction |
| SessionManager | `session/manager.rs` | Wraps `Box<dyn AgentRuntime>`, delegates all calls |
| Session logger | `session/logger.rs` | Writes `.memory/sessions/`, no runtime reference |
| ZaosEvent | `events/zaos_events.rs` | 15 variants, provider-neutral |
| Forwarder | `commands.rs` | Receives ZaosEvent, runs Policy Engine |
| AppState | `commands.rs` | `runtime_kind: Arc<RwLock<RuntimeKind>>` |
| Frontend stores | `runtimeStore.ts` | Loads `RuntimeInfo` via IPC |
| Frontend events | `useTauriEvents.ts` | Listens to `ZaosEvent` only |
| Policy Engine | `policy.rs` | Works on tool names + inputs, no runtime coupling |
| Workflow Engine | `workflow/` | Phase management, gate logic, runtime-agnostic |
| Memory system | `memory/` | Reader/writer for `.memory/` files |

## What is provider-specific (by design)

| Component | Files | Why it stays specific |
|-----------|-------|----------------------|
| ClaudeRuntime impl | `runtime/claude.rs` | CLI spawn, stdin/stdout JSONL protocol |
| CliEvent types | `events/types.rs` | Claude stream-json wire format |
| Claude mapper | `events/claude_mapper.rs` | CliEvent -> ZaosEvent translation |
| JSONL parser | `events/parser.rs` | Reads Claude stdout format |
| Deployer content | `deployer/embedded.rs` | CLAUDE.md, .mdc rules, settings.json hooks |
| Deployer sync | `deployer/sync.rs` | `sync_claude_md()`, Claude Code hooks format |
| Hooks binary | `bin/zaos_hooks.rs` | Hardcodes `.claude/` paths, duplicates RuntimePaths |
| MCP discovery | `mcp_server/mod.rs` | Writes `.mcp.json` (Claude Code format) |

## Checklist: adding a new runtime (e.g. Codex)

### Backend (Rust)

1. **Add variant to `RuntimeKind`** (`runtime/paths.rs`)
   - Add `Codex` variant to the enum
   - Add a branch in `RuntimePaths::for_kind()` with Codex-specific paths

2. **Implement `AgentRuntime`** (new file `runtime/codex.rs`)
   - Implement all 10 trait methods
   - `start_session()` must return `broadcast::Receiver<ZaosEvent>`
   - Internal parsing + mapping is encapsulated (see `claude.rs` for reference)

3. **Add mapper** (new file `events/codex_mapper.rs`)
   - Define Codex-native event types (equivalent of `CliEvent`)
   - Implement `map_codex_event() -> Vec<ZaosEvent>`
   - Register module in `events/mod.rs`

4. **Register in factory** (`runtime/mod.rs`)
   - Add `RuntimeKind::Codex => Box::new(codex::CodexRuntime::new(dir))` in `create_runtime()`

5. **Deployer** (optional)
   - Add Codex-specific embedded files if needed
   - Add `sync_codex_config()` in `deployer/sync.rs` if the runtime uses config files

### Frontend (TypeScript)

6. **Update `RuntimeInfo.kind`** (`types/runtime.ts`)
   - Already extensible: `kind: "claude" | (string & {})`
   - UI components use `runtimeStore.info?.name` dynamically (no hardcoded names)

### Configuration

7. **Runtime selection mechanism** (not yet implemented)
   - Option A: CLI argument `--runtime codex`
   - Option B: Project config `.zaos/config.json` with `{ "runtime": "codex" }`
   - Option C: UI selector in dashboard
   - The `runtime_kind` in `AppState` is already `Arc<RwLock<>>`, ready for runtime switching

## Known gaps (backlog)

| ID | Gap | Impact | Effort |
|----|-----|--------|--------|
| H1 | Hooks binary hardcodes `.claude/` paths | Hooks won't work with non-Claude runtime | Medium — needs to read RuntimeKind from config |
| D1 | Deployer is monolithic | Can only deploy Claude-specific content | Medium — needs per-runtime embedded content |
| D2 | `sync_claude_md()` name/logic | Deploys Claude-specific instruction file | Low — rename + parameterize |
| M1 | MCP discovery writes `.mcp.json` | Claude Code specific format | Low — parameterize or skip for other runtimes |
| P1 | `parse_stream` in `events/parser.rs` still exported | Unused after internalisation, but still public | Low — cleanup |
| P2 | `events/types.rs` re-exported via `pub use types::*` | Exposes CliEvent types unnecessarily | Low — remove glob re-export |

## Design decisions

1. **Mapping inside the runtime, not in the forwarder**: Each runtime converts its native events to `ZaosEvent` before broadcasting. The forwarder is runtime-agnostic.

2. **One mapper per provider**: `claude_mapper.rs`, `codex_mapper.rs`, etc. Convention is `<provider>_mapper.rs` in `events/`.

3. **Factory over registry**: `create_runtime()` uses a simple match. No dynamic registration needed for a small number of runtimes.

4. **RuntimeKind is switchable but not switched**: `Arc<RwLock<RuntimeKind>>` is ready for a future `switch_runtime` IPC command, but no UI or command exists yet.

5. **Frontend never sees provider-specific types**: The TS `claudeMapper.ts` adapter exists but is unused (backend does the mapping). It can be kept as reference or removed.
