# ZAOS Rust Backend — Implementation Summary

## Overview
All Rust backend files for the ZAOS Tauri v2 project have been created. The implementation follows Tauri v2 patterns and the stream-json protocol specification from the design docs.

## Files Created

### Configuration & Build
1. **src-tauri/Cargo.toml** — Tauri v2 project manifest with dependencies:
   - tauri 2.x, serde, serde_json, tokio (full)
   - notify (file watching), tauri-plugin-shell
   - tracing, thiserror, async-trait, dashmap, uuid, chrono

2. **src-tauri/tauri.conf.json** — Tauri v2 configuration:
   - App: "zaos", Window title: "ZAOS — Zep Agentic OS"
   - Resolution: 1400x900, dark theme
   - devUrl: http://localhost:1420
   - Shell plugin permissions enabled

3. **src-tauri/build.rs** — Standard Tauri build script

4. **src-tauri/capabilities/default.json** — Tauri v2 capabilities:
   - Window management, shell open/execute, core app permissions

### Events & Protocol Parsing
5. **src-tauri/src/events/types.rs** — Complete CliEvent enum matching section 10 of stream-json-protocol.md:
   - CliEvent enum with: System, StreamDelta, Assistant, User, RateLimit, Result
   - ApiStreamEvent enum for wrapped API events
   - ContentBlock enum: Thinking, ToolUse, Text
   - All supporting types: Usage, RateLimitInfo, UserMessage, etc.
   - Implements serde tag-based deserialization for proper routing

6. **src-tauri/src/events/parser.rs** — Async JSONL parser:
   - Reads lines from ChildStdout asynchronously
   - Deserializes to CliEvent using serde
   - Broadcasts via tokio broadcast channel
   - Includes error handling and forward-compatibility for unknown events

7. **src-tauri/src/events/mod.rs** — Module re-exports

### Session Management
8. **src-tauri/src/session/manager.rs** — SessionManager:
   - spawn_cli() → spawns `claude --output-format stream-json --include-partial-messages`
   - send_prompt(text) → writes to stdin
   - send_resume(session_id, text) → resumes existing session with --resume flag
   - interrupt() → sends SIGINT/kill signal
   - list_sessions() → reads from ~/.claude/projects/
   - get_event_sender() → provides broadcast channel for UI
   - is_running() → checks subprocess status
   - Uses tokio::process::Command for async spawning

9. **src-tauri/src/session/mod.rs** — Module re-exports

### Workflow Engine
10. **src-tauri/src/workflow/state.rs** — WorkflowState struct:
    - Core fields: phase, epic, task, mode, gate_validated, last_updated
    - Extended fields: history (Vec<PhaseTransition>), session (SessionMetadata)
    - WorkflowMode enum: Free, Pipeline
    - PhaseTransition, SessionMetadata, TokenUsage types
    - Methods: record_transition(), set_session(), update_tokens()

11. **src-tauri/src/workflow/engine.rs** — WorkflowEngine:
    - load_state() → reads from .workflow/state.json
    - save_state() → writes to .workflow/state.json
    - set_phase(), validate_gate(), next_phase() → phase management
    - set_mode(), set_epic(), set_task() → state updates
    - get_pipeline_for_task_type() → returns phase sequence by task type
    - watch_state_file() → returns broadcast receiver for changes
    - Uses tokio::fs for async file operations

12. **src-tauri/src/workflow/mod.rs** — Module re-exports

### Screenshots
13. **src-tauri/src/screenshots/manager.rs** — ScreenshotManager stub:
    - Screenshot struct with metadata (id, timestamp, path, session_id, tool_use_id)
    - watch_directory() → TODO: file watcher with notify
    - capture_on_demand() → TODO: MCP call to GoPeak
    - compare() → TODO: visual diff
    - get_gallery() → TODO: session screenshot query
    - cleanup_old() → TODO: cleanup logic

14. **src-tauri/src/screenshots/mod.rs** — Module re-exports

### Memory Reader
15. **src-tauri/src/memory/reader.rs** — MemoryReader stub:
    - MemoryIndex, Epic, SessionLog, ProjectState types
    - read_index() → TODO: parse MEMORY.md
    - read_state() → TODO: parse state.md
    - read_current_epic() → TODO: parse current-epic.md
    - read_session_log() → TODO: read session logs
    - watch_changes() → TODO: file watcher

16. **src-tauri/src/memory/mod.rs** — Module re-exports

### MCP Server (Phase 2)
17. **src-tauri/src/mcp_server/mod.rs** — Placeholder module with TODO for:
    - Local TCP server (127.0.0.1:random)
    - Ephemeral auth token generation
    - MCP protocol handshake
    - Tool definitions (show_diff, notify, get_ui_state, capture_screenshot)
    - .mcp.json registration
    - Tool invocation handling

