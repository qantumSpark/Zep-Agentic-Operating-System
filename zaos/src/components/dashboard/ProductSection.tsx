/* React 19 JSX transform */
import { useProductStore } from "../../stores/productStore";
import type {
  ProductBrief,
  ExperienceGoals,
} from "../../types/productContract";

export function ProductSection() {
  const contract = useProductStore((s) => s.productContract);
  const loaded = useProductStore((s) => s.loaded);

  if (!loaded) {
    return <p className="text-zinc-500 text-xs italic">Loading...</p>;
  }

  // Check if any artifact has content
  const hasAny =
    contract &&
    (contract.brief ||
      contract.experience_goals);

  if (!hasAny) {
    return (
      <p className="text-zinc-500 text-xs italic">
        No product contract data yet
      </p>
    );
  }

  return (
    <div className="space-y-4 text-sm">
      {/* Brief */}
      {contract.brief && <BriefBlock brief={contract.brief} />}
      {/* Experience Goals */}
      {contract.experience_goals && (
        <ExperienceBlock goals={contract.experience_goals} />
      )}
    </div>
  );
}

/* ------------------------------------------------------------------ */
/*  Sub-components (internal)                                          */
/* ------------------------------------------------------------------ */

function BriefBlock({ brief }: { brief: ProductBrief }) {
  const hasContent =
    brief.success_definition ||
    brief.constraints.length > 0 ||
    brief.out_of_scope.length > 0;
  if (!hasContent) return null;

  return (
    <div>
      <label className="text-zinc-400 text-xs uppercase tracking-wide">
        Brief — Détails complémentaires
      </label>
      <div className="mt-1 space-y-2">
        {brief.success_definition && (
          <div className="px-2 py-1.5 rounded bg-zinc-800/60">
            <span className="text-zinc-500 text-xs">Definition de succes</span>
            <p className="text-zinc-200 text-xs mt-0.5">
              {brief.success_definition}
            </p>
          </div>
        )}
        {brief.constraints.length > 0 && (
          <div className="px-2 py-1.5 rounded bg-zinc-800/60">
            <span className="text-zinc-500 text-xs">Contraintes</span>
            <ul className="mt-0.5">
              {brief.constraints.map((c, i) => (
                <li key={i} className="text-zinc-300 text-xs">
                  • {c}
                </li>
              ))}
            </ul>
          </div>
        )}
        {brief.out_of_scope.length > 0 && (
          <div className="px-2 py-1.5 rounded bg-zinc-800/60">
            <span className="text-zinc-500 text-xs">Hors-scope</span>
            <ul className="mt-0.5">
              {brief.out_of_scope.map((c, i) => (
                <li key={i} className="text-zinc-300 text-xs">
                  • {c}
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>
    </div>
  );
}

function ExperienceBlock({ goals }: { goals: ExperienceGoals }) {
  const hasContent =
    goals.ux_standards.length > 0 ||
    goals.anti_patterns.length > 0;
  if (!hasContent) return null;

  return (
    <div>
      <label className="text-zinc-400 text-xs uppercase tracking-wide">
        UX — Standards & Anti-patterns
      </label>
      <div className="mt-1 space-y-2">
        {goals.ux_standards.length > 0 && (
          <div className="px-2 py-1.5 rounded bg-zinc-800/60">
            <span className="text-zinc-500 text-xs">Standards UX</span>
            <ul className="mt-0.5">
              {goals.ux_standards.map((s, i) => (
                <li key={i} className="text-zinc-300 text-xs">
                  • {s}
                </li>
              ))}
            </ul>
          </div>
        )}
        {goals.anti_patterns.length > 0 && (
          <div className="px-2 py-1.5 rounded bg-zinc-800/60">
            <span className="text-zinc-500 text-xs">Anti-patterns</span>
            <ul className="mt-0.5">
              {goals.anti_patterns.map((a, i) => (
                <li key={i} className="text-red-400/80 text-xs">
                  • {a}
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>
    </div>
  );
}
