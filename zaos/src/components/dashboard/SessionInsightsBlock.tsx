import { useProductStore } from "../../stores/productStore";
import { useWorkflowStore } from "../../stores/workflowStore";
import { getSuggestedPersonaForPhase } from "../../utils/personaMapping";

function formatRelativeDate(dateStr: string): string {
  const today = new Date();
  const parts = dateStr.split("-");
  if (parts.length !== 3) return dateStr;
  const date = new Date(Number(parts[0]), Number(parts[1]) - 1, Number(parts[2]));
  const todayStart = new Date(today.getFullYear(), today.getMonth(), today.getDate());
  const diffMs = todayStart.getTime() - date.getTime();
  const diffDays = Math.round(diffMs / (1000 * 60 * 60 * 24));
  if (diffDays === 0) return "aujourd'hui";
  if (diffDays === 1) return "hier";
  if (diffDays > 1) return `il y a ${diffDays}j`;
  return dateStr;
}

export function SessionInsightsBlock() {
  const insights = useProductStore((s) => s.productContract?.session_insights ?? null);
  const currentPhase = useWorkflowStore((s) => s.phase);

  if (!insights) return null;

  const hasDecisions = insights.decisions.length > 0 && !insights.decisions.every((d) => d.startsWith("_"));
  const hasRisks = insights.risks.length > 0 && !insights.risks.every((r) => r.startsWith("_"));
  const hasValidations = insights.next_validations.length > 0 && !insights.next_validations.every((v) => v.startsWith("_"));
  const hasLearnings = insights.learnings.length > 0 && !insights.learnings.every((l) => l.startsWith("_"));
  const hasContext = insights.date !== "unknown" || insights.phase !== "unknown";

  // Persona: prioritize persisted value, fallback to local calculation
  const suggestedPersona =
    (insights.suggested_next_persona && insights.suggested_next_persona !== "unknown" && insights.suggested_next_persona !== "")
      ? insights.suggested_next_persona
      : (currentPhase ? getSuggestedPersonaForPhase(currentPhase) : null);

  // If nothing meaningful, don't render
  if (!hasDecisions && !hasRisks && !hasValidations && !hasLearnings && !hasContext) return null;

  return (
    <div>
      <label className="text-zinc-400 text-xs uppercase tracking-wide">Session Insights</label>
      <div className="mt-1 space-y-1.5">
        {/* Risks — attention/danger first */}
        {hasRisks && (
          <div className="px-2 py-1.5 rounded bg-amber-900/20 border border-amber-800/30">
            <span className="text-amber-400 text-[11px] font-medium">{"\u26A0\uFE0F"} Risques</span>
            <ul className="mt-0.5">
              {insights.risks.filter((r) => !r.startsWith("_")).map((r, i) => (
                <li key={i} className="text-amber-300/80 text-xs">{"\u2022"} {r}</li>
              ))}
            </ul>
          </div>
        )}

        {/* Next validations — actionable */}
        {hasValidations && (
          <div className="px-2 py-1.5 rounded bg-green-900/20 border border-green-800/30">
            <span className="text-green-400 text-[11px] font-medium">{"\u2705"} Prochaines validations</span>
            <ul className="mt-0.5">
              {insights.next_validations.filter((v) => !v.startsWith("_")).map((v, i) => (
                <li key={i} className="text-green-300/80 text-xs">{"\u2022"} {v}</li>
              ))}
            </ul>
          </div>
        )}

        {/* Decisions */}
        {hasDecisions && (
          <div className="px-2 py-1.5 rounded bg-blue-900/30 border border-blue-800/30">
            <span className="text-blue-400 text-[11px] font-medium">{"\u2696\uFE0F"} Decisions</span>
            <ul className="mt-0.5">
              {insights.decisions.filter((d) => !d.startsWith("_")).map((d, i) => (
                <li key={i} className="text-zinc-300 text-xs">{"\u2022"} {d}</li>
              ))}
            </ul>
          </div>
        )}

        {/* Learnings */}
        {hasLearnings && (
          <div className="px-2 py-1.5 rounded bg-purple-900/20 border border-purple-800/30">
            <span className="text-purple-400 text-[11px] font-medium">{"\uD83D\uDCA1"} Apprentissages</span>
            <ul className="mt-0.5">
              {insights.learnings.filter((l) => !l.startsWith("_")).map((l, i) => (
                <li key={i} className="text-purple-300/80 text-xs">{"\u2022"} {l}</li>
              ))}
            </ul>
          </div>
        )}

        {/* Context (discret) */}
        {hasContext && (
          <div className="flex flex-wrap gap-x-3 gap-y-0.5 text-zinc-500 text-[11px] px-1">
            {insights.date !== "unknown" && (
              <span title={insights.date}>{formatRelativeDate(insights.date)}</span>
            )}
            {insights.phase !== "unknown" && <span>{insights.phase}</span>}
            {insights.epic !== "unknown" && <span>{insights.epic}</span>}
            {insights.duration_secs > 0 && <span>{Math.round(insights.duration_secs / 60)}min</span>}
            {insights.agents_used.length > 0 && (
              <span>{insights.agents_used.join(", ")}</span>
            )}
          </div>
        )}

        {/* Suggested persona — visible badge */}
        {suggestedPersona && (
          <div className="px-1">
            <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-indigo-900/30 border border-indigo-700/40 text-[11px] font-medium text-indigo-300">
              {"\uD83C\uDFAD"} {suggestedPersona}
            </span>
          </div>
        )}
      </div>
    </div>
  );
}
