import React from "react";
import { useActionsStore } from "../../stores/actionsStore";

const TOOL_ICONS: Record<string, string> = {
  Read: "🔍",
  Write: "📂",
  Edit: "✏️",
  Bash: "🖥️",
  Screenshot: "📸",
  Glob: "🔎",
  Grep: "🔎",
  WebSearch: "🌐",
  WebFetch: "🌐",
  Task: "💬",
};

function StatusIndicator({ status }: { status: string }) {
  if (status === "running") {
    return (
      <span className="inline-block w-3 h-3 rounded-full bg-blue-500 animate-pulse" />
    );
  }
  if (status === "success") return <span>✅</span>;
  if (status === "error") return <span>❌</span>;
  return <span>⏳</span>;
}

export function ActionsFeed() {
  const actions = useActionsStore((state) => state.actions);

  return (
    <div className="space-y-2 max-h-96 overflow-y-auto">
      {actions.length === 0 ? (
        <div className="text-zinc-500 text-sm italic">No actions yet</div>
      ) : (
        [...actions].reverse().map((action) => (
          <div
            key={action.id}
            className={`flex items-start gap-2 p-2 rounded transition-colors text-xs ${
              action.status === "running"
                ? "bg-blue-500/10 ring-1 ring-blue-500/30"
                : "bg-zinc-800/30 hover:bg-zinc-800/50"
            }`}
          >
            <span className="flex-shrink-0 mt-0.5">
              {TOOL_ICONS[action.tool] || "🔧"}
            </span>
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2">
                <span className="font-medium text-zinc-200 truncate">
                  {action.summary}
                </span>
                <StatusIndicator status={action.status} />
              </div>
              {action.resultPreview && (
                <div className="text-zinc-400 mt-0.5 truncate">
                  {action.resultPreview}
                </div>
              )}
              <div className="text-zinc-500 mt-0.5">
                {new Date(action.timestamp).toLocaleTimeString()}
              </div>
            </div>
          </div>
        ))
      )}
    </div>
  );
}