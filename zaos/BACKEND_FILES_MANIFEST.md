# ZAOS Rust Backend — Complete Files Manifest

**Generated:** 2026-03-29
**Project:** ZAOS (Zep Agentic OS)
**Framework:** Tauri v2
**Language:** Rust (2021 edition)
**Total Files:** 19
**Total Lines of Code:** ~1,541

---

## File Locations & Descriptions

### 1. Configuration & Build (4 files)

#### `/src-tauri/Cargo.toml` (28 lines)
**Purpose:** Rust project manifest
**Contents:**
- Package metadata: name="zaos", version="0.1.0", edition="2021"
- Dependencies: tauri 2.x, serde, serde_json, tokio (full features)
- Build dependencies: tauri-build
- Additional crates: notify, uuid, chrono, tracing, thiserror, async-trait, dashmap
- Optimized release profile (LTO, single codegen unit)

#### `/src-tauri/tauri.conf.json` (35 lines)
**Purpose:** Tauri application configuration
**Contents:**
- App name: "zaos"
- Window: title "ZAOS — Zep Agentic OS", 1400x900, dark theme, resizable
- Dev settings: devUrl http://localhost:1420, beforeDevCommand "npm run dev"
- Build settings: beforeBuildCommand "npm run build", frontendDist "../dist"
- Security: CSP null for local development
- Bundle targets: deb, appimage

#### `/src-tauri/build.rs` (2 lines)
**Purpose:** Build script
**Contents:** Calls tauri_build::build() to generate Tauri bindings

#### `/src-tauri/capabilities/default.json` (13 lines)
**Purpose:** Tauri v2 capabilities and permissions
**Contents:**
- Window management: create, minimize, maximize, close
- App control: hide, show
- Shell plugin: open, execute
- Configurable allow/deny lists

---

### 2. Events & Stream-JSON Protocol (3 files)

#### `/src-tauri/src/events/types.rs` (219 lines)
**Purpose:** Complete event type definitions matching stream-json-protocol.md section 10
**Key Types:**
- `CliEvent` enum (tag-based serialization):
  - System(SystemEvent)
  - StreamDelta(StreamDeltaEvent) — *only with --include-partial-messages*
  - Assistant(AssistantEvent) — message snapshot
  - User(UserEvent) — tool_result
  - RateLimit(RateLimitEvent)
  - Result(ResultEvent)
  - Unknown — forward compatibility
- `ApiStreamEvent` enum (nested API events):
  - MessageStart, ContentBlockStart, ContentBlockDelta, ContentBlockStop
  - MessageDelta, MessageStop
- `ContentBlock` enum:
  - Thinking { thinking, signature }
  - ToolUse { id, name, input, caller }
  - Text { text }
- Supporting types: SystemEvent, AssistantEvent, UserEvent, RateLimitEvent, ResultEvent
- Data types: Usage, RateLimitInfo, ContentBlockInfo, DeltaContent, etc.

**Design Notes:**
- Uses serde tag-based deserialization for proper routing
- All types derive Debug, Clone, Deserialize, Serialize
- Complete mapping of stream-json protocol to Rust types
- Handles optional fields (Option<T>) for flexible API responses

#### `/src-tauri/src/events/parser.rs` (66 lines)
**Purpose:** Async JSONL parser for stream-json events
**Key Functions:**
- `parse_stream(stdout: ChildStdout, tx: broadcast::Sender<CliEvent>)` — main entry point
  - Async function that reads lines from child process stdout
  - Deserializes each line to CliEvent via serde_json
  - Broadcasts successful events to all UI listeners
  - Logs and continues on parse errors
- Error handling: ParserError enum with IO and JSON variants

**Features:**
- Non-blocking async I/O with tokio
- BufReader for efficient line reading
- Broadcast channel for fan-out delivery to multiple UI components
- Forward compatibility: unknown events logged but don't crash
- Debug logging of parsed events

#### `/src-tauri/src/events/mod.rs` (3 lines)
**Purpose:** Module declarations and re-exports
**Exports:** parser, types, and public API (parse_stream, ParserError, all type definitions)

---

### 3. Session Management (2 files)

#### `/src-tauri/src/session/manager.rs` (243 lines)
**Purpose:** Manage Claude Code CLI subprocess lifecycle
**Key Components:**
- `SessionManager` struct:
  - `child: Option<Child>` — active subprocess
  - `session_id: Option<String>` — current session ID
  - `tx: broadcast::Sender<CliEvent>` — event broadcast channel
  - `project_dir: PathBuf` — project directory for --project-dir flag

