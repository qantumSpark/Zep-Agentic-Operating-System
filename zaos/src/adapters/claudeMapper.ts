/**
 * Claude Code CLI → ZAOS Event Mapper
 *
 * Converts Claude-specific CliEvent payloads into provider-neutral ZaosEvent[].
 * This is the ONLY file that should import from types/events.ts.
 * All other frontend code should consume types/zaosEvents.ts.
 */

import type {
  CliEvent,
  SystemEvent,
  StreamDeltaEvent,
  AssistantEvent,
  UserEvent,
  ControlRequest,
  RateLimitEvent,
  ResultEvent,
} from "../types/events";
import type { ZaosEvent } from "../types/zaosEvents";

/**
 * Map a single Claude CLI event to zero or more ZAOS events.
 *
 * One Claude event can produce multiple ZAOS events. For example,
 * an AssistantEvent with 3 content blocks produces 3 separate events.
 */
export function mapClaudeEvent(payload: CliEvent): ZaosEvent[] {
  switch (payload.type) {
    case "system":
      return mapSystemEvent(payload as SystemEvent);
    case "stream_event":
      return mapStreamEvent(payload as StreamDeltaEvent);
    case "assistant":
      return mapAssistantEvent(payload as AssistantEvent);
    case "user":
      return mapUserEvent(payload as UserEvent);
    case "control_request":
      return mapControlRequest(payload as ControlRequest);
    case "rate_limit_event":
      return mapRateLimitEvent(payload as RateLimitEvent);
    case "result":
      return mapResultEvent(payload as ResultEvent);
    default:
      return [{ type: "unknown" }];
  }
}

/**
 * Extract parent_tool_use_id from any event payload (delegation heartbeat).
 * Returns the ID if present, undefined otherwise.
 */
