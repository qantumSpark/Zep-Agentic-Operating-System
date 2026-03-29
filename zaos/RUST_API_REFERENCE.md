# ZAOS Rust Backend — API Reference

## Tauri IPC Commands

These commands are exposed to the frontend via Tauri IPC. Call them using `invoke("command_name", args)`.

### 1. send_prompt
```typescript
// Frontend call
const response = await invoke<SendPromptResponse>('send_prompt', {
  text: "Describe the current scene"
})

// Response
interface SendPromptResponse {
  success: boolean
  message: string  // "Prompt sent"
}

// Backend handler
async fn send_prompt(text: String, state: State<'_, AppState>) -> Result<SendPromptResponse, String>
```
**Behavior:**
- Checks if CLI is running, spawns if needed
- Writes prompt to CLI stdin (newline-terminated)
- Returns immediately (streaming events via event channel)
- Errors: CLI not found, write failed

---

### 2. interrupt_session
```typescript
// Frontend call
const response = await invoke<InterruptResponse>('interrupt_session')

// Response
interface InterruptResponse {
  success: boolean
  message: string  // "Session interrupted"
}

// Backend handler
async fn interrupt_session(state: State<'_, AppState>) -> Result<InterruptResponse, String>
```
**Behavior:**
- Sends SIGKILL to CLI subprocess
- Closes stdin/stdout pipes
- Does not wait for process (fire-and-forget)

---

### 3. validate_gate
```typescript
// Frontend call
const response = await invoke<ValidateGateResponse>('validate_gate')

// Response
interface ValidateGateResponse {
  success: boolean
  next_phase: string  // e.g., "implementation"
  message: string     // "Gate validated"
}

// Backend handler
async fn validate_gate(state: State<'_, AppState>) -> Result<ValidateGateResponse, String>
```
**Behavior:**
- Calls WorkflowEngine::validate_gate() (sets gate_validated = true)
- Calls WorkflowEngine::next_phase() (advances to next phase)
- Saves state.json
- Returns next phase name
- Errors: invalid phase, I/O error

---

### 4. set_mode
```typescript
// Frontend call
const response = await invoke<SetModeResponse>('set_mode', {
  mode: "free"  // or "pipeline"
})

// Response
interface SetModeResponse {
  success: boolean
  mode: string
}

// Backend handler
async fn set_mode(
  mode: String,
  state: State<'_, AppState>
) -> Result<SetModeResponse, String>
```
**Behavior:**
- Validates mode ("free" or "pipeline")
- Updates WorkflowState::mode
- Saves state.json
- Broadcasts state change
- Errors: invalid mode, I/O error

---

### 5. get_workflow_state
```typescript
// Frontend call
const response = await invoke<WorkflowStateResponse>('get_workflow_state')

// Response
interface WorkflowStateResponse {
  phase: string             // e.g., "implementation"
  epic: string              // e.g., "Combat System"
  task: string              // current task description
  mode: string              // "free" or "pipeline"
  gate_validated: boolean
}

// Backend handler
async fn get_workflow_state(
  state: State<'_, AppState>
) -> Result<WorkflowStateResponse, String>
```
**Behavior:**
- Loads state.json if not in memory
- Returns current WorkflowState values
- Does not modify state
- Errors: file not found, JSON parse error

---

### 6. check_cli_auth
```typescript
// Frontend call
const response = await invoke<CheckAuthResponse>('check_cli_auth')

// Response
interface CheckAuthResponse {
  authenticated: boolean  // true if "claude --version" succeeds
  version: string         // e.g., "Claude Code v2.1.85"
  message: string
}

// Backend handler
async fn check_cli_auth() -> Result<CheckAuthResponse, String>
```
**Behavior:**
- Spawns `claude --version`
- Checks exit code
- Never returns error (returns authenticated: false on failure)
- Safe to call on startup to verify CLI availability

---

