import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { CliEvent, StreamDeltaEvent, AssistantEvent, Message } from "../types/events";
import { useChatStore } from "../stores/chatStore";

/**
 * Hook for managing streaming text display
 * Accumulates text_delta events into the current message
 */
export function useStreaming() {
  const {
    messages,
    streamingTextBuffer,
    isStreaming,
    isThinking,
    addMessage,
    appendStreamText,
    setStreaming,
    setThinking,
    clearStreamingBuffer,
  } = useChatStore();

  useEffect(() => {
    let unlistener: UnlistenFn | null = null;
    let currentMessageId: string | null = null;

    const setupListener = async () => {
      unlistener = await listen<CliEvent>("agent-event", (event) => {
        const payload = event.payload;

        // Handle stream events (deltas)
        if (payload.type === "stream_event") {
          const streamEvent = payload as StreamDeltaEvent;
          const innerEvent = streamEvent.event;

          // Text delta - accumulate text
          if (innerEvent.type === "content_block_delta") {
            const delta = innerEvent.delta as any;
            if (delta.type === "text_delta" && delta.text) {
              appendStreamText(delta.text);
              setStreaming(true);
              if (!currentMessageId) {
                currentMessageId = `msg-${Date.now()}`;
              }
            }
          }

          // Thinking delta
          else if (innerEvent.type === "content_block_delta") {
            const delta = innerEvent.delta as any;
            if (delta.type === "thinking_delta") {
              setThinking(true);
            }
          }

          // Content block stop - finalize current block
          else if (innerEvent.type === "content_block_stop") {
            if (streamingTextBuffer && currentMessageId) {
              // The message will be finalized when we get the assistant event
              setStreaming(false);
            }
          }

          // Message stop - end of turn
          else if (innerEvent.type === "message_stop") {
            setStreaming(false);
            setThinking(false);
            currentMessageId = null;
          }
        }

        // Handle assistant event (complete message snapshot)
        else if (payload.type === "assistant") {
          const assistantEvent = payload as AssistantEvent;
          const msg = assistantEvent.message;

          // Extract text content from message
          const textContent = msg.content
            .filter((block: any) => block.type === "text")
            .map((block: any) => block.text)
            .join("\n");

          // Extract thinking if present
          const thinkingContent = msg.content
            .filter((block: any) => block.type === "thinking")
            .map((block: any) => block.thinking)
            .join("\n");

          if (textContent || thinkingContent) {
            const finalContent =
              streamingTextBuffer || textContent;

            const chatMessage: Message = {
              id: msg.id,
              role: "assistant",
              content: finalContent,
              timestamp: Date.now(),
            };

            addMessage(chatMessage);
            clearStreamingBuffer();
            currentMessageId = null;
          }

          setStreaming(false);
        }

        // Handle user event (tool results)
        else if (payload.type === "user") {
          const userEvent = payload;
          if ("message" in userEvent && userEvent.message) {
            const msg = userEvent.message as any;
            const resultContent =
              msg.content?.[0]?.content || "Tool executed";

            const chatMessage: Message = {
              id: `user-${Date.now()}`,
              role: "user",
              content: resultContent,
              timestamp: Date.now(),
            };

            addMessage(chatMessage);
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
  }, [
    addMessage,
    appendStreamText,
    clearStreamingBuffer,
    isStreaming,
    isThinking,
    setStreaming,
    setThinking,
    streamingTextBuffer,
  ]);
}
