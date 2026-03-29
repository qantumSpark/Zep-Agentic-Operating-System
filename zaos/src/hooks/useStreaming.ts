import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { CliEvent, StreamDeltaEvent, AssistantEvent, Message } from "../types/events";
import { useChatStore } from "../stores/chatStore";

/**
 * Hook for managing streaming text display.
 * Listener is registered ONCE (empty deps).
 * Uses getState() to avoid stale closures and re-registrations.
 */
export function useStreaming() {
  useEffect(() => {
    let unlistener: UnlistenFn | null = null;
    let currentMessageId: string | null = null;
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
            const delta = innerEvent.delta as any;
            if (delta.type === "text_delta" && delta.text) {
              store.appendStreamText(delta.text);
              store.setStreaming(true);
              if (!currentMessageId) {
                currentMessageId = `msg-${Date.now()}`;
              }
            } else if (delta.type === "thinking_delta") {
              store.setThinking(true);
            }
          } else if (innerEvent.type === "content_block_stop") {
            store.setStreaming(false);
          } else if (innerEvent.type === "message_stop") {
            store.setStreaming(false);
            store.setThinking(false);
            currentMessageId = null;
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

          const textContent = msg.content
            .filter((block: any) => block.type === "text")
            .map((block: any) => block.text)
            .join("\n");

          if (textContent) {
            const currentBuffer = store.getStreamingTextBuffer();
            const finalContent = currentBuffer || textContent;

            const chatMessage: Message = {
              id: msg.id,
              role: "assistant",
              content: finalContent,
              timestamp: Date.now(),
            };

            store.addMessage(chatMessage);
            store.clearStreamingBuffer();
            currentMessageId = null;
          }

          store.setStreaming(false);
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