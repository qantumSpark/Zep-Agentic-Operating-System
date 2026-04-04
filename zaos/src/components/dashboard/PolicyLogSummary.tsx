import { useMemo } from "react";
import { usePermissionStore } from "../../stores/permissionStore";

const RISK_COLORS: Record<string, string> = {
  low: "bg-green-700/60 text-green-200",
  medium: "bg-yellow-700/60 text-yellow-200",
  high: "bg-orange-700/60 text-orange-200",
  critical: "bg-red-700/60 text-red-200",
};

function formatTime(ts: number): string {
  const d = new Date(ts);
  return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

export function PolicyLogSummary() {
  const policyLog = usePermissionStore((s) => s.policyLog);

  const recentDenies = useMemo(() => {
    return policyLog
      .filter((e) => e.wasAutoDenied)
      .sort((a, b) => b.timestamp - a.timestamp)
      .slice(0, 5);
  }, [policyLog]);

  if (recentDenies.length === 0) return null;

  return (
    <div>
      <label className="text-zinc-500 text-xs uppercase tracking-wide">
        Auto-deny recents
      </label>
      <ul className="mt-1 space-y-1">
        {recentDenies.map((entry, i) => (
          <li
            key={`${entry.timestamp}-${i}`}
            className="flex items-center justify-between bg-zinc-800/30 rounded px-2 py-1 text-xs"
          >
            <span className="text-zinc-400 truncate">
              {entry.toolName}
              <span className="text-zinc-600 mx-1">&mdash;</span>
              {entry.reason}
            </span>
            <span className="flex items-center gap-1.5 shrink-0 ml-2">
              <span
                className={`px-1.5 py-0.5 rounded text-[10px] font-medium ${RISK_COLORS[entry.riskLevel] || "bg-zinc-600 text-zinc-200"}`}
              >
                {(entry.riskLevel || "unknown").toUpperCase()}
              </span>
              <span className="text-zinc-600">{formatTime(entry.timestamp)}</span>
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}