### 7. list_sessions
```typescript
// Frontend call
const response = await invoke<ListSessionsResponse>('list_sessions')

// Response
interface ListSessionsResponse {
  sessions: string[]  // session IDs
}

// Backend handler
async fn list_sessions(state: State<'_, AppState>) -> Result<ListSessionsResponse, String>
```
**Behavior:**
- Returns list of available session IDs
- Currently stub (returns empty Vec)
- TODO: Read from ~/.claude/projects/<encoded-cwd>/

---

## Event Broadcasts

The backend broadcasts events via Tauri event system. Subscribe to these in the frontend.

### CliEvent Stream
```typescript
// Frontend subscription
import { listen } from '@tauri-apps/api/event'

const unlisten = await listen<CliEvent>('cli-event', (event) => {
  console.log('Received CLI event:', event.payload)
})

// Event types (received as JSON)
interface CliEvent {
  type: 'system' | 'stream_event' | 'assistant' | 'user' | 'rate_limit_event' | 'result'
  // ... type-specific fields
}
```

**System Event**
```typescript
{
  type: 'system',
  subtype: 'init',
  session_id: string,
  cwd: string,
  model: string,
  tools: string[],
  claude_code_version: string,
  // ... more fields
}
```

**Assistant Event** (message snapshot)
```typescript
{
  type: 'assistant',
  message: {
    id: string,
    model: string,
    content: ContentBlock[],
    stop_reason: string | null,
    usage: { input_tokens, output_tokens, ... }
  },
  session_id: string,
  uuid: string
}
```

**Stream Event** (delta)
```typescript
{
  type: 'stream_event',
  event: {
    type: 'content_block_delta',
    index: number,
    delta: {
      type: 'text_delta',
      text: string
    }
  },
  session_id: string,
  uuid: string
}
```

**User Event** (tool result)
```typescript
{
  type: 'user',
  message: {
    role: 'user',
    content: [
      {
        tool_use_id: string,
        type: 'tool_result',
        content: string
      }
    ]
  },
  timestamp: string,
  tool_use_result: { type: string, ... }
}
```

**Rate Limit Event**
```typescript
{
  type: 'rate_limit_event',
  rate_limit_info: {
    status: 'allowed' | 'rate_limited',
    resetsAt: number,  // Unix timestamp
    rateLimitType: string
  },
  session_id: string
}
```

**Result Event**
```typescript
{
  type: 'result',
  subtype: 'success' | 'error',
  is_error: boolean,
  duration_ms: number,
  num_turns: number,
  result: string,
  total_cost_usd: number,
  usage: { input_tokens, output_tokens, ... }
}
```

### WorkflowState Change Event
```typescript
// Emitted when workflow state changes via validate_gate, set_mode, etc.
{
  type: 'workflow-state',
  phase: string,
  epic: string,
  task: string,
  mode: 'free' | 'pipeline',
  gate_validated: boolean,
  last_updated: string
}
```

---

## Rust Module API

### events::types

```rust
pub enum CliEvent { ... }
pub struct SystemEvent { ... }
pub struct AssistantEvent { ... }
pub struct UserEvent { ... }
pub struct RateLimitEvent { ... }
pub struct ResultEvent { ... }
pub enum ContentBlock {
  Thinking { thinking: String, signature: String },
  ToolUse { id: String, name: String, input: Value, caller: Option<Value> },
  Text { text: String },
}
```

### events::parser

```rust
pub async fn parse_stream(
  stdout: ChildStdout,
  tx: broadcast::Sender<CliEvent>
) -> Result<()>

pub enum ParserError { ... }
pub type Result<T> = std::result::Result<T, ParserError>;
```

### session::manager

