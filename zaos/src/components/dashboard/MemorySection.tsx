import React from "react";
import { useMemoryStore } from "../../stores/memoryStore";

// ---------------------------------------------------------------------------
// Status badge helper
// ---------------------------------------------------------------------------

interface StatusBadgeProps {
  status: string;
}

function statusColor(status: string): string {
  const s = status.toUpperCase().trim();
  if (s === "TERMINE" || s === "TERMINÉ" || s === "VALIDATED" || s === "DONE")
    return "bg-green-700/60 text-green-300";
  if (s === "EN COURS" || s === "EN_COURS" || s === "IN_PROGRESS")
    return "bg-blue-700/60 text-blue-300";
  if (s.startsWith("BLOCK") || s.startsWith("BLOQU"))
    return "bg-red-700/60 text-red-300";
  // default: not started / A FAIRE / pending
  return "bg-zinc-600/60 text-zinc-300";
}

function StatusBadge({ status }: StatusBadgeProps) {
  return (
    <span
      className={`inline-block px-1.5 py-0.5 rounded text-[10px] font-semibold uppercase leading-none ${statusColor(status)}`}
    >
      {status}
    </span>
  );
}

// ---------------------------------------------------------------------------
// MemorySection
// ---------------------------------------------------------------------------

export function MemorySection() {
  const loaded = useMemoryStore((s) => s.loaded);
  const milestones = useMemoryStore((s) => s.milestones);
  const activeEpic = useMemoryStore((s) => s.activeEpic);
  const blocages = useMemoryStore((s) => s.blocages);
  const currentEpic = useMemoryStore((s) => s.currentEpic);

  if (!loaded) {
    return <p className="text-zinc-500 text-xs italic">Loading...</p>;
  }

  return (
    <div className="space-y-4 text-sm">
      {/* ---- Milestones ---- */}
      {milestones.length > 0 && (
        <div>
          <label className="text-zinc-400 text-xs uppercase tracking-wide">
            Milestones
          </label>
          <ul className="mt-1 space-y-1">
            {milestones.map((m) => (
              <li
                key={m.number}
                className="flex items-center justify-between gap-2 px-2 py-1 rounded bg-zinc-800/60"
              >
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

      {/* ---- Active Epic ---- */}
      {activeEpic && (
        <div>
          <label className="text-zinc-400 text-xs uppercase tracking-wide">
            Active Epic
          </label>
          <p className="text-zinc-100 font-medium mt-0.5">{activeEpic}</p>
        </div>
      )}

      {/* ---- Current Epic + Tasks ---- */}
      {currentEpic && (
        <div>
          <label className="text-zinc-400 text-xs uppercase tracking-wide">
            {currentEpic.name}
          </label>
          {currentEpic.objective && (
            <p className="text-zinc-400 text-xs mt-0.5">
              {currentEpic.objective}
            </p>
          )}

          {currentEpic.tasks.length > 0 && (
            <div className="mt-2 rounded border border-zinc-700 overflow-hidden">
              <table className="w-full text-xs">
                <thead>
                  <tr className="bg-zinc-800 text-zinc-400">
                    <th className="text-left px-2 py-1 font-medium">#</th>
                    <th className="text-left px-2 py-1 font-medium">Task</th>
                    <th className="text-right px-2 py-1 font-medium">
                      Status
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {currentEpic.tasks.map((t) => (
                    <tr
                      key={t.number}
                      className="border-t border-zinc-700/50 hover:bg-zinc-800/40"
                    >
                      <td className="px-2 py-1 text-zinc-500">{t.number}</td>
                      <td className="px-2 py-1 text-zinc-200 truncate max-w-[160px]">
                        {t.name}
                      </td>
                      <td className="px-2 py-1 text-right">
                        <StatusBadge status={t.status} />
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}

      {/* ---- Blocages ---- */}
      {blocages && (
        <div>
          <label className="text-zinc-400 text-xs uppercase tracking-wide">
            Blocages
          </label>
          <p className="text-red-400 text-xs mt-0.5">{blocages}</p>
        </div>
      )}

      {/* Nothing loaded */}
      {milestones.length === 0 && !activeEpic && !currentEpic && !blocages && (
        <p className="text-zinc-500 text-xs italic">No memory data available</p>
      )}
    </div>
  );
}
