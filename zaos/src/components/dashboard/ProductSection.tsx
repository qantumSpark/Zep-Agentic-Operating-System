/* React 19 JSX transform */
import { useProductStore } from "../../stores/productStore";
import { StatusBadge } from "../common/StatusBadge";
import type {
  ProductBrief,
  ExperienceGoals,
  AcceptanceChecks,
  ReleaseReadiness,
  SessionInsights,
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
      contract.experience_goals ||
      contract.acceptance_checks ||
      contract.release_readiness ||
      contract.session_insights);

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
      {/* Acceptance Checks */}
      {contract.acceptance_checks && (
        <AcceptanceBlock checks={contract.acceptance_checks} />
      )}
      {/* Release Readiness */}
      {contract.release_readiness && (
        <ReleaseBlock readiness={contract.release_readiness} />
      )}
      {/* Session Insights */}
      {contract.session_insights && (
        <SessionBlock insights={contract.session_insights} />
      )}
    </div>
  );
}

/* ------------------------------------------------------------------ */
/*  Sub-components (internal)                                          */
/* ------------------------------------------------------------------ */

function BriefBlock({ brief }: { brief: ProductBrief }) {
  const hasContent =
    brief.vision ||
    brief.audience ||
    brief.rationale ||
    brief.constraints.length > 0 ||
    brief.out_of_scope.length > 0 ||
    brief.success_definition;
  if (!hasContent) return null;

  return (
    <div>
      <label className="text-zinc-400 text-xs uppercase tracking-wide">
        Brief
      </label>
      <div className="mt-1 space-y-2">
        {brief.vision && (
          <div className="px-2 py-1.5 rounded bg-zinc-800/60">
            <span className="text-zinc-500 text-xs">Vision</span>
            <p className="text-zinc-200 text-xs mt-0.5">{brief.vision}</p>
          </div>
        )}
        {brief.audience && (
          <div className="px-2 py-1.5 rounded bg-zinc-800/60">
            <span className="text-zinc-500 text-xs">Pour qui</span>
            <p className="text-zinc-200 text-xs mt-0.5">{brief.audience}</p>
          </div>
        )}
        {brief.rationale && (
          <div className="px-2 py-1.5 rounded bg-zinc-800/60">
            <span className="text-zinc-500 text-xs">Pourquoi</span>
            <p className="text-zinc-200 text-xs mt-0.5">{brief.rationale}</p>
          </div>
        )}
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
    goals.goals.length > 0 ||
    goals.ux_standards.length > 0 ||
    goals.anti_patterns.length > 0;
  if (!hasContent) return null;

  return (
    <div>
      <label className="text-zinc-400 text-xs uppercase tracking-wide">
        Experience Goals
      </label>
      <div className="mt-1 space-y-2">
        {goals.goals.length > 0 && (
          <div className="rounded border border-zinc-700 overflow-hidden">
            <table className="w-full text-xs">
              <thead>
                <tr className="bg-zinc-800 text-zinc-400">
                  <th className="text-left px-2 py-1 font-medium">#</th>
                  <th className="text-left px-2 py-1 font-medium">Qualite</th>
                  <th className="text-left px-2 py-1 font-medium">Critere</th>
                  <th className="text-right px-2 py-1 font-medium">
                    Priorite
                  </th>
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
        )}
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

function AcceptanceBlock({ checks }: { checks: AcceptanceChecks }) {
  const hasContent =
    checks.checks.length > 0 || checks.manual_validations.length > 0;
  if (!hasContent) return null;

  return (
    <div>
      <label className="text-zinc-400 text-xs uppercase tracking-wide">
        Acceptance Checks
      </label>
      <div className="mt-1 space-y-2">
        {checks.checks.length > 0 && (
          <div className="rounded border border-zinc-700 overflow-hidden">
            <table className="w-full text-xs">
              <thead>
                <tr className="bg-zinc-800 text-zinc-400">
                  <th className="text-left px-2 py-1 font-medium">#</th>
                  <th className="text-left px-2 py-1 font-medium">Check</th>
                  <th className="text-right px-2 py-1 font-medium">Statut</th>
                  <th className="text-left px-2 py-1 font-medium">Notes</th>
                </tr>
              </thead>
              <tbody>
                {checks.checks.map((c) => (
                  <tr
                    key={c.number}
                    className="border-t border-zinc-700/50 hover:bg-zinc-800/40"
                  >
                    <td className="px-2 py-1 text-zinc-500">{c.number}</td>
                    <td className="px-2 py-1 text-zinc-200">{c.check}</td>
                    <td className="px-2 py-1 text-right">
                      <StatusBadge status={c.status} />
                    </td>
                    <td className="px-2 py-1 text-zinc-400 truncate max-w-[120px]">{c.notes}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
        {checks.manual_validations.length > 0 && (
          <div className="px-2 py-1.5 rounded bg-zinc-800/60">
            <span className="text-zinc-500 text-xs">Validations manuelles</span>
            <ul className="mt-0.5">
              {checks.manual_validations.map((v, i) => (
                <li key={i} className="text-zinc-300 text-xs">
                  • {v}
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>
    </div>
  );
}

function ReleaseBlock({ readiness }: { readiness: ReleaseReadiness }) {
  const hasContent =
    readiness.overall_state_summary ||
    readiness.checklist.length > 0 ||
    readiness.open_risks.length > 0;
  if (!hasContent) return null;

  return (
    <div>
      <label className="text-zinc-400 text-xs uppercase tracking-wide">
        Release Readiness
      </label>
      <div className="mt-1 space-y-2">
        {readiness.overall_state_summary && (
          <div className="px-2 py-1.5 rounded bg-zinc-800/60">
            <p className="text-zinc-200 text-xs">
              {readiness.overall_state_summary}
            </p>
          </div>
        )}
        {readiness.checklist.length > 0 && (
          <div className="rounded border border-zinc-700 overflow-hidden">
            <table className="w-full text-xs">
              <thead>
                <tr className="bg-zinc-800 text-zinc-400">
                  <th className="text-left px-2 py-1 font-medium">#</th>
                  <th className="text-left px-2 py-1 font-medium">Item</th>
                  <th className="text-center px-2 py-1 font-medium">
                    Bloquant
                  </th>
                  <th className="text-right px-2 py-1 font-medium">Statut</th>
                  <th className="text-left px-2 py-1 font-medium">Notes</th>
                </tr>
              </thead>
              <tbody>
                {readiness.checklist.map((item) => (
                  <tr
                    key={item.number}
                    className="border-t border-zinc-700/50 hover:bg-zinc-800/40"
                  >
                    <td className="px-2 py-1 text-zinc-500">{item.number}</td>
                    <td className="px-2 py-1 text-zinc-200">{item.item}</td>
                    <td className="px-2 py-1 text-center">
                      {item.blocking === "YES" ? (
                        <span className="text-red-400 text-[10px] font-semibold">
                          YES
                        </span>
                      ) : (
                        <span className="text-zinc-500 text-[10px]">no</span>
                      )}
                    </td>
                    <td className="px-2 py-1 text-right">
                      <StatusBadge status={item.status} />
                    </td>
                    <td className="px-2 py-1 text-zinc-400 truncate max-w-[120px]">{item.notes}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
        {readiness.open_risks.length > 0 && (
          <div className="px-2 py-1.5 rounded bg-zinc-800/60">
            <span className="text-zinc-500 text-xs">Risques ouverts</span>
            <ul className="mt-0.5">
              {readiness.open_risks.map((r, i) => (
                <li key={i} className="text-amber-400/80 text-xs">
                  • {r}
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>
    </div>
  );
}

function SessionBlock({ insights }: { insights: SessionInsights }) {
  const hasContent =
    insights.decisions.length > 0 ||
    insights.learnings.length > 0 ||
    insights.risks.length > 0 ||
    insights.next_validations.length > 0;
  // Also check if metadata is meaningful (not "unknown")
  const hasMeta =
    insights.date !== "unknown" ||
    insights.epic !== "unknown" ||
    insights.phase !== "unknown";
  if (!hasContent && !hasMeta) return null;

  return (
    <div>
      <label className="text-zinc-400 text-xs uppercase tracking-wide">
        Session Insights
      </label>
      <div className="mt-1 space-y-2">
        {hasMeta && (
          <div className="flex gap-3 px-2 py-1 rounded bg-zinc-800/60 text-xs text-zinc-400">
            {insights.date !== "unknown" && (
              <span>
                Date: <span className="text-zinc-300">{insights.date}</span>
              </span>
            )}
            {insights.epic !== "unknown" && (
              <span>
                Epic: <span className="text-zinc-300">{insights.epic}</span>
              </span>
            )}
            {insights.phase !== "unknown" && (
              <span>
                Phase: <span className="text-zinc-300">{insights.phase}</span>
              </span>
            )}
          </div>
        )}
        {insights.decisions.length > 0 && (
          <div className="px-2 py-1.5 rounded bg-zinc-800/60">
            <span className="text-zinc-500 text-xs">Decisions</span>
            <ul className="mt-0.5">
              {insights.decisions.map((d, i) => (
                <li key={i} className="text-zinc-300 text-xs">
                  • {d}
                </li>
              ))}
            </ul>
          </div>
        )}
        {insights.learnings.length > 0 && (
          <div className="px-2 py-1.5 rounded bg-zinc-800/60">
            <span className="text-zinc-500 text-xs">Apprentissages</span>
            <ul className="mt-0.5">
              {insights.learnings.map((l, i) => (
                <li key={i} className="text-zinc-300 text-xs">
                  • {l}
                </li>
              ))}
            </ul>
          </div>
        )}
        {insights.risks.length > 0 && (
          <div className="px-2 py-1.5 rounded bg-zinc-800/60">
            <span className="text-zinc-500 text-xs">Risques</span>
            <ul className="mt-0.5">
              {insights.risks.map((r, i) => (
                <li key={i} className="text-amber-400/80 text-xs">
                  • {r}
                </li>
              ))}
            </ul>
          </div>
        )}
        {insights.next_validations.length > 0 && (
          <div className="px-2 py-1.5 rounded bg-zinc-800/60">
            <span className="text-zinc-500 text-xs">Prochaines validations</span>
            <ul className="mt-0.5">
              {insights.next_validations.map((v, i) => (
                <li key={i} className="text-zinc-300 text-xs">
                  • {v}
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>
    </div>
  );
}
