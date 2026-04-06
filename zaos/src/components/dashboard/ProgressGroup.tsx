import { useMemo } from "react";
import { useMemoryStore } from "../../stores/memoryStore";
import { useActionsStore } from "../../stores/actionsStore";
import { StatusBadge } from "../common/StatusBadge";
import { PhaseIndicator } from "./PhaseIndicator";
import { SessionInsightsBlock } from "./SessionInsightsBlock";
import { getToolIcon } from "../../utils/toolIcons";
import { StatusIndicator } from "../common/StatusIndicator";

export function ProgressGroup() {
  const currentEpic = useMemoryStore((s) => s.currentEpic);
  const milestones = useMemoryStore((s) => s.milestones);
  const actions = useActionsStore((s) => s.actions);

  const recentActions = useMemo(
    () => [...actions].reverse().slice(0, 5),
    [actions],
  );

  return (
    <div className="space-y-4 text-sm">
      {/* Bloc 1 — Phase */}
      <PhaseIndicator />

      {/* Bloc 1.5 — Session Insights */}
      <SessionInsightsBlock />

      {/* Bloc 2 — Tasks de l'epic */}
      {currentEpic && currentEpic.tasks.length > 0 && (() => {
        const done = currentEpic.tasks.filter((t) => {
          const s = t.status.toUpperCase().trim();
          return s === "DONE" || s === "VALIDATED" || s === "TERMINE" || s === "TERMINÉ";
        }).length;
        const total = currentEpic.tasks.length;
        const pct = total > 0 ? Math.round((done / total) * 100) : 0;
        return (
          <div>
            <label className="text-zinc-400 text-xs uppercase tracking-wide">
              {currentEpic.name || "Tasks"}
            </label>
            <div className="mt-2 mb-1">
              <div className="flex items-center justify-between text-xs mb-1">
                <span className="text-zinc-400">{done}/{total} tasks</span>
                <span className="text-zinc-500">{pct}%</span>
              </div>
              <div className="w-full h-1.5 bg-zinc-700 rounded-full overflow-hidden">
                <div className="h-full bg-green-500 rounded-full transition-all duration-300" style={{ width: `${pct}%` }} />
              </div>
            </div>
            <div className="mt-2 rounded border border-zinc-700 overflow-hidden">
              <table className="w-full text-xs">
                <thead>
                  <tr className="bg-zinc-800 text-zinc-400">
                    <th className="text-left px-2 py-1 font-medium">#</th>
                    <th className="text-left px-2 py-1 font-medium">Task</th>
                    <th className="text-right px-2 py-1 font-medium">Status</th>
                  </tr>
                </thead>
                <tbody>
                  {currentEpic.tasks.map((t) => (
                    <tr key={t.number} className="border-t border-zinc-700/50 hover:bg-zinc-800/40">
                      <td className="px-2 py-1 text-zinc-500">{t.number}</td>
                      <td className="px-2 py-1 text-zinc-200 truncate max-w-[160px]">{t.name}</td>
                      <td className="px-2 py-1 text-right"><StatusBadge status={t.status} /></td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        );
      })()}

      {/* Bloc 3 — Milestones */}
      {milestones.length > 0 && (
        <div>
          <label className="text-zinc-400 text-xs uppercase tracking-wide">Milestones</label>
          <ul className="mt-1 space-y-1">
            {milestones.map((m) => (
              <li key={m.number} className="flex items-center justify-between gap-2 px-2 py-1 rounded bg-zinc-800/60">
                <span className="text-zinc-200 text-xs truncate">
                  <span className="text-zinc-500 mr-1">#{m.number}</span>
                  {m.name}
                </span>
                <StatusBadge status={m.status} />
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Bloc 4 — Actions feed condensé */}
      {recentActions.length > 0 && (
        <div>
          <label className="text-zinc-400 text-xs uppercase tracking-wide">Actions récentes</label>
          <div className="mt-1 space-y-1">
            {recentActions.map((action) => (
              <div key={action.id} className={`flex items-start gap-2 p-2 rounded text-xs ${
                action.status === "running" ? "bg-blue-500/10 ring-1 ring-blue-500/30" : "bg-zinc-800/30"
              } ${action.parentId ? "ml-6 border-l-2 border-zinc-700 pl-2" : ""}`}>
                <span className="flex-shrink-0 mt-0.5">{getToolIcon(action.tool)}</span>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="font-medium text-zinc-200 truncate">{action.summary}</span>
                    <StatusIndicator status={action.status} />
                  </div>
                </div>
              </div>
            ))}
            {actions.length > 5 && (
              <p className="text-zinc-500 text-xs text-center mt-1">
                {actions.length - 5} actions supplémentaires dans Advanced
              </p>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