**Key Methods:**
- `new(project_dir)` — create new manager
- `check_cli_auth()` — verify `claude --version` works (async static)
- `spawn_cli(project_dir)` — spawn CLI with flags:
  - `--output-format stream-json`
  - `--verbose`
  - `--include-partial-messages`
  - `--project-dir <path>`
  - Returns broadcast::Receiver for UI to listen on
- `send_prompt(text)` — write prompt to stdin (line-terminated)
- `send_resume(session_id, text)` — spawn with --resume flag
- `interrupt()` — send kill signal to subprocess
- `list_sessions()` — read ~/.claude/projects/ (stub, returns Vec)
- `get_event_sender()` — return broadcast sender
- `is_running()` — check subprocess status via try_wait()

**Error Handling:**
- SessionError enum: CliNotFound, NotSpawned, WriteFailed, ParseError, Io
- Result<T> = std::result::Result<T, SessionError>

**Design Notes:**
- Uses tokio::process::Command for async subprocess spawning
- Pipes stdin/stdout/stderr to Stdio::piped()
- Event parser runs in separate tokio task
- Thread-safe: wrapped in Arc<Mutex<>> in commands
- Async/await throughout

#### `/src-tauri/src/session/mod.rs` (3 lines)
**Purpose:** Module re-exports
**Exports:** SessionError, SessionInfo, SessionManager

---

### 4. Workflow Engine (3 files)

#### `/src-tauri/src/workflow/state.rs` (167 lines)
**Purpose:** Workflow state definition and persistence
**Key Types:**
- `WorkflowState` struct:
  - Core fields: phase, epic, task, mode, gate_validated, last_updated
  - Extended fields: history (Vec<PhaseTransition>)
- `WorkflowMode` enum: Free, Pipeline
- `PhaseTransition` struct: from_phase, to_phase, timestamp, reason

**Key Methods:**
- `new(phase, epic, task)` — create WorkflowState
- `default()` — default state: phase="idle", mode=Pipeline
- `record_transition(from, to, reason)` — add to history

**Features:**
- Timestamps use chrono::Utc::now() in RFC3339 format
- History audit trail of all phase transitions
- Mirrors .workflow/state.json schema with ZAOS extensions

#### `/src-tauri/src/workflow/engine.rs` (234 lines)
**Purpose:** Workflow state management and file I/O
**Key Components:**
- `WorkflowEngine` struct:
  - `state_path: PathBuf` — path to .workflow/state.json
  - `current_state: WorkflowState` — in-memory state
  - `tx: broadcast::Sender<WorkflowState>` — state change events

**Key Methods:**
- `new(project_dir)` — create engine, setup state_path
- `load_state()` — read from .workflow/state.json via tokio::fs
- `save_state()` — write to .workflow/state.json via tokio::fs
- `get_phase()` → &str
- `set_phase(new_phase)` — record transition, update, save
- `validate_gate()` — mark gate_validated = true, save
- `next_phase()` — advance phase using state machine:
  - idle → comprehension → specification → architecture → implementation → review → test → closure → idle
- `set_mode(WorkflowMode)` — free or pipeline
- `set_epic(name)`, `set_task(description)` — metadata updates
- `get_pipeline_for_task_type(type)` — return phase sequence:
  - "bugfix" → [comprehension, implementation, review, closure]
  - "feature" → [comprehension, specification, architecture, implementation, review, test, closure]
  - "refactor" → [specification, implementation, review, test, closure]
  - default → full pipeline
- `watch_state_file()` — return broadcast receiver for UI
- `get_state()`, `get_state_mut()` — access current state

**Error Handling:**
- WorkflowError enum: Io, JsonError, InvalidPhase, StateNotFound, WatchError
- All operations return Result<T>

**Design Notes:**
- Async I/O with tokio::fs for non-blocking file operations
- State machine for phase progression
- Broadcast channel for reactive UI updates
- All changes timestamped and recorded

#### `/src-tauri/src/workflow/mod.rs` (3 lines)
**Purpose:** Module re-exports
**Exports:** WorkflowEngine, WorkflowError, WorkflowState, WorkflowMode

---

### 5. Screenshots (2 files)

#### `/src-tauri/src/screenshots/manager.rs` (90 lines)
**Purpose:** Screenshot management and gallery
**Key Types:**
- `Screenshot` struct: id, timestamp, path, session_id, tool_use_id