export function extractParentToolUseId(payload: CliEvent): string | undefined {
  return (payload as Record<string, unknown>).parent_tool_use_id as string | undefined;
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

function mapSystemEvent(event: SystemEvent): ZaosEvent[] {
  if (event.subtype !== "init") return [{ type: "unknown" }];
  return [
    {
      type: "session_init",
      sessionId: event.session_id,
      model: event.model,
      tools: event.tools,
      mcpServers: event.mcp_servers,
      cwd: event.cwd,
      runtimeVersion: event.claude_code_version,
      agents: event.agents,
    },
  ];
}

// ---------------------------------------------------------------------------
// Streaming deltas
// ---------------------------------------------------------------------------

function mapStreamEvent(event: StreamDeltaEvent): ZaosEvent[] {
  const inner = event.event;
  const events: ZaosEvent[] = [];

  // Agent activity heartbeat
  if (event.parent_tool_use_id) {
    events.push({ type: "agent_activity", delegationId: event.parent_tool_use_id });
  }

  switch (inner.type) {
    case "content_block_start": {
      const startEvt = inner as import("../types/events").ContentBlockStartEvent;
      const blockType = startEvt.content_block?.type as "text" | "thinking" | "tool_use" | undefined;
      events.push({ type: "stream_control", action: "block_start", blockType });
      break;
    }
    case "content_block_delta": {
      const deltaEvt = inner as import("../types/events").ContentBlockDeltaEvent;
      const delta = deltaEvt.delta;
      if (delta.type === "text_delta") {
        const td = delta as import("../types/events").TextDeltaContent;
        events.push({ type: "message_delta", text: td.text });
      } else if (delta.type === "thinking_delta") {
        const tk = delta as import("../types/events").ThinkingDeltaContent;
        events.push({ type: "thinking_delta", text: tk.thinking });
      }
      break;
    }
    case "content_block_stop":
      events.push({ type: "stream_control", action: "block_stop" });
      break;
    case "message_stop":
      events.push({ type: "stream_control", action: "message_end" });
      break;
    default:
      break;
  }

  return events.length > 0 ? events : [{ type: "unknown" }];
}

// ---------------------------------------------------------------------------
// Assistant (complete snapshot)
// ---------------------------------------------------------------------------

function mapAssistantEvent(event: AssistantEvent): ZaosEvent[] {
  const msg = event.message;
  const events: ZaosEvent[] = [];

  // Agent activity heartbeat
  if (event.parent_tool_use_id) {
    events.push({ type: "agent_activity", delegationId: event.parent_tool_use_id });
  }

  for (const block of msg.content) {
    switch (block.type) {
      case "text": {
        const textBlock = block as import("../types/events").TextBlock;
        if (textBlock.text) {
          events.push({
            type: "message_complete",
            messageId: msg.id,
            text: textBlock.text,
          });
        }
        break;
      }

      case "tool_use": {
        const toolBlock = block as import("../types/events").ToolUseBlock;
        events.push({
          type: "tool_call_started",
          id: toolBlock.id,
          name: toolBlock.name,
          input: toolBlock.input,
          parentToolUseId: event.parent_tool_use_id || undefined,
          messageId: msg.id,
        });
        break;
      }

      case "thinking": {
        const thinkingBlock = block as import("../types/events").ThinkingBlock;
        if (thinkingBlock.thinking) {
          events.push({
            type: "thinking_complete",
            messageId: msg.id,
            text: thinkingBlock.thinking,
          });
        }
        break;
      }

      default:
        break;
    }
  }

  return events.length > 0 ? events : [{ type: "unknown" }];
}

// ---------------------------------------------------------------------------
// User (tool results)
// ---------------------------------------------------------------------------

function mapUserEvent(event: UserEvent): ZaosEvent[] {
  const events: ZaosEvent[] = [];

  if (event.parent_tool_use_id) {
    events.push({ type: "agent_activity", delegationId: event.parent_tool_use_id });
  }

  if (event.message?.content) {
    for (const block of event.message.content) {
      if (block.type === "tool_result" && "tool_use_id" in block) {
        const contentStr =
          typeof block.content === "string"
            ? block.content
            : JSON.stringify(block.content);
        const isError =
          block.is_error === true ||
          /^(Error|error|ERROR)[:\s]/.test(contentStr) ||
          /^(?:ENOENT|EACCES|EPERM|EISDIR)\b/.test(contentStr);

        events.push({
          type: "tool_call_finished",
          toolUseId: block.tool_use_id,
          output: contentStr,
          isError,
        });
      }
    }
  }

  return events.length > 0 ? events : [{ type: "unknown" }];
}

// ---------------------------------------------------------------------------
// Control request (permissions)
// ---------------------------------------------------------------------------

function mapControlRequest(req: ControlRequest): ZaosEvent[] {
  const nested = (req as Record<string, unknown>).request as
    | { tool_name?: string; input?: Record<string, unknown>; description?: string }
    | undefined;

  return [
    {
      type: "approval_requested",
      requestId: req.request_id,
      toolName: nested?.tool_name || req.tool || undefined,
      toolInput: (nested?.input || req.input || undefined) as Record<string, unknown> | undefined,
      description: nested?.description || req.message || undefined,
    },
  ];
}

// ---------------------------------------------------------------------------
// Rate limit
// ---------------------------------------------------------------------------

function mapRateLimitEvent(event: RateLimitEvent): ZaosEvent[] {
  return [
    {
      type: "rate_limited",
      status: event.rate_limit_info.status,
      resetsAt: event.rate_limit_info.resetsAt,
    },
  ];
}

// ---------------------------------------------------------------------------
// Result (end of run)
// ---------------------------------------------------------------------------

function mapResultEvent(event: ResultEvent): ZaosEvent[] {
  const events: ZaosEvent[] = [
    {
      type: "run_completed",
      isError: event.is_error,
      durationMs: event.duration_ms,
      durationApiMs: event.duration_api_ms,
      numTurns: event.num_turns,
      result: event.result,
      stopReason: event.stop_reason,
      sessionId: event.session_id,
      costUsd: event.total_cost_usd,
    },
  ];

  if (event.usage) {
    events.push({
      type: "token_usage",
      inputTokens: event.usage.input_tokens || 0,
      outputTokens: event.usage.output_tokens || 0,
      cacheReadTokens: event.usage.cache_read_input_tokens || 0,
      durationMs: event.duration_ms,
      costUsd: event.total_cost_usd,
    });
  }

  return events;
}
