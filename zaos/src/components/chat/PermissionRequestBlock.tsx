import { useState } from "react";
import { formatToolSummary } from "../../utils/toolFormatters";
import { getToolIcon } from "../../utils/toolIcons";
import { usePermissionStore } from "../../stores/permissionStore";
import type { ApprovalRequestedEvent } from "../../types/zaosEvents";

interface PermissionRequestBlockProps {
  request: ApprovalRequestedEvent;
}

const RISK_COLORS: Record<string, string> = {
  low: "bg-green-700 text-green-100",
  medium: "bg-yellow-700 text-yellow-100",
  high: "bg-orange-700 text-orange-100",
  critical: "bg-red-700 text-red-100",
};

const VERDICT_LABELS: Record<string, string> = {
  allow: "Recommended: Allow",
  ask: "Recommended: Review",
  deny: "Recommended: Deny",
};

export function PermissionRequestBlock({ request }: PermissionRequestBlockProps) {
  const [expanded, setExpanded] = useState(false);
  const [responded, setResponded] = useState(false);
  const [decision, setDecision] = useState<"allow" | "deny" | null>(null);
  const respondToRequest = usePermissionStore((s) => s.respondToRequest);

  const toolName = request.toolName || "Unknown";
  const toolInput = request.toolInput || {};
  const description = request.description || "Permission requested";
  const icon = getToolIcon(toolName);
  const summary = formatToolSummary(toolName, toolInput as Record<string, unknown>);

  // Policy Engine fields (optional — fallback gracieux)
  const riskLevel = request.policyRiskLevel;
  const verdict = request.policyVerdict;
  const reason = request.policyReason;
  const matchedRules = request.policyMatchedRules;
  const hasPolicy = !!(riskLevel || verdict || reason);

  const handleResponse = async (allow: boolean) => {
    setResponded(true);
    setDecision(allow ? "allow" : "deny");
    await respondToRequest(request.requestId, allow);
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
        {riskLevel && (
          <span className={`text-xs font-medium px-1.5 py-0.5 rounded ${RISK_COLORS[riskLevel] || "bg-zinc-600 text-zinc-200"}`}>
            {riskLevel.toUpperCase()}
          </span>
        )}
        <span className="text-zinc-500 text-xs">
          {expanded ? "\u25B2" : "\u25BC"}
        </span>
      </button>

      {/* Policy info section */}
      {hasPolicy && (
        <div className="px-3 py-1.5 border-t border-amber-800/40 space-y-1">
          {verdict && (
            <div className="text-xs font-medium text-zinc-300">
              {VERDICT_LABELS[verdict] || verdict}
            </div>
          )}
          {reason && (
            <div className="text-xs text-zinc-400">
              {reason}
            </div>
          )}
          {matchedRules && matchedRules.length > 0 && (
            <div className="flex flex-wrap gap-1">
              {matchedRules.map((rule) => (
                <span
                  key={rule}
                  className="text-xs bg-zinc-700 text-zinc-300 px-1.5 py-0.5 rounded"
                >
                  {rule}
                </span>
              ))}
            </div>
          )}
        </div>
      )}

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
