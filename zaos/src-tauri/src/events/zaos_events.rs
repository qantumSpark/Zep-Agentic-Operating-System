//! ZAOS Normalized Event Model — provider-neutral.
//!
//! Every runtime (Claude, Codex, …) maps its native events to this model
//! before emitting to the frontend. The frontend never sees runtime-specific types.

use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ZaosEvent {
    // -- Session lifecycle --------------------------------------------------
    #[serde(rename = "session_init")]
    SessionInit {
        #[serde(rename = "sessionId")]
        session_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        model: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tools: Option<Vec<String>>,
        #[serde(rename = "mcpServers", skip_serializing_if = "Option::is_none")]
        mcp_servers: Option<Vec<Value>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cwd: Option<String>,
        #[serde(rename = "runtimeVersion", skip_serializing_if = "Option::is_none")]
        runtime_version: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        agents: Option<Vec<String>>,
    },

    // -- Streaming text & thinking ------------------------------------------
    #[serde(rename = "message_delta")]
    MessageDelta { text: String },

    #[serde(rename = "thinking_delta")]
    ThinkingDelta { text: String },

    #[serde(rename = "stream_control")]
    StreamControl {
        action: String,
        #[serde(rename = "blockType", skip_serializing_if = "Option::is_none")]
        block_type: Option<String>,
    },

    // -- Complete messages (snapshots) --------------------------------------
    #[serde(rename = "message_complete")]
    MessageComplete {
        #[serde(rename = "messageId")]
        message_id: String,
        text: String,
    },

    #[serde(rename = "thinking_complete")]
    ThinkingComplete {
        #[serde(rename = "messageId")]
        message_id: String,
        text: String,
    },

    // -- Tool calls ---------------------------------------------------------
    #[serde(rename = "tool_call_started")]
    ToolCallStarted {
        id: String,
        name: String,
        input: Value,
        #[serde(rename = "parentToolUseId", skip_serializing_if = "Option::is_none")]
        parent_tool_use_id: Option<String>,
        #[serde(rename = "messageId")]
        message_id: String,
    },

    #[serde(rename = "tool_call_finished")]
    ToolCallFinished {
        #[serde(rename = "toolUseId")]
        tool_use_id: String,
        output: String,
        #[serde(rename = "isError")]
        is_error: bool,
    },

    // -- Permissions --------------------------------------------------------
    #[serde(rename = "approval_requested")]
    ApprovalRequested {
        #[serde(rename = "requestId")]
        request_id: String,
        #[serde(rename = "toolName", skip_serializing_if = "Option::is_none")]
        tool_name: Option<String>,
        #[serde(rename = "toolInput", skip_serializing_if = "Option::is_none")]
        tool_input: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },

    // -- Run lifecycle ------------------------------------------------------
    #[serde(rename = "run_completed")]
    RunCompleted {
        #[serde(rename = "isError")]
        is_error: bool,
        #[serde(rename = "durationMs")]
        duration_ms: u64,
        #[serde(rename = "durationApiMs")]
        duration_api_ms: u64,
        #[serde(rename = "numTurns")]
        num_turns: u32,
        result: String,
        #[serde(rename = "stopReason")]
        stop_reason: String,
        #[serde(rename = "sessionId")]
        session_id: String,
        #[serde(rename = "costUsd")]
        cost_usd: f64,
    },

    #[serde(rename = "token_usage")]
    TokenUsage {
        #[serde(rename = "inputTokens")]
        input_tokens: u64,
        #[serde(rename = "outputTokens")]
        output_tokens: u64,
        #[serde(rename = "cacheReadTokens")]
        cache_read_tokens: u64,
        #[serde(rename = "durationMs")]
        duration_ms: u64,
        #[serde(rename = "costUsd")]
        cost_usd: f64,
    },

    #[serde(rename = "rate_limited")]
    RateLimited {
        status: String,
        #[serde(rename = "resetsAt")]
        resets_at: u64,
    },

    // -- Agent delegation ---------------------------------------------------
    #[serde(rename = "agent_activity")]
    AgentActivity {
        #[serde(rename = "delegationId")]
        delegation_id: String,
    },

    // -- Catch-all ----------------------------------------------------------
    #[serde(rename = "unknown")]
    Unknown {},
}
