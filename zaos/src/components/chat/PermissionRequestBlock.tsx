import React, { useState } from "react";
import { formatToolSummary } from "../../utils/toolFormatters";
import { usePermissionStore } from "../../stores/permissionStore";
import type { ControlRequest } from "../../types/events";

interface PermissionRequestBlockProps {
  request: ControlRequest;
}

const TOOL_ICONS: Record<string, string> = {
  Read: "\u{1F4C4}",
  Write: "\u{270F}\u{FE0F}",
  Edit: "\u{1F527}",
  Bash: "\u{1F4BB}",
  Glob: "\u{1F50D}",
  Grep: "\u{1F50E}",
  Agent: "\u{1F916}",
  WebSearch: "\u{1F310}",
  WebFetch: "\u{1F310}",
};

function getToolIcon(name: string): string {
  return TOOL_ICONS[name] || "\u{2699}\u{FE0F}";
}

export function PermissionRequestBlock({ request }: PermissionRequestBlockProps) {
  const [expanded, setExpanded] = useState(false);
  const [responded, setResponded] = useState(false);
  const [decision, setDecision] = useState<"allow" | "deny" | null>(null);
  const respondToRequest = usePermissionStore((s) => s.respondToRequest);

  // Tool info lives in request.request (nested object from CLI)
  const nested = request.request;
  const toolName = nested?.tool_name || request.tool || "Unknown";
  const toolInput = nested?.input || request.input || {};
  const description = nested?.description || request.message || "Permission requested";
  const icon = getToolIcon(toolName);
  const summary = formatToolSummary(toolName, toolInput as Record<string, unknown>);

  const handleResponse = async (allow: boolean) => {
    setResponded(true);
    setDecision(allow ? "allow" : "deny");
    await respondToRequest(request.request_id, allow);
  };

  return (
    <div className="my-1 rounded-md border border-amber-700/60 bg-amber-950/20 text-sm">
      {/* Header */}
      <button
        onClick={() => setExpanded(!expanded)}
        className="flex items-center gap-2 w-full px-3 py-1.5 text-left hover:bg-zinc-700/20 transition-colors"
      >
        <span className="text-amber-400">{icon}</span>
        <span className="font-medium text-amber-300">Permission Request</span>
        <span className="text-zinc-400 truncate flex-1">
          {toolName}: {summary}
        </span>
        <span className="text-zinc-500 text-xs">
          {expanded ? "\u25B2" : "\u25BC"}
        </span>
      </button>

      {/* Message */}
      <div className="px-3 py-1 text-zinc-300 text-xs">
        {description}
      </div>

      {/* Expanded details */}
      {expanded && (
        <pre className="px-3 py-2 text-xs text-zinc-400 border-t border-amber-800/40 overflow-x-auto max-h-48">
          {JSON.stringify(toolInput, null, 2)}
        </pre>
      )}

      {/* Action buttons */}
      <div className="flex gap-2 px-3 py-2 border-t border-amber-800/40">
        {responded ? (
          <span
            className={`text-xs font-medium px-3 py-1 rounded ${
              decision === "allow"
                ? "bg-green-900/40 text-green-400"
                : "bg-red-900/40 text-red-400"
            }`}
          >
            {decision === "allow" ? "\u2705 Approved" : "\u274C Denied"}
          </span>
        ) : (
          <>
            <button
              onClick={() => handleResponse(true)}
              className="bg-green-700 hover:bg-green-600 text-white text-xs font-medium px-3 py-1 rounded transition-colors"
            >
              Approve
            </button>
            <button
              onClick={() => handleResponse(false)}
              className="bg-red-700 hover:bg-red-600 text-white text-xs font-medium px-3 py-1 rounded transition-colors"
            >
              Deny
            </button>
          </>
        )}
      </div>
    </div>
  );
}
