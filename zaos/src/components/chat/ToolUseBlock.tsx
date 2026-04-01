import { useState } from "react";
import { formatToolSummary } from "../../utils/toolFormatters";
import { getToolIcon } from "../../utils/toolIcons";

interface ToolUseProps {
  name: string;
  input: Record<string, unknown>;
  toolUseId: string;
}

interface ToolResultProps {
  content: string;
  isError?: boolean;
}

export function ToolUseBlock({ name, input, toolUseId: _toolUseId }: ToolUseProps) {
  const [expanded, setExpanded] = useState(false);
  const icon = getToolIcon(name);
  const summary = formatToolSummary(name, input);

  return (
    <div className="my-1 rounded-md border border-zinc-700 bg-zinc-800/60 text-sm">
      <button
        onClick={() => setExpanded(!expanded)}
        className="flex items-center gap-2 w-full px-3 py-1.5 text-left hover:bg-zinc-700/30 transition-colors"
      >
        <span>{icon}</span>
        <span className="font-medium text-blue-400">{name}</span>
        {summary && (
          <span className="text-zinc-400 truncate flex-1">{summary}</span>
        )}
        <span className="text-zinc-500 text-xs">
          {expanded ? "\u25B2" : "\u25BC"}
        </span>
      </button>
      {expanded && (
        <pre className="px-3 py-2 text-xs text-zinc-400 border-t border-zinc-700 overflow-x-auto max-h-48">
          {JSON.stringify(input, null, 2)}
        </pre>
      )}
    </div>
  );
}

export function ToolResultBlock({ content, isError }: ToolResultProps) {
  const [expanded, setExpanded] = useState(false);
  const preview = content.slice(0, 120);
  const hasMore = content.length > 120;

  return (
    <div
      className={`my-1 rounded-md border text-sm ${
        isError
          ? "border-red-800/60 bg-red-950/30"
          : "border-green-800/40 bg-green-950/20"
      }`}
    >
      <button
        onClick={() => setExpanded(!expanded)}
        className="flex items-center gap-2 w-full px-3 py-1.5 text-left hover:bg-zinc-700/20 transition-colors"
      >
        <span>{isError ? "\u274C" : "\u2705"}</span>
        <span className={`truncate flex-1 ${isError ? "text-red-400" : "text-green-400"}`}>
          {preview}{hasMore && !expanded ? "..." : ""}
        </span>
        {hasMore && (
          <span className="text-zinc-500 text-xs">
            {expanded ? "\u25B2" : "\u25BC"}
          </span>
        )}
      </button>
      {expanded && hasMore && (
        <pre className="px-3 py-2 text-xs text-zinc-400 border-t border-zinc-700 overflow-x-auto max-h-64 whitespace-pre-wrap">
          {content}
        </pre>
      )}
    </div>
  );
}
