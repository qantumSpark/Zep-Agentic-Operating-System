import React, { useRef, useState } from "react";
import { useChatStore } from "../../stores/chatStore";
import { invoke } from "@tauri-apps/api/core";

export function InputBar() {
  const [text, setText] = useState("");
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const isStreaming = useChatStore((state) => state.isStreaming);
  const addMessage = useChatStore((state) => state.addMessage);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleSend = async () => {
    if (!text.trim()) return;
    addMessage({
      id: `user-${Date.now()}`,
      role: "user",
      content: text,
      timestamp: Date.now(),
    });
    try {
      const result = await invoke("send_prompt", { text });
      console.log("send_prompt result:", result);
    } catch (error: unknown) {
      console.error("Failed to send prompt:", error);
      // Show error as a chat message so user can see it
      addMessage({
        id: `error-${Date.now()}`,
        role: "assistant",
        content: `[ERROR] ${error instanceof Error ? error.message : String(error)}`,
        timestamp: Date.now(),
      });
    }
    setText("");
  };

  const handleStop = async () => {
    try {
      await invoke("interrupt_session");
    } catch (error) {
      console.error("Failed to stop CLI:", error);
    }
  };

  return (
    <div className="flex gap-2 items-end">
      <textarea
        id="main-input"
        ref={textareaRef}
        value={text}
        onChange={(e) => setText(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder="Message... (Shift+Enter for newline)"
        className="flex-1 bg-zinc-800 border border-zinc-600 rounded px-3 py-2 text-white placeholder-zinc-400 resize-none focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500/50 max-h-32"
        rows={2}
      />
      {isStreaming ? (
        <button
          onClick={handleStop}
          className="bg-red-600 hover:bg-red-700 text-white rounded px-4 py-2 flex items-center gap-2 transition-colors flex-shrink-0"
        >
          ■ Stop
        </button>
      ) : (
        <button
          onClick={handleSend}
          disabled={!text.trim()}
          className="bg-blue-600 hover:bg-blue-700 text-white rounded px-4 py-2 flex items-center gap-2 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex-shrink-0"
        >
          ➤ Send
        </button>
      )}
    </div>
  );
}