```rust
pub struct SessionManager { ... }

impl SessionManager {
  pub fn new(project_dir: PathBuf) -> Self
  pub async fn check_cli_auth() -> Result<String>
  pub async fn spawn_cli(
    &mut self,
    project_dir: Option<PathBuf>
  ) -> Result<broadcast::Receiver<CliEvent>>
  pub async fn send_prompt(&mut self, text: &str) -> Result<()>
  pub async fn send_resume(&mut self, session_id: &str, text: &str) -> Result<()>
  pub async fn interrupt(&mut self) -> Result<()>
  pub async fn list_sessions(&self) -> Result<Vec<String>>
  pub fn get_event_sender(&self) -> broadcast::Sender<CliEvent>
  pub fn is_running(&mut self) -> bool
}

pub enum SessionError { ... }
pub type Result<T> = std::result::Result<T, SessionError>;
```

### workflow::state

```rust
pub struct WorkflowState {
  pub phase: String,
  pub epic: String,
  pub task: String,
  pub mode: WorkflowMode,
  pub gate_validated: bool,
  pub last_updated: String,
  pub history: Vec<PhaseTransition>,
  pub session: Option<SessionMetadata>,
}

pub enum WorkflowMode { Free, Pipeline }

pub struct PhaseTransition {
  pub from_phase: String,
  pub to_phase: String,
  pub timestamp: String,
  pub reason: Option<String>,
}

pub struct SessionMetadata {
  pub session_id: String,
  pub started_at: String,
  pub updated_at: String,
  pub tokens_used: TokenUsage,
}

impl WorkflowState {
  pub fn new(phase: String, epic: String, task: String) -> Self
  pub fn record_transition(&mut self, from: String, to: String, reason: Option<String>)
  pub fn set_session(&mut self, session_id: String)
  pub fn update_tokens(&mut self, input: u64, output: u64, cache_read: u64, cache_creation: u64)
}
```

### workflow::engine

```rust
pub struct WorkflowEngine { ... }

impl WorkflowEngine {
  pub fn new(project_dir: PathBuf) -> Self
  pub async fn load_state(&mut self) -> Result<WorkflowState>
  pub async fn save_state(&self) -> Result<()>
  pub fn get_phase(&self) -> &str
  pub async fn set_phase(&mut self, new_phase: String) -> Result<()>
  pub async fn validate_gate(&mut self) -> Result<bool>
  pub async fn next_phase(&mut self) -> Result<String>
  pub async fn set_mode(&mut self, mode: WorkflowMode) -> Result<()>
  pub async fn set_epic(&mut self, name: String) -> Result<()>
  pub async fn set_task(&mut self, description: String) -> Result<()>
  pub fn get_pipeline_for_task_type(&self, task_type: &str) -> Vec<&'static str>
  pub fn watch_state_file(&self) -> broadcast::Receiver<WorkflowState>
  pub fn get_state(&self) -> &WorkflowState
  pub fn get_state_mut(&mut self) -> &mut WorkflowState
}

pub enum WorkflowError { ... }
pub type Result<T> = std::result::Result<T, WorkflowError>;
```

### screenshots::manager

```rust
pub struct Screenshot {
  pub id: String,
  pub timestamp: String,
  pub path: PathBuf,
  pub session_id: String,
  pub tool_use_id: Option<String>,
}

pub struct ScreenshotManager { ... }

impl ScreenshotManager {
  pub fn new(watch_dir: PathBuf) -> Self
  pub async fn watch_directory(&self) -> Result<()>           // TODO
  pub async fn capture_on_demand(&self) -> Result<Screenshot> // TODO
  pub async fn compare(&self, before: &Screenshot, after: &Screenshot) -> Result<String> // TODO
  pub async fn get_gallery(&self, session_id: &str) -> Result<Vec<Screenshot>> // TODO
  pub async fn cleanup_old(&self, max_age_days: u32) -> Result<()> // TODO
}

pub enum ScreenshotError { ... }
pub type Result<T> = std::result::Result<T, ScreenshotError>;
```

### memory::reader

