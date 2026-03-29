use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Top-level event from the stream-json protocol
/// With --include-partial-messages, we receive both StreamDelta and Assistant snapshots
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum CliEvent {
    #[serde(rename = "system")]
    System(SystemEvent),

    /// API events streamed as deltas — ONLY with --include-partial-messages
    #[serde(rename = "stream_event")]
    StreamDelta(StreamDeltaEvent),

    /// Complete snapshot of assistant message (emitted after deltas)
    #[serde(rename = "assistant")]
    Assistant(AssistantEvent),

    /// Result of a tool use
    #[serde(rename = "user")]
    User(UserEvent),

    #[serde(rename = "rate_limit_event")]
    RateLimit(RateLimitEvent),

    #[serde(rename = "result")]
    Result(ResultEvent),

    #[serde(rename = "control_request")]
    ControlRequest(ControlRequest),

    /// For unrecognized events (forward compatibility)
    #[serde(other)]
    Unknown,
}

/// Wrapper around Claude API events (streaming deltas)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StreamDeltaEvent {
    pub event: ApiStreamEvent,
    pub session_id: String,
    pub parent_tool_use_id: Option<String>,
    pub uuid: String,
}

/// Raw Claude API event (inside stream_event.event)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ApiStreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: Value },

    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        index: u32,
        content_block: ContentBlockInfo,
    },

    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        index: u32,
        delta: DeltaContent,
    },

    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: u32 },

    #[serde(rename = "message_delta")]
    MessageDelta {
        delta: Value,
        usage: Option<Usage>,
    },

    #[serde(rename = "message_stop")]
    MessageStop,

    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContentBlockInfo {
    #[serde(rename = "type")]
    pub block_type: String, // "text", "tool_use", "thinking"
    pub text: Option<String>,
    pub id: Option<String>,     // for tool_use
    pub name: Option<String>,   // for tool_use
    pub input: Option<Value>,   // for tool_use
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum DeltaContent {
    #[serde(rename = "text_delta")]
    TextDelta { text: String },

    #[serde(rename = "input_json_delta")]
    InputJsonDelta { partial_json: String },

    #[serde(rename = "thinking_delta")]
    ThinkingDelta { thinking: String },

    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SystemEvent {
    pub subtype: String, // "init", "api_retry"
    pub session_id: String,
    pub uuid: String,
    // init fields
    pub cwd: Option<String>,
    pub tools: Option<Vec<String>>,
    pub mcp_servers: Option<Vec<Value>>,
    pub model: Option<String>,
    pub claude_code_version: Option<String>,
    pub agents: Option<Vec<String>>,
    pub slash_commands: Option<Vec<String>>,
    pub skills: Option<Vec<String>>,
    // api_retry fields
    pub attempt: Option<u32>,
    pub max_retries: Option<u32>,
    pub error: Option<String>,
    pub error_status: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssistantEvent {
    pub message: AssistantMessage,
    pub parent_tool_use_id: Option<String>,
    pub session_id: String,
    pub uuid: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssistantMessage {
    pub id: String,
    pub model: String,
    pub content: Vec<ContentBlock>,
    pub stop_reason: Option<String>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "thinking")]
    Thinking {
        thinking: String,
        signature: String,
    },

    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
        caller: Option<Value>,
    },

    #[serde(rename = "text")]
    Text { text: String },

    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserEvent {
    pub message: UserMessage,
    pub parent_tool_use_id: Option<String>,
    pub session_id: String,
    pub uuid: String,
    pub timestamp: Option<String>,
    pub tool_use_result: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserMessage {
    pub role: String,
    pub content: Vec<ToolResultBlock>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolResultBlock {
    pub tool_use_id: String,
    #[serde(rename = "type")]
    pub block_type: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimitEvent {
    pub rate_limit_info: RateLimitInfo,
    pub uuid: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimitInfo {
    pub status: String, // "allowed", "rate_limited"
    #[serde(rename = "resetsAt")]
    pub resets_at: u64,
    #[serde(rename = "rateLimitType")]
    pub rate_limit_type: String,
    pub overageStatus: Option<String>,
    pub isUsingOverage: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResultEvent {
    pub subtype: String, // "success", "error"
    pub is_error: bool,
    pub duration_ms: u64,
    pub duration_api_ms: u64,
    pub num_turns: u32,
    pub result: String,
    pub stop_reason: String,
    pub session_id: String,
    pub total_cost_usd: f64,
    pub usage: Value,
    #[serde(rename = "modelUsage")]
    pub model_usage: Option<Value>,
    pub uuid: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ControlRequest {
    pub request_id: String,
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default)]
    pub input: Option<Value>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub uuid: Option<String>,
    #[serde(default)]
    pub subtype: Option<String>,
    /// Catch any extra fields we don't know about yet
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_input_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
}