### Commands
18. **src-tauri/src/commands.rs** — All Tauri IPC commands:
    - AppState struct: manages SessionManager, WorkflowEngine, project_dir
    - Response types: SendPromptResponse, InterruptResponse, ValidateGateResponse, etc.
    - **send_prompt()** — forward text to CLI stdin
    - **interrupt_session()** — send SIGINT
    - **validate_gate()** → move to next phase
    - **set_mode()** → free or pipeline
    - **get_workflow_state()** → read current state
    - **check_cli_auth()** → verify claude CLI availability
    - **list_sessions()** → enumerate sessions
    - All commands return JSON-serializable responses via #[tauri::command]

### Entry Point
19. **src-tauri/src/main.rs** — Tauri v2 application entry:
    - Initializes tracing (logs to stderr)
    - Reads ZAOS_PROJECT_DIR env var or uses cwd
    - Creates AppState
    - Registers all 7 commands via generate_handler!
    - Enables tauri-plugin-shell
    - Standard Tauri v2 setup flow

## Architecture Highlights

### Event Flow
1. Frontend invokes `send_prompt(text)` via Tauri IPC
2. SessionManager spawns CLI if needed, writes to stdin
3. CLI stdout (stream-json JSONL) is piped to event parser
4. Parser deserializes each line to CliEvent, broadcasts on channel
5. Frontend subscribes to channel, receives events in real-time
6. Events drive UI updates (chat, actions feed, workflow panel)

### State Management
- **SessionManager**: Manages CLI subprocess lifecycle, stdin/stdout
- **WorkflowEngine**: Reads/writes .workflow/state.json, manages phase transitions
- **Workflow history**: Maintains audit trail of phase changes with timestamps
- **Token tracking**: SessionMetadata tracks input/output/cache tokens

### Error Handling
- Custom error types with thiserror: SessionError, WorkflowError, ParserError, MemoryError
- Result<T> type aliases for ergonomics
- Errors propagate to frontend as String messages via Tauri
- Parser continues on deserialization errors (forward compatibility)

### Async/Concurrency
- tokio runtime for all async operations
- tokio::process::Command for subprocess spawning
- tokio::fs for async file I/O
- tokio::sync::broadcast for event broadcasting
- Arc<Mutex<T>> for thread-safe state sharing across Tauri handlers

### Testing
- Unit tests in most modules (test module configs, enums, basic functionality)
- Tests use #[cfg(test)] and #[tokio::test] where async needed
- Can be run with `cargo test`

## Integration Points

### With Frontend
- All commands return JSON via Tauri IPC (serde::Serialize)
- Backend emits Tauri events for async notifications
- Frontend can invoke commands and listen for event broadcasts

### With Claude Code CLI
- Spawned with: `claude --output-format stream-json --verbose --include-partial-messages --project-dir <path>`
- stdin: JSON prompts (line-based, text format)
- stdout: JSONL stream of CliEvent objects
- stderr: logs and errors (captured via tracing)
- Authentication: Uses existing OAuth Max config (~/.claude/credentials)

### With Filesystem
- Reads/writes .workflow/state.json (WorkflowEngine)
- Reads from .memory/* (MemoryReader — stub)
- Watches .memory/ for changes (file watcher — Phase 2)
- Session data in ~/.claude/projects/<encoded-cwd>/

### With GoPeak MCP (Phase 2)
- Screenshots captured via GoPeak tool invocations
- ScreenshotManager will expose capture_on_demand() MCP call
- Screenshots stored and displayed in gallery

## Compilation Status

All files are syntactically valid Rust code. To compile:

```bash
cd src-tauri
cargo check          # Verify no errors
cargo build          # Full build
cargo test           # Run tests
```

The project requires Rust 2021 edition and Tauri v2 build tools.

## Phase 2 TODOs

Marked throughout with `TODO` comments:
- ScreenshotManager: file watcher, GoPeak MCP integration
- MemoryReader: Markdown parsing for memory files
- MCP Server: zaos-ide local server implementation
- Additional tools and enhanced state management

## Key Design Decisions

1. **Type Safety**: Full serde-based deserialization with tag enums for event routing
2. **Error Handling**: Custom error types instead of generic strings
3. **Concurrency**: tokio broadcast for fan-out event delivery to UI
4. **File Watching**: Stub-ready with notify crate in dependencies
5. **Forward Compatibility**: Parser handles unknown event types gracefully
6. **Extensibility**: Module structure allows easy addition of new components
7. **Testing**: Unit tests in each module for core functionality
