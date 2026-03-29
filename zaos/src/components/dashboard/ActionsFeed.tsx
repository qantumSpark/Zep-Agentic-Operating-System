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

const STATUS_ICONS: Record<string, string> = {
  success: "✅",
  error: "❌",
  running: "🔄",
};

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
            className="flex items-start gap-2 p-2 bg-zinc-800/30 rounded hover:bg-zinc-800/50 transition-colors text-xs"
          >
            <span className="flex-shrink-0 mt-0.5">
              {TOOL_ICONS[action.tool] || "🔧"}
            </span>
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2">
                <span className="font-medium text-zinc-200 truncate">
                  {action.summary}
                </span>
                <span>{STATUS_ICONS[action.status] || "⏳"}</span>
              </div>
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