import React, { useEffect, useState } from "react";
import { useSessionStore } from "../../stores/sessionStore";
import { useWorkflowStore } from "../../stores/workflowStore";
import { Zap, Clock, Link2 } from "lucide-react";

/**
 * Bottom status bar: tokens, duration, connections, mode
 */
export function StatusBar() {
  const tokens = useSessionStore((state) => state.tokens);
  const duration = useSessionStore((state) => state.duration);
  const connections = useSessionStore((state) => state.connections);
  const model = useSessionStore((state) => state.model);
  const mode = useWorkflowStore((state) => state.mode);
  const [displayDuration, setDisplayDuration] = useState(duration);

  // Update display duration every second
  useEffect(() => {
    const interval = setInterval(() => {
      setDisplayDuration((prev) => prev + 1);
    }, 1000);
    return () => clearInterval(interval);
  }, []);

  const formatDuration = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;

    if (hours > 0) {
      return `${hours}h ${minutes}m ${secs}s`;
    }
    if (minutes > 0) {
      return `${minutes}m ${secs}s`;
    }
    return `${secs}s`;
  };

  const totalTokens = tokens.input + tokens.output;
  const maxTokens = 200000; // Plan Max context window
  const tokenPercent = Math.round((totalTokens / maxTokens) * 100);

  return (
    <div className="flex items-center justify-between px-4 py-2 bg-zinc-800 border-t border-zinc-700 text-xs text-zinc-300 gap-4">
      {/* Token usage */}
      <div className="flex items-center gap-2 min-w-0">
        <Zap size={14} className="text-yellow-500 flex-shrink-0" />
        <div className="flex gap-1">
          <span className="font-medium">{tokenPercent}%</span>
          <span className="text-zinc-500">
            {totalTokens.toLocaleString()}/{maxTokens.toLocaleString()} tokens
          </span>
          {tokens.cache > 0 && (
            <span className="text-blue-400">
              (cache: {tokens.cache.toLocaleString()})
            </span>
          )}
        </div>
      </div>

      {/* Divider */}
      <div className="w-px h-4 bg-zinc-600" />

      {/* Session duration */}
      <div className="flex items-center gap-2 min-w-0">
        <Clock size={14} className="text-blue-400 flex-shrink-0" />
        <span>{formatDuration(displayDuration)}</span>
      </div>

      {/* Divider */}
      <div className="w-px h-4 bg-zinc-600" />

      {/* Connections */}
      <div className="flex items-center gap-3 min-w-0">
        <div className="flex items-center gap-1">
          <Link2 size={12} className="flex-shrink-0" />
          <div className="flex gap-1.5">
            <div
              title="Claude Code CLI"
              className={`w-2 h-2 rounded-full flex-shrink-0 ${
                connections.cli ? "bg-green-500" : "bg-red-500"
              }`}
            />
            <div
              title="GoPeak"
              className={`w-2 h-2 rounded-full flex-shrink-0 ${
                connections.gopeak ? "bg-green-500" : "bg-red-500"
              }`}
            />
            <div
              title="Godot"
              className={`w-2 h-2 rounded-full flex-shrink-0 ${
                connections.godot ? "bg-green-500" : "bg-red-500"
              }`}
            />
          </div>
        </div>
      </div>

      {/* Divider */}
      <div className="w-px h-4 bg-zinc-600" />

      {/* Model and mode */}
      <div className="flex items-center gap-2 text-zinc-400">
        {model && <span>{model}</span>}
        {model && <span className="text-zinc-600">·</span>}
        <span className="capitalize">{mode}</span>
      </div>
    </div>
  );
}
