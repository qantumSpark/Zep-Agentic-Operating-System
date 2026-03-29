import React, { useRef, useState } from "react";
import { Send, Square } from "lucide-react";
import { useChatStore } from "../../stores/chatStore";
import { invoke } from "@tauri-apps/api/core";

/**
 * Text input: multiline textarea
 * Enter to send, Shift+Enter for newline
 * Stop button when streaming
 */
export function InputBar() {
  const [text, setText] = useState("");
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const isStreaming = useChatStore((state) => state.isStreaming);
  const addMessage = useChatStore((state) => state.addMessage);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    // Enter to send
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
    // Shift+Enter for newline
    else if (e.key === "Enter" && e.shiftKey) {
      // Let default behavior work
    }
    // Escape to clear
    else if (e.key === "Escape") {
      setText("");
    }
  };

  const handleSend = async () => {
    if (!text.trim()) return;

    // Add user message to chat
    addMessage({
      id: `user-${Date.now()}`,
      role: "user",
      content: text,
      timestamp: Date.now(),
    });

    // Send to backend
    try {
      await invoke("send_prompt", { text });
    } catch (error) {
      console.error("Failed to send prompt:", error);
    }

    setText("");
  };

  const handleStop = async () => {
    try {
      await invoke("interrupt_cli");
    } catch (error) {
      console.error("Failed to stop CLI:", error);
    }
  };

  return (
    <div className="flex gap-2 items-flex-end">
      <textarea
        ref={textareaRef}
        value={text}
        onChange={(e) => setText(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder="Type your message... (Shift+Enter for newline, Enter to send)"
        className="flex-1 bg-zinc-800 border border-zinc-600 rounded px-3 py-2 text-white placeholder-zinc-400 resize-none focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500/50 max-h-32"
        rows={2}
      />

      {isStreaming ? (
        <button
          onClick={handleStop}
          className="bg-red-600 hover:bg-red-700 text-white rounded px-4 py-2 flex items-center gap-2 transition-colors flex-shrink-0"
        >
          <Square size={16} />
          Stop
        </button>
      ) : (
        <button
          onClick={handleSend}
          disabled={!text.trim()}
          className="bg-blue-600 hover:bg-blue-700 text-white rounded px-4 py-2 flex items-center gap-2 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex-shrink-0"
        >
          <Send size={16} />
          Send
        </button>
      )}
    </div>
  );
}
