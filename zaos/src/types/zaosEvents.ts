/**
 * ZAOS Normalized Event Model — provider-neutral.
 *
 * Every runtime (Claude, Codex, ...) must map its native events to this model.
 * Hooks and stores consume only these types, never runtime-specific ones.
 */

// ---------------------------------------------------------------------------
// Discriminated union
// ---------------------------------------------------------------------------

export type ZaosEvent =
  | SessionInitEvent
  | MessageDeltaEvent
  | ThinkingDeltaEvent
  | StreamControlEvent
  | MessageCompleteEvent
  | ThinkingCompleteEvent
  | ToolCallStartedEvent
  | ToolCallFinishedEvent
  | ApprovalRequestedEvent
  | PolicyDecisionEvent
  | RunCompletedEvent
  | TokenUsageEvent
  | RateLimitedEvent
  | AgentActivityEvent
  | UnknownEvent;

// ---------------------------------------------------------------------------
// Session lifecycle
// ---------------------------------------------------------------------------

export interface SessionInitEvent {
  type: "session_init";
  sessionId: string;
  model?: string;
  tools?: string[];
  mcpServers?: unknown[];
  cwd?: string;
  runtimeVersion?: string;
  agents?: string[];
}

// ---------------------------------------------------------------------------
// Streaming text & thinking
// ---------------------------------------------------------------------------

export interface MessageDeltaEvent {
  type: "message_delta";
  text: string;
}

export interface ThinkingDeltaEvent {
  type: "thinking_delta";
  text: string;
}

/** Stream-level flow control signals */
export interface StreamControlEvent {
  type: "stream_control";
  action: "block_start" | "block_stop" | "message_end";
  blockType?: "text" | "thinking" | "tool_use";
}

// ---------------------------------------------------------------------------
// Complete messages (snapshots)
// ---------------------------------------------------------------------------

export interface MessageCompleteEvent {
  type: "message_complete";
  messageId: string;
  text: string;
}

export interface ThinkingCompleteEvent {
  type: "thinking_complete";
  messageId: string;
  text: string;
}

// ---------------------------------------------------------------------------
// Tool calls
// ---------------------------------------------------------------------------

export interface ToolCallStartedEvent {
  type: "tool_call_started";
  id: string;
  name: string;
  input: Record<string, unknown>;
  parentToolUseId?: string;
  messageId: string;
}

export interface ToolCallFinishedEvent {
  type: "tool_call_finished";
  toolUseId: string;
  output: string;
  isError: boolean;
}

// ---------------------------------------------------------------------------
// Permissions
// ---------------------------------------------------------------------------

export interface ApprovalRequestedEvent {
  type: "approval_requested";
  requestId: string;
  toolName?: string;
  toolInput?: Record<string, unknown>;
  description?: string;
  /** Policy Engine fields — set by ZAOS, not by the runtime provider */
  policyVerdict?: "allow" | "ask" | "deny";
  policyRiskLevel?: "low" | "medium" | "high" | "critical";
  policyReason?: string;
  policyMatchedRules?: string[];
}

// ---------------------------------------------------------------------------
// Policy Engine decisions (auto-allow / auto-deny log)
// ---------------------------------------------------------------------------

/** Logged when ZAOS Policy Engine auto-allows or auto-denies an action (not shown as permission prompt) */
export interface PolicyDecisionEvent {
  type: "policy_decision";
  toolName?: string;
  verdict: "allow" | "deny";
  riskLevel: "low" | "medium" | "high" | "critical";
  reason: string;
  matchedRules: string[];
}

// ---------------------------------------------------------------------------
// Run lifecycle
// ---------------------------------------------------------------------------

export interface RunCompletedEvent {
  type: "run_completed";
  isError: boolean;
  durationMs: number;
  durationApiMs: number;
  numTurns: number;
  result: string;
  stopReason: string;
  sessionId: string;
  costUsd: number;
}

export interface TokenUsageEvent {
  type: "token_usage";
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens: number;
  durationMs: number;
  costUsd: number;
}

export interface RateLimitedEvent {
  type: "rate_limited";
  status: string;
  resetsAt: number;
}

// ---------------------------------------------------------------------------
// Agent delegation
// ---------------------------------------------------------------------------

/** Heartbeat: an event was received from a running sub-agent */
export interface AgentActivityEvent {
  type: "agent_activity";
  delegationId: string;
}

// ---------------------------------------------------------------------------
// Catch-all
// ---------------------------------------------------------------------------

export interface UnknownEvent {
  type: "unknown";
}

// ---------------------------------------------------------------------------
// Future event placeholders (not yet emitted by any runtime)
// ---------------------------------------------------------------------------
// These types are defined for forward compatibility.
// They will be added to the ZaosEvent union when implemented.

/** An artifact (file, image, etc.) was created or modified */
export interface ArtifactUpdatedEvent {
  type: "artifact_updated";
  artifactType: "file" | "image" | "diff";
  path?: string;
  description?: string;
}

/** A proof (screenshot, test result, etc.) was added */
export interface ProofAddedEvent {
  type: "proof_added";
  proofType: "screenshot" | "test_result" | "build_output";
  path?: string;
  summary?: string;
}

/** Session summary is ready (end of run) */
export interface SessionSummaryReadyEvent {
  type: "session_summary_ready";
  summary: string;
  tokensTotal: number;
  costTotal: number;
}
