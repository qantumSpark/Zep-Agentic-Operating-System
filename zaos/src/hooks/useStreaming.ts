import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { CliEvent, StreamDeltaEvent, AssistantEvent, UserEvent, Message } from "../types/events";
import { useChatStore } from "../stores/chatStore";
import { useActionsStore } from "../stores/actionsStore";
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

    const setupListener = async () => {
      unlistener = await listen<CliEvent>("agent-event", (event) => {
        const store = useChatStore.getState();
        const payload = event.payload;
        // Handle stream events (deltas)
        if (payload.type === "stream_event") {
          const streamEvent = payload as StreamDeltaEvent;
          const innerEvent = streamEvent.event;

          if (innerEvent.type === "content_block_delta") {
            const delta = innerEvent.delta;
            if (delta.type === "text_delta") {
              store.appendStreamText(delta.text);
              store.setStreaming(true);
            } else if (delta.type === "thinking_delta") {
              store.setThinking(true);
            }
          } else if (innerEvent.type === "content_block_stop") {
            store.setStreaming(false);
          } else if (innerEvent.type === "message_stop") {
            store.setStreaming(false);
            store.setThinking(false);
          }
        }
        // Handle assistant event (complete message snapshot)
        else if (payload.type === "assistant") {
          const assistantEvent = payload as AssistantEvent;
          const msg = assistantEvent.message;

          // Deduplicate: skip if we already added this message
          if (seenMessageIds.has(msg.id)) {
            return;
          }
          seenMessageIds.add(msg.id);

          // Process each content block
          for (const block of msg.content) {
            if (block.type === "text" && block.text) {
              const finalContent = block.text;

              const chatMessage: Message = {
                id: msg.id,
                role: "assistant",
                content: finalContent,
                timestamp: Date.now(),
              };
              store.addMessage(chatMessage);
              store.clearStreamingBuffer();
            } else if (block.type === "tool_use") {
              // Create a chat message for the tool use
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
            } else if (block.type === "thinking" && block.thinking) {
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

          store.setStreaming(false);
          store.setThinking(false);
        }
        // Handle user event (tool results)
        else if (payload.type === "user") {
          const userEvent = payload as UserEvent;
          const userMsg = userEvent.message;

          if (userMsg.content) {
            for (const result of userMsg.content) {
              if (result.tool_use_id) {
                // Update action status
                const actionsStore = useActionsStore.getState();
                const isError = result.type === "tool_error";
                actionsStore.updateActionStatus(
                  result.tool_use_id,
                  isError ? "error" : "success"
                );

                // Add tool result message to chat
                const resultMsg: Message = {
                  id: `result-${result.tool_use_id}`,
                  role: "system",
                  content: "",
                  timestamp: Date.now(),
                  toolResult: {
                    toolUseId: result.tool_use_id,
                    content: typeof result.content === "string"
                      ? result.content.slice(0, 500)
                      : JSON.stringify(result.content).slice(0, 500),
                    isError,
                  },
                };
                store.addMessage(resultMsg);
              }
            }
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