**Key Methods (mostly TODO for Phase 2):**
- `new(watch_dir)` — create manager
- `watch_directory()` — TODO: use notify crate to monitor directory
- `capture_on_demand()` — TODO: call GoPeak MCP tool
- `compare(before, after)` — TODO: visual diff
- `get_gallery(session_id)` — TODO: query session screenshots
- `cleanup_old(max_age_days)` — TODO: cleanup old files

**Design Notes:**
- Stub implementation with TODO markers for Phase 2
- Ready to integrate with notify crate (already in Cargo.toml)
- MCP integration points defined for future GoPeak calls

#### `/src-tauri/src/screenshots/mod.rs` (3 lines)
**Purpose:** Module re-exports
**Exports:** Screenshot, ScreenshotError, ScreenshotManager

---

### 6. Memory Reader (2 files)

#### `/src-tauri/src/memory/reader.rs` (126 lines)
**Purpose:** Read workflow memory files from .memory/ directory
**Key Types:**
- `MemoryIndex` struct: epics, sessions
- `Epic` struct: name, description, status
- `SessionLog` struct: id, date, duration_ms, tokens_used
- `ProjectState` struct: name, created_at, last_updated

**Key Methods (mostly TODO for Phase 2):**
- `new(project_dir)` — create reader, setup .memory/ path
- `read_index()` — TODO: parse MEMORY.md markdown
- `read_state()` — TODO: parse state.md
- `read_current_epic()` — TODO: parse current-epic.md
- `read_session_log(date)` — TODO: read session-*.md files
- `watch_changes()` — TODO: notify-based file watcher

**Design Notes:**
- Stub implementation ready for Phase 2
- Markdown parsing will be added later
- File watcher infrastructure ready

#### `/src-tauri/src/memory/mod.rs` (3 lines)
**Purpose:** Module re-exports
**Exports:** MemoryReader, MemoryError, MemoryIndex, ProjectState

---

### 7. MCP Server (1 file)

#### `/src-tauri/src/mcp_server/mod.rs` (26 lines)
**Purpose:** Placeholder for zaos-ide MCP server (Phase 2)
**Planned Features:**
- Local TCP server on 127.0.0.1:random port
- Ephemeral auth token generation (~/.claude/ide/)
- MCP protocol handshake
- Tool definitions: show_diff, notify, get_ui_state, capture_screenshot
- Registration in .mcp.json for CLI discovery
- Bidirectional communication with Claude Code CLI

**Current Status:** Comprehensive TODO documentation for Phase 2 implementation

---

### 8. Commands (1 file)

#### `/src-tauri/src/commands.rs` (242 lines)
**Purpose:** Tauri IPC command handlers
**Key Components:**
- `AppState` struct:
  - `session_manager: Arc<Mutex<SessionManager>>`
  - `workflow_engine: Arc<Mutex<WorkflowEngine>>`
  - `project_dir: PathBuf`
  - Thread-safe state shared across Tauri handlers

**Response Types:**
- SendPromptResponse { success, message }
- InterruptResponse { success, message }
- ValidateGateResponse { success, next_phase, message }
- SetModeResponse { success, mode }
- WorkflowStateResponse { phase, epic, task, mode, gate_validated }
- CheckAuthResponse { authenticated, version, message }
- ListSessionsResponse { sessions }

