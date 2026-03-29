import React from "react";

/**
 * Animated "thinking" indicator shown when Claude is thinking
 */
export function ThinkingIndicator() {
  return (
    <div className="flex items-center gap-2 text-zinc-400">
      <span className="animate-pulse text-base" role="img" aria-label="brain">&#x1F9E0;</span>
      <span className="text-sm italic">Claude is thinking...</span>
      <div className="flex gap-1">
        <div className="w-1 h-1 bg-blue-500 rounded-full animate-bounce" style={{ animationDelay: "0ms" }} />
        <div className="w-1 h-1 bg-blue-500 rounded-full animate-bounce" style={{ animationDelay: "150ms" }} />
        <div className="w-1 h-1 bg-blue-500 rounded-full animate-bounce" style={{ animationDelay: "300ms" }} />
      </div>
    </div>
  );
}
