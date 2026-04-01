/* React 19 JSX transform */
import { useMemoryStore } from "../../stores/memoryStore";
import { StatusBadge } from "../common/StatusBadge";

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
          <p className="text-zinc-100 font-medium mt-0.5">
            {activeEpic.replace(/\s*\(\d+\/\d+\s*t[aâ]ches?\)/i, "")}
          </p>
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

          {currentEpic.tasks.length === 0 && (
            <div className="mt-2 px-2 py-1.5 rounded bg-zinc-800/60 border border-zinc-700/50">
              <p className="text-zinc-500 text-xs italic">Plan en attente...</p>
            </div>
          )}

          {currentEpic.tasks.length > 0 && (() => {
            const done = currentEpic.tasks.filter((t) => {
              const s = t.status.toUpperCase().trim();
              return s === "DONE" || s === "VALIDATED" || s === "TERMINE" || s === "TERMINÉ";
            }).length;
            const total = currentEpic.tasks.length;
            const pct = total > 0 ? Math.round((done / total) * 100) : 0;
            return (
              <div className="mt-2 mb-1">
                <div className="flex items-center justify-between text-xs mb-1">
                  <span className="text-zinc-400">{done}/{total} tasks</span>
                  <span className="text-zinc-500">{pct}%</span>
                </div>
                <div className="w-full h-1.5 bg-zinc-700 rounded-full overflow-hidden">
                  <div
                    className="h-full bg-green-500 rounded-full transition-all duration-300"
                    style={{ width: `${pct}%` }}
                  />
                </div>
              </div>
            );
          })()}

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