**Command Handlers (Tauri #[tauri::command]):**
1. `send_prompt(text)` — forward to CLI stdin
   - Auto-spawns CLI if needed
   - Returns SendPromptResponse

2. `interrupt_session()` — send SIGINT
   - Returns InterruptResponse

3. `validate_gate()` — mark gate valid, advance phase
   - Calls workflow.validate_gate() then next_phase()
   - Returns ValidateGateResponse with new phase

4. `set_mode(mode)` — free or pipeline
   - Validates mode string ("free" or "pipeline")
   - Returns SetModeResponse

5. `get_workflow_state()` — read current state
   - Loads state from .workflow/state.json if needed
   - Returns WorkflowStateResponse

6. `check_cli_auth()` — verify Claude CLI availability
   - Calls SessionManager::check_cli_auth()
   - Returns CheckAuthResponse (no error, returns auth=false if unavailable)

7. `list_sessions()` — enumerate sessions
   - Returns ListSessionsResponse

**Design Notes:**
- All handlers are async functions
- Use State<'_, AppState> to access shared state
- All responses derive Serialize for automatic JSON conversion
- Errors converted to String messages
- AppState creation in main.rs, shared via tauri::Builder::manage()

---

### 9. Entry Point (1 file)

#### `/src-tauri/src/main.rs` (66 lines)
**Purpose:** Tauri v2 application entry point
**Key Functions:**
- `main()` — application entry:
  1. Initialize tracing via tracing_subscriber with EnvFilter
  2. Read ZAOS_PROJECT_DIR env var or use current directory
  3. Create AppState with project_dir
  4. Build Tauri app:
     - Enable tauri-plugin-shell
     - Manage app state via State<AppState>
     - Register all 7 commands via generate_handler! macro
     - Setup hook for initialization logging
  5. Run application

**Build Configuration:**
- cfg attribute: `#[cfg_attr(...)] windows_subsystem = "windows"`
- Hides console window on Windows release builds

**Logging:**
- Uses tracing crate for structured logging
- Supports RUST_LOG environment variable
- Logs project directory and startup events

**Module Declarations:**
- Declares all submodules: commands, events, memory, mcp_server, screenshots, session, workflow

---

## Statistics

| Category | Count |
|----------|-------|
| Total Files | 19 |
| Source Files (.rs) | 15 |
| Config Files | 4 |
| Total Lines of Code | ~1,541 |
| Modules | 7 (events, session, workflow, screenshots, memory, mcp_server, commands) |
| Public Types | ~30 |
| Tauri Commands | 7 |
| Error Types | 4 (SessionError, WorkflowError, ParserError, MemoryError) |

---

## Compilation & Testing

### Quick Start
```bash
cd src-tauri

# Check for errors (no build artifacts)
cargo check

# Full build
cargo build --debug

# Run tests
cargo test

# Release build
cargo build --release
```

### Environment
- Rust Edition: 2021
- MSRV: 1.70+ (typical for Tauri v2)
- Async Runtime: tokio
- Target Platforms: Linux, macOS, Windows

### Dependencies Summary
```
tauri              2.x        (desktop framework)
serde/serde_json   1.0        (serialization)
tokio              1.x        (async runtime, full features)
notify             6.x        (file watching)
uuid               1.0        (unique IDs)
chrono             0.4        (timestamps)
tracing            0.1        (logging)
thiserror          1.0        (error handling)
async-trait        0.1        (async traits)
dashmap            5.5        (concurrent hashmap)
tauri-plugin-shell 2.x        (shell operations)
```

---

## Integration Architecture

```
Frontend (React/TypeScript)
    ↓
Tauri IPC (invoke/events)
    ↓
Commands (send_prompt, validate_gate, etc.)
    ↓
AppState (SessionManager + WorkflowEngine)
    ↓
┌─────────────────────────────────────┐
│ SessionManager                       │
│ - spawn CLI subprocess              │
│ - manage stdin/stdout               │
│ - emit CliEvent broadcasts          │
└─────────────────────────────────────┘
    ↓
Claude Code CLI (stream-json format)
    ↓
┌─────────────────────────────────────┐
│ WorkflowEngine                       │
│ - read/write state.json             │
│ - manage phase transitions          │
│ - emit WorkflowState broadcasts     │
└─────────────────────────────────────┘
    ↓
.workflow/state.json

Frontend listens to:
- Event broadcasts from SessionManager (chat updates)
- State broadcasts from WorkflowEngine (pipeline updates)
```

---

## Next Steps (Phase 2)

1. **Frontend Integration**: Build React components to consume Tauri commands and event streams
2. **MCP Server**: Implement zaos-ide MCP server for bidirectional CLI-UI communication
3. **File Watchers**: Complete notify-based watchers for .memory/ and .workflow/ directories
4. **Screenshot Integration**: Connect ScreenshotManager to GoPeak MCP tool calls
5. **Memory Parsing**: Add markdown parsing for .memory/*.md files
6. **Testing**: Integration tests with mock CLI subprocess
7. **Error UI**: Enhanced error display and recovery suggestions

---

## File Locations (Absolute Paths)

All files are located under: `/sessions/festive-funny-fermi/mnt/Workflow/zaos/src-tauri/`

Configuration:
- `Cargo.toml`
- `tauri.conf.json`
- `build.rs`
- `capabilities/default.json`

Source Code:
- `src/main.rs`
- `src/commands.rs`
- `src/events/{types.rs, parser.rs, mod.rs}`
- `src/session/{manager.rs, mod.rs}`
- `src/workflow/{state.rs, engine.rs, mod.rs}`
- `src/screenshots/{manager.rs, mod.rs}`
- `src/memory/{reader.rs, mod.rs}`
- `src/mcp_server/mod.rs`
