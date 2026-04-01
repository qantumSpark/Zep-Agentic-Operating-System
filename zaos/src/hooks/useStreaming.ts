import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { CliEvent, Message } from "../types/events";
import type { ApprovalRequestedEvent } from "../types/zaosEvents";
import { mapClaudeEvent } from "../adapters/claudeMapper";
import { useChatStore } from "../stores/chatStore";
import { useActionsStore } from "../stores/actionsStore";
import { useAgentsStore, mapAgentName } from "../stores/agentsStore";
import { usePermissionStore } from "../stores/permissionStore";
import { useSessionStore } from "../stores/sessionStore";
import { formatToolSummary } from "../utils/toolFormatters";

/**
 * Hook for managing streaming text display.
 * Listener is registered ONCE (empty deps).
 * Uses getState() to avoid stale closures and re-registrations.
 */
export function useStreaming() {
  useEffect(() => {
    let unlistener: UnlistenFn | null = null;
    // Track which message IDs we've already added
    const seenMessageIds = new Set<string>();
    // Track which individual content block IDs have been processed
    const seenBlockIds = new Set<string>();

    const setupListener = async () => {
      unlistener = await listen<CliEvent>("agent-event", (event) => {
        const store = useChatStore.getState();
        const payload = event.payload;
        const zaosEvents = mapClaudeEvent(payload);

        for (const ze of zaosEvents) {
          switch (ze.type) {
            case "session_init": {
              const actionsStore = useActionsStore.getState();
              actionsStore.clearActions();
              seenMessageIds.clear();
              seenBlockIds.clear();

              // Capture available agents from system init
              const agentsStore = useAgentsStore.getState();
              agentsStore.reset();
              if (ze.agents && ze.agents.length > 0) {
                agentsStore.setAvailableAgents(ze.agents);
              }
              break;
            }

            case "agent_activity": {
              const agentsStore = useAgentsStore.getState();
              if (agentsStore.delegations.some((d) => d.id === ze.delegationId && d.status === "running")) {
                agentsStore.updateLastSeen(ze.delegationId);
              }
              break;
            }

            case "stream_control": {
              if (ze.action === "block_start" && ze.blockType === "thinking") {
                store.setThinking(true);
              } else if (ze.action === "block_stop") {
                store.setStreaming(false);
                store.setThinking(false);
              } else if (ze.action === "message_end") {
                store.setStreaming(false);
                store.setThinking(false);
              }
              break;
            }

            case "message_delta": {
              store.appendStreamText(ze.text);
              store.setStreaming(true);
              break;
            }

            case "thinking_delta": {
              store.setThinking(true);
              break;
            }

            case "message_complete": {
              seenMessageIds.add(ze.messageId);

              const chatMessage: Message = {
                id: ze.messageId,
                role: "assistant",
                content: ze.text,
                timestamp: Date.now(),
              };
              // addMessage has internal dedup — no-op if msg.id already exists
              store.addMessage(chatMessage);
              // updateMessage refreshes content if message exists (for subsequent snapshots)
              store.updateMessage(ze.messageId, { content: ze.text });
              store.clearStreamingBuffer();
              break;
            }

            case "thinking_complete": {
              if (!seenBlockIds.has(`${ze.messageId}-thinking`)) {
                seenBlockIds.add(`${ze.messageId}-thinking`);
                const thinkMsg: Message = {
                  id: `${ze.messageId}-thinking`,
                  role: "assistant",
                  content: "",
                  timestamp: Date.now(),
                  thinking: ze.text,
                };
                store.addMessage(thinkMsg);
              }
              break;
            }

            case "tool_call_started": {
              if (!seenBlockIds.has(ze.id)) {
                seenBlockIds.add(ze.id);
                const toolMsg: Message = {
                  id: `${ze.messageId}-tool-${ze.id}`,
                  role: "assistant",
                  content: "",
                  timestamp: Date.now(),
                  toolUse: {
                    id: ze.id,
                    name: ze.name,
                    input: ze.input,
                  },
                };
                store.addMessage(toolMsg);

                // Feed actionsStore
                const actionsStore = useActionsStore.getState();
                actionsStore.addAction({
                  id: ze.id,
                  type: ze.name.toLowerCase(),
                  tool: ze.name,
                  summary: formatToolSummary(ze.name, ze.input),
                  timestamp: Date.now(),
                  status: "running",
                  details: ze.input,
                  parentId: ze.parentToolUseId,
                });

                // Detect agent delegation (tool_use with name "Agent")
                if (ze.name === "Agent") {
                  const agentInput = ze.input as Record<string, unknown>;
                  const agentsStore = useAgentsStore.getState();
                  const subagentType = String(agentInput.subagent_type ?? "general-purpose");
                  const description = String(agentInput.description ?? "");
                  const prompt = agentInput.prompt ? String(agentInput.prompt) : undefined;
                  agentsStore.addDelegation({
                    id: ze.id,
                    agentType: subagentType,
                    mappedAgent: mapAgentName(subagentType, description, prompt),
                    description,
                    startedAt: Date.now(),
                  });
                }
              }
              break;
            }

            case "tool_call_finished": {
              // Update action status + result preview
              const actionsStore = useActionsStore.getState();
              actionsStore.updateAction(ze.toolUseId, {
                status: ze.isError ? "error" : "success",
                resultPreview: ze.output.slice(0, 200),
              });

              // Complete agent delegation if this tool_result matches one
              const agentsStore = useAgentsStore.getState();
              const matchingDelegation = agentsStore.delegations.find(
                (d) => d.id === ze.toolUseId && d.status === "running"
              );
              if (matchingDelegation) {
                const duration = Date.now() - matchingDelegation.startedAt;
                agentsStore.completeDelegation(
                  ze.toolUseId,
                  ze.isError ? "error" : "completed"
                );
                useSessionStore.getState().recordAgentTiming(
                  matchingDelegation.mappedAgent,
                  duration
                );
              }

              // Add tool result message to chat
              const resultMsg: Message = {
                id: `result-${ze.toolUseId}`,
                role: "system",
                content: "",
                timestamp: Date.now(),
                toolResult: {
                  toolUseId: ze.toolUseId,
                  content: ze.output.slice(0, 500),
                  isError: ze.isError,
                },
              };
              store.addMessage(resultMsg);
              break;
            }

            case "approval_requested": {
              // Add to permission store
              const permStore = usePermissionStore.getState();
              permStore.addRequest(ze as ApprovalRequestedEvent);

              // Add a system message to chat with the permission request
              const permMsg: Message = {
                id: `perm-${ze.requestId}`,
                role: "system",
                content: "",
                timestamp: Date.now(),
                permissionRequest: ze as ApprovalRequestedEvent,
              };
              store.addMessage(permMsg);
              break;
            }

            case "unknown":
            default:
              // no-op
              break;
          }
        }
      });
    };

    setupListener();

    return () => {
      if (unlistener) {
        unlistener();
      }
    };
  }, []); // Empty deps — listener registered once
}
