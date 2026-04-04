import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Message } from "../types/events";
import type { ZaosEvent, ApprovalRequestedEvent, PolicyDecisionEvent } from "../types/zaosEvents";
import { useChatStore } from "../stores/chatStore";
import { useActionsStore } from "../stores/actionsStore";
import { useAgentsStore, mapAgentName } from "../stores/agentsStore";
import { usePermissionStore } from "../stores/permissionStore";
import type { PolicyLogEntry } from "../stores/permissionStore";
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
      unlistener = await listen<ZaosEvent>("agent-event", (event) => {
        const store = useChatStore.getState();
        const ze = event.payload;

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
            store.addMessage(chatMessage);
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
                  input: ze.input as Record<string, unknown>,
                },
              };
              store.addMessage(toolMsg);

              const actionsStore = useActionsStore.getState();
              actionsStore.addAction({
                id: ze.id,
                type: ze.name.toLowerCase(),
                tool: ze.name,
                summary: formatToolSummary(ze.name, ze.input as Record<string, unknown>),
                timestamp: Date.now(),
                status: "running",
                details: ze.input as Record<string, unknown>,
                parentId: ze.parentToolUseId,
              });

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
            const actionsStore = useActionsStore.getState();
            actionsStore.updateAction(ze.toolUseId, {
              status: ze.isError ? "error" : "success",
              resultPreview: ze.output.slice(0, 200),
            });

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
            const permStore = usePermissionStore.getState();
            const approvalEvent = ze as ApprovalRequestedEvent;
            permStore.addRequest(approvalEvent);

            const permMsg: Message = {
              id: `perm-${ze.requestId}`,
              role: "system",
              content: "",
              timestamp: Date.now(),
              permissionRequest: approvalEvent,
            };
            store.addMessage(permMsg);
            break;
          }

          case "policy_decision": {
            const policyEvent = ze as PolicyDecisionEvent;
            const permStore = usePermissionStore.getState();
            const logEntry: PolicyLogEntry = {
              timestamp: Date.now(),
              toolName: policyEvent.toolName ?? "unknown",
              verdict: policyEvent.verdict,
              riskLevel: policyEvent.riskLevel,
              reason: policyEvent.reason,
              wasAutoApproved: policyEvent.verdict === "allow",
              wasAutoDenied: policyEvent.verdict === "deny",
            };
            permStore.addPolicyLogEntry(logEntry);
            break;
          }

          case "unknown":
          default:
            break;
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
