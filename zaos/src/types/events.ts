/**
 * Type definitions for stream-json protocol events
 * Mapping of Claude Code CLI event types to TypeScript interfaces
 */

export type CliEvent =
  | SystemEvent
  | StreamDeltaEvent
  | AssistantEvent
  | UserEvent
  | RateLimitEvent
  | ResultEvent;

// ============================================================================
// System Event
// ============================================================================

export interface SystemEvent {
  type: "system";
  subtype: "init" | "api_retry";
  session_id: string;
  uuid: string;
  cwd?: string;
  tools?: string[];
  mcp_servers?: unknown[];
  model?: string;
  claude_code_version?: string;
  agents?: string[];
  // for api_retry subtype
  attempt?: number;
  max_retries?: number;
  error?: string;
  error_status?: number;
}

// ============================================================================
// Stream Delta Events (with --include-partial-messages)
// ============================================================================

export interface StreamDeltaEvent {
  type: "stream_event";
  event: ApiStreamEvent;
  session_id: string;
  parent_tool_use_id?: string;
  uuid: string;
}

export type ApiStreamEvent =
  | MessageStartEvent
  | ContentBlockStartEvent
  | ContentBlockDeltaEvent
  | ContentBlockStopEvent
  | MessageDeltaEvent
  | MessageStopEvent
  | UnknownStreamEvent;

export interface MessageStartEvent {
  type: "message_start";
  message: Record<string, unknown>;
}

export interface ContentBlockStartEvent {
  type: "content_block_start";
  index: number;
  content_block: {
    type: string; // "text", "tool_use", "thinking"
    text?: string;
    id?: string;
    name?: string;
  };
}

export interface ContentBlockDeltaEvent {
  type: "content_block_delta";
  index: number;
  delta: DeltaContent;
}

export type DeltaContent =
  | TextDeltaContent
  | InputJsonDeltaContent
  | ThinkingDeltaContent
  | UnknownDeltaContent;

export interface TextDeltaContent {
  type: "text_delta";
  text: string;
}

export interface InputJsonDeltaContent {
  type: "input_json_delta";
  partial_json: string;
}

export interface ThinkingDeltaContent {
  type: "thinking_delta";
  thinking: string;
}

export interface UnknownDeltaContent {
  type: string;
  [key: string]: unknown;
}

export interface ContentBlockStopEvent {
  type: "content_block_stop";
  index: number;
}

export interface MessageDeltaEvent {
  type: "message_delta";
  delta: Record<string, unknown>;
  usage?: Usage;
}

export interface MessageStopEvent {
  type: "message_stop";
}

export interface UnknownStreamEvent {
  type: string;
  [key: string]: unknown;
}

// ============================================================================
// Assistant Event
// ============================================================================

export interface AssistantEvent {
  type: "assistant";
  message: AssistantMessage;
  parent_tool_use_id?: string;
  session_id: string;
  uuid: string;
}

export interface AssistantMessage {
  id: string;
  model: string;
  content: ContentBlock[];
  stop_reason?: string;
  usage?: Usage;
}

export type ContentBlock =
  | ThinkingBlock
  | ToolUseBlock
  | TextBlock
  | UnknownBlock;

export interface ThinkingBlock {
  type: "thinking";
  thinking: string;
  signature?: string;
}

export interface ToolUseBlock {
  type: "tool_use";
  id: string;
  name: string;
  input: Record<string, unknown>;
  caller?: {
    type: string;
  };
}

export interface TextBlock {
  type: "text";
  text: string;
}

export interface UnknownBlock {
  type: string;
  [key: string]: unknown;
}

// ============================================================================
// User Event (tool result)
// ============================================================================

export interface UserEvent {
  type: "user";
  message: UserMessage;
  parent_tool_use_id?: string;
  session_id: string;
  uuid: string;
  timestamp?: string;
  tool_use_result?: Record<string, unknown>;
}

export interface UserMessage {
  role: string;
  content: ToolResultBlock[];
}

export interface ToolResultBlock {
  tool_use_id: string;
  type: string;
  content: string;
}

// ============================================================================
// Rate Limit Event
// ============================================================================

export interface RateLimitEvent {
  type: "rate_limit_event";
  rate_limit_info: RateLimitInfo;
  uuid: string;
  session_id: string;
}

export interface RateLimitInfo {
  status: string; // "allowed", "rate_limited"
  resetsAt: number;
  rateLimitType: string;
  overageStatus?: string;
  overageDisabledReason?: string;
  isUsingOverage?: boolean;
}

// ============================================================================
// Result Event
// ============================================================================

export interface ResultEvent {
  type: "result";
  subtype: string; // "success", "error"
  is_error: boolean;
  duration_ms: number;
  duration_api_ms: number;
  num_turns: number;
  result: string;
  stop_reason: string;
  session_id: string;
  total_cost_usd: number;
  usage: Usage;
  modelUsage?: Record<string, ModelUsageDetail>;
  permission_denials?: string[];
  uuid: string;
}

export interface ModelUsageDetail {
  inputTokens: number;
  outputTokens: number;
  cacheReadInputTokens?: number;
  cacheCreationInputTokens?: number;
  costUSD: number;
  contextWindow: number;
  maxOutputTokens: number;
}

// ============================================================================
// Common Types
// ============================================================================

export interface Usage {
  input_tokens: number;
  output_tokens: number;
  cache_creation_input_tokens?: number;
  cache_read_input_tokens?: number;
}

export interface SessionInfo {
  sessionId: string;
  model: string;
  cwd: string;
  tools: string[];
  mcpServers: string[];
  agents: string[];
  version: string;
  startTime: number;
}

export interface Message {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
  agent?: string; // Agent name if present
  timestamp: number;
  isStreaming?: boolean;
}

export interface Action {
  id: string;
  type: string; // "read", "write", "edit", "bash", "glob", "grep", "screenshot", etc.
  tool: string; // Tool name
  summary: string; // Short summary for display
  timestamp: number;
  status: "pending" | "running" | "success" | "error";
  details?: Record<string, unknown>;
}
