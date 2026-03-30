import React, { useEffect, useState } from "react";
import { useSessionStore } from "../../stores/sessionStore";
import { useWorkflowStore } from "../../stores/workflowStore";

/**
 * Bottom status bar: tokens, duration, connections, mode
 */
export function StatusBar() {
  const tokens = useSessionStore((state) => state.tokens);
  const duration = useSessionStore((state) => state.duration);
  const connections = useSessionStore((state) => state.connections);
  const cliVersion = useSessionStore((state) => state.cliVersion);
  const cliAuthMessage = useSessionStore((state) => state.cliAuthMessage);
  const model = useSessionStore((state) => state.model);
  const startTime = useSessionStore((state) => state.startTime);
  const mode = useWorkflowStore((state) => state.mode);
  const [displayDuration, setDisplayDuration] = useState(duration);

  useEffect(() => {
    setDisplayDuration(duration);
    if (!startTime) return;

    const interval = setInterval(() => {
      setDisplayDuration((d) => d + 1);
    }, 1000);
    return () => clearInterval(interval);
  }, [duration, startTime]);

  const formatDuration = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    if (hours > 0) return `${hours}h ${minutes}m ${secs}s`;
    if (minutes > 0) return `${minutes}m ${secs}s`;
    return `${secs}s`;
  };

  const totalTokens = tokens.input + tokens.output;
  const maxTokens = 200000;
  const tokenPercent = Math.round((totalTokens / maxTokens) * 100);

  return (
    <div className="flex items-center justify-between px-4 py-2 bg-zinc-800 border-t border-zinc-700 text-xs text-zinc-300 gap-4">
      {/* Token usage */}
      <div className="flex items-center gap-2 min-w-0">
        <span className="text-yellow-500">⚡</span>
        <div className="flex gap-1">
          <span className="font-medium">{tokenPercent}%</span>
          <span className="text-zinc-500">
            {totalTokens.toLocaleString()}/{maxTokens.toLocaleString()}
          </span>
          {tokens.cache > 0 && (
            <span className="text-blue-400">
              (cache: {tokens.cache.toLocaleString()})
            </span>
          )}
        </div>
      </div>
      <div className="w-px h-4 bg-zinc-600" />
      {/* Duration */}
      <div className="flex items-center gap-2 min-w-0">
        <span className="text-blue-400">⏱</span>
        <span>{formatDuration(displayDuration)}</span>
      </div>
      <div className="w-px h-4 bg-zinc-600" />
      {/* Connections */}
      <div className="flex items-center gap-3 min-w-0">
        <span>🔗</span>
        <div className="flex gap-1.5">
          <div title={connections.cli ? "Claude CLI " + cliVersion : cliAuthMessage || "CLI not connected"} className={`w-2 h-2 rounded-full ${connections.cli ? "bg-green-500" : "bg-red-500"}`} />
          <div title="GoPeak" className={`w-2 h-2 rounded-full ${connections.gopeak ? "bg-green-500" : "bg-red-500"}`} />
          <div title="Godot" className={`w-2 h-2 rounded-full ${connections.godot ? "bg-green-500" : "bg-red-500"}`} />
        </div>
      </div>
      <div className="w-px h-4 bg-zinc-600" />
      {/* Model + mode */}
      <div className="flex items-center gap-2 text-zinc-400">
        {model && <span>{model}</span>}
        {model && <span className="text-zinc-600">·</span>}
        <span className="capitalize">{mode}</span>
      </div>
    </div>
  );
}