```rust
pub struct MemoryIndex {
  pub epics: Vec<Epic>,
  pub sessions: Vec<SessionLog>,
}

pub struct Epic {
  pub name: String,
  pub description: String,
  pub status: String,
}

pub struct ProjectState {
  pub name: String,
  pub created_at: String,
  pub last_updated: String,
}

pub struct MemoryReader { ... }

impl MemoryReader {
  pub fn new(project_dir: PathBuf) -> Self
  pub async fn read_index(&self) -> Result<MemoryIndex>                   // TODO
  pub async fn read_state(&self) -> Result<ProjectState>                  // TODO
  pub async fn read_current_epic(&self) -> Result<Option<Epic>>           // TODO
  pub async fn read_session_log(&self, date: &str) -> Result<Option<SessionLog>> // TODO
  pub async fn watch_changes(&self) -> Result<()>                          // TODO
}

pub enum MemoryError { ... }
pub type Result<T> = std::result::Result<T, MemoryError>;
```

### commands

```rust
pub struct AppState {
  pub session_manager: Arc<Mutex<SessionManager>>,
  pub workflow_engine: Arc<Mutex<WorkflowEngine>>,
  pub project_dir: PathBuf,
}

// All commands are #[tauri::command] async functions
// See command documentation above
```

---

## Common Patterns

### Command with State Access
```rust
#[tauri::command]
pub async fn my_command(
  input: String,
  state: State<'_, AppState>
) -> Result<MyResponse, String> {
  let session = state.session_manager.lock().await;
  // Use session...
  Ok(MyResponse { /* ... */ })
}
```

### Broadcasting Events
```rust
let tx = state.session_manager.get_event_sender();
let _ = tx.send(my_event);  // Broadcast to all receivers
```

### Subscribing to Events (Frontend)
```typescript
const unlisten = await listen<CliEvent>('cli-event', (event) => {
  // Handle event
})

// Later: cleanup
unlisten()
```

### Locking Shared State
```rust
let mut engine = state.workflow_engine.lock().await;
engine.set_phase("implementation".to_string()).await?;
drop(engine);  // Lock released
```

---

## Error Handling

All commands return `Result<T, String>` where errors are converted to strings.

**Frontend:**
```typescript
try {
  const response = await invoke('command_name')
  // Handle response
} catch (error) {
  console.error('Command failed:', error)
  // error is the error message string
}
```

**Common Error Messages:**
- "Failed to spawn CLI: ..." — CLI not found or already running
- "Failed to send prompt: ..." — stdin write failed
- "Failed to load state: ..." — state.json missing or invalid JSON
- "Invalid phase: ..." — unrecognized phase name
- "Claude CLI not found or not authenticated" — from check_cli_auth

---

## Data Flow Example: send_prompt

```
Frontend
  ↓
invoke('send_prompt', { text: "..." })
  ↓
Tauri IPC bridge
  ↓
send_prompt(text: String, state: State<'_, AppState>)
  ↓
session_manager.lock().await
  ↓
is_running() check
  ├─ If not running: spawn_cli()
  │  ├─ Command::new("claude")
  │  ├─ Add --output-format stream-json
  │  ├─ Pipe stdout
  │  └─ Spawn event parser task
  │
  └─ send_prompt(&text)
     ├─ Get stdin
     ├─ Write "text\n"
     └─ Flush
  ↓
Parser task reads stdout lines
  ↓
serde_json::from_str::<CliEvent>()
  ↓
tx.send(event)  // Broadcast to all receivers
  ↓
Frontend receives via listen<CliEvent>()
  ↓
React component updates chat, actions feed, etc.
```

---

## Testing

```bash
# Run all tests
cargo test

# Run tests for specific module
cargo test events::

# Run with output
cargo test -- --nocapture

# Run single test
cargo test test_event_deserialization
```

All modules include unit tests covering:
- Type creation and defaults
- Serialization/deserialization
- Basic functionality
- Error cases
