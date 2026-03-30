import React, { useEffect, useRef } from "react";
import { useChatStore } from "../../stores/chatStore";
import { MessageBubble } from "./MessageBubble";
import { InputBar } from "./InputBar";
import { ThinkingIndicator } from "./ThinkingIndicator";

/**
 * Chat panel: messages list + input bar at bottom
 * Auto-scrolls on new messages
 */
export function ChatPanel() {
  const messages = useChatStore((state) => state.messages);
  const isThinking = useChatStore((state) => state.isThinking);
  const streamingText = useChatStore((state) => state.streamingTextBuffer);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, streamingText]);

  return (
    <div className="flex flex-col h-full bg-zinc-900">
      {/* Messages area */}
      <div className="flex-1 overflow-y-auto px-4 py-4 space-y-4">
        {messages.length === 0 ? (
          <div className="h-full flex items-center justify-center text-zinc-400">
            <p>No messages yet. Start by typing a message below.</p>
          </div>
        ) : (
          <>
            {messages.map((message) => (
              <MessageBubble key={message.id} message={message} />
            ))}
            {streamingText && (
              <MessageBubble
                message={{
                  id: "streaming",
                  role: "assistant",
                  content: streamingText,
                  timestamp: Date.now(),
                  isStreaming: true,
                }}
              />
            )}
            {isThinking && !streamingText && <ThinkingIndicator />}
            <div ref={messagesEndRef} />
          </>
        )}
      </div>

      {/* Input area */}
      <div className="border-t border-zinc-700 bg-zinc-800/50 p-4">
        <InputBar />
      </div>
    </div>
  );
}
