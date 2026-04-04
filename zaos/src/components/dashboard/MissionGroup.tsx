import { useProductStore } from "../../stores/productStore";
import { useMemoryStore } from "../../stores/memoryStore";
import { StatusBadge } from "../common/StatusBadge";
import { EpicHeader } from "./EpicHeader";

export function MissionGroup() {
  const productContract = useProductStore((s) => s.productContract);
  const currentEpic = useMemoryStore((s) => s.currentEpic);

  const brief = productContract?.brief ?? null;
  const goals = productContract?.experience_goals ?? null;
  const hasBrief = brief && (brief.vision || brief.audience || brief.rationale);
  const hasGoals = goals && goals.goals.length > 0;
  const hasEpic = !!currentEpic;

  // Empty state
  if (!hasBrief && !hasGoals && !hasEpic) {
    return (
      <div className="text-center py-4">
        <p className="text-zinc-400 text-sm">Aucune mission définie pour ce projet.</p>
        <p className="text-zinc-500 text-xs mt-1">
          Ajoutez un product-brief.md dans .memory/ pour structurer la mission.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-3 text-sm">
      {/* Bloc 1 — Product Brief condensé */}
      {hasBrief && (
        <div>
          <label className="text-zinc-400 text-xs uppercase tracking-wide">Brief</label>
          <div className="mt-1 space-y-1">
            {brief.vision && (
              <div className="px-2 py-1.5 rounded bg-zinc-800/60">
                <span className="text-zinc-500 text-xs">Vision</span>{" "}
                <span className="text-zinc-200 text-xs">{brief.vision}</span>
              </div>
            )}
            {brief.audience && (
              <div className="px-2 py-1.5 rounded bg-zinc-800/60">
                <span className="text-zinc-500 text-xs">Pour qui</span>{" "}
                <span className="text-zinc-200 text-xs">{brief.audience}</span>
              </div>
            )}
            {brief.rationale && (
              <div className="px-2 py-1.5 rounded bg-zinc-800/60">
                <span className="text-zinc-500 text-xs">Pourquoi</span>{" "}
                <span className="text-zinc-200 text-xs">{brief.rationale}</span>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Bloc 2 — Experience Goals condensé */}
      {hasGoals && (
        <div>
          <label className="text-zinc-400 text-xs uppercase tracking-wide">
            Experience Goals
          </label>
          <div className="mt-1 rounded border border-zinc-700 overflow-hidden">
            <table className="w-full text-xs">
              <thead>
                <tr className="bg-zinc-800 text-zinc-400">
                  <th className="text-left px-2 py-1 font-medium">#</th>
                  <th className="text-left px-2 py-1 font-medium">Qualite</th>
                  <th className="text-left px-2 py-1 font-medium">Critere</th>
                  <th className="text-right px-2 py-1 font-medium">Priorite</th>
                </tr>
              </thead>
              <tbody>
                {goals.goals.map((g) => (
                  <tr
                    key={g.number}
                    className="border-t border-zinc-700/50 hover:bg-zinc-800/40"
                  >
                    <td className="px-2 py-1 text-zinc-500">{g.number}</td>
                    <td className="px-2 py-1 text-zinc-200">{g.quality}</td>
                    <td className="px-2 py-1 text-zinc-300">{g.criterion}</td>
                    <td className="px-2 py-1 text-right">
                      <StatusBadge status={g.priority} />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Bloc 3 — Epic active */}
      <EpicHeader />
    </div>
  );
}
