import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { CliEvent, StreamDeltaEvent, AssistantEvent, UserEvent, ControlRequest, SystemEvent, Message } from "../types/events";
import { useChatStore } from "../stores/chatStore";
import { useActionsStore } from "../stores/actionsStore";
import { usePermissionStore } from "../stores/permissionStore";
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
        // Handle system init — clear actions for new session
        if (payload.type === "system") {
          const systemEvent = payload as SystemEvent;
          if (systemEvent.subtype === "init") {
            const actionsStore = useActionsStore.getState();
            actionsStore.clearActions();
            seenMessageIds.clear();
            seenBlockIds.clear();
          }
        }
        // Handle stream events (deltas)
        else if (payload.type === "stream_event") {
          const streamEvent = payload as StreamDeltaEvent;
          const innerEvent = streamEvent.event;

          if (innerEvent.type === "content_block_start") {
            if (innerEvent.content_block?.type === "thinking") {
              store.setThinking(true);
            }
          } else if (innerEvent.type === "content_block_delta") {
            const delta = innerEvent.delta;
            if (delta.type === "text_delta") {
              store.appendStreamText(delta.text);
              store.setStreaming(true);
            } else if (delta.type === "thinking_delta") {
              store.setThinking(true);
            }
          } else if (innerEvent.type === "content_block_stop") {
            store.setStreaming(false);
            store.setThinking(false);
          } else if (innerEvent.type === "message_stop") {
            store.setStreaming(false);
            store.setThinking(false);
          }
        }
        // Handle assistant event (complete message snapshot)
        else if (payload.type === "assistant") {
          const assistantEvent = payload as AssistantEvent;
          const msg = assistantEvent.message;

          seenMessageIds.add(msg.id);

          // Process each content block
          for (const block of msg.content) {
            if (block.type === "text" && block.text) {
              const chatMessage: Message = {
                id: msg.id,
                role: "assistant",
                content: block.text,
                timestamp: Date.now(),
              };
              // addMessage has internal dedup — no-op if msg.id already exists
              store.addMessage(chatMessage);
              // updateMessage refreshes content if message exists (for subsequent snapshots)
              store.updateMessage(msg.id, { content: block.text });
              store.clearStreamingBuffer();
            } else if (block.type === "tool_use" && !seenBlockIds.has(block.id)) {
              seenBlockIds.add(block.id);
              const toolMsg: Message = {
                id: `${msg.id}-tool-${block.id}`,
                role: "assistant",
                content: "",
                timestamp: Date.now(),
                toolUse: {
                  id: block.id,
                  name: block.name,
                  input: block.input,
                },
              };
              store.addMessage(toolMsg);

              // Feed actionsStore
              const actionsStore = useActionsStore.getState();
              actionsStore.addAction({
                id: block.id,
                type: block.name.toLowerCase(),
                tool: block.name,
                summary: formatToolSummary(block.name, block.input),
                timestamp: Date.now(),
                status: "running",
                details: block.input,
              });
            } else if (block.type === "thinking" && block.thinking && !seenBlockIds.has(`${msg.id}-thinking`)) {
              seenBlockIds.add(`${msg.id}-thinking`);
              const thinkMsg: Message = {
                id: `${msg.id}-thinking`,
                role: "assistant",
                content: "",
                timestamp: Date.now(),
                thinking: block.thinking,
              };
              store.addMessage(thinkMsg);
            }
          }
        }
        // Handle user event (tool results)
        else if (payload.type === "user") {
          const userEvent = payload as UserEvent;
          const userMsg = userEvent.message;

          if (userMsg.content) {
            for (const block of userMsg.content) {
              if (block.type === "tool_result") {
                const contentStr = typeof block.content === "string"
                  ? block.content
                  : JSON.stringify(block.content);
                const isError = block.is_error === true
                  || /^(Error|error|ERROR)[:\s]/.test(contentStr)
                  || /^(?:ENOENT|EACCES|EPERM|EISDIR)\b/.test(contentStr);

                // Update action status + result preview
                const actionsStore = useActionsStore.getState();
                actionsStore.updateAction(block.tool_use_id, {
                  status: isError ? "error" : "success",
                  resultPreview: contentStr.slice(0, 200),
                });

                // Add tool result message to chat
                const resultMsg: Message = {
                  id: `result-${block.tool_use_id}`,
                  role: "system",
                  content: "",
                  timestamp: Date.now(),
                  toolResult: {
                    toolUseId: block.tool_use_id,
                    content: contentStr.slice(0, 500),
                    isError,
                  },
                };
                store.addMessage(resultMsg);
              }
            }
          }
        }
        // Handle control_request (permission prompt)
        else if (payload.type === "control_request") {
          const controlReq = payload as ControlRequest;

          // Add to permission store
          const permStore = usePermissionStore.getState();
          permStore.addRequest(controlReq);

          // Add a system message to chat with the permission request
          const permMsg: Message = {
            id: `perm-${controlReq.request_id}`,
            role: "system",
            content: "",
            timestamp: Date.now(),
            permissionRequest: controlReq,
          };
          store.addMessage(permMsg);
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