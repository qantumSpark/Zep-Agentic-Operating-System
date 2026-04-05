import { useProductStore } from "../../stores/productStore";
import { useWorkflowStore } from "../../stores/workflowStore";
import { getSuggestedPersonaForPhase } from "../../utils/personaMapping";

export function SessionInsightsBlock() {
  const insights = useProductStore((s) => s.productContract?.session_insights ?? null);
  const currentPhase = useWorkflowStore((s) => s.phase);

  if (!insights) return null;

  const hasDecisions = insights.decisions.length > 0 && !insights.decisions.every((d) => d.startsWith("_"));
  const hasRisks = insights.risks.length > 0 && !insights.risks.every((r) => r.startsWith("_"));
  const hasValidations = insights.next_validations.length > 0 && !insights.next_validations.every((v) => v.startsWith("_"));
  const hasContext = insights.date !== "unknown" || insights.phase !== "unknown";

  const suggestedPersona = currentPhase ? getSuggestedPersonaForPhase(currentPhase) : null;

  // If nothing meaningful, don't render
  if (!hasDecisions && !hasRisks && !hasValidations && !hasContext) return null;

  return (
    <div>
      <label className="text-zinc-400 text-xs uppercase tracking-wide">Session Insights</label>
      <div className="mt-1 space-y-1.5">
        {/* Decisions */}
        {hasDecisions && (
          <div className="px-2 py-1.5 rounded bg-blue-900/30 border border-blue-800/30">
            <span className="text-blue-400 text-[11px] font-medium">Decisions</span>
            <ul className="mt-0.5">
              {insights.decisions.filter((d) => !d.startsWith("_")).map((d, i) => (
                <li key={i} className="text-zinc-300 text-xs">• {d}</li>
              ))}
            </ul>
          </div>
        )}

        {/* Risks */}
        {hasRisks && (
          <div className="px-2 py-1.5 rounded bg-amber-900/20 border border-amber-800/30">
            <span className="text-amber-400 text-[11px] font-medium">Risques</span>
            <ul className="mt-0.5">
              {insights.risks.filter((r) => !r.startsWith("_")).map((r, i) => (
                <li key={i} className="text-amber-300/80 text-xs">• {r}</li>
              ))}
            </ul>
          </div>
        )}

        {/* Next validations */}
        {hasValidations && (
          <div className="px-2 py-1.5 rounded bg-green-900/20 border border-green-800/30">
            <span className="text-green-400 text-[11px] font-medium">Prochaines validations</span>
            <ul className="mt-0.5">
              {insights.next_validations.filter((v) => !v.startsWith("_")).map((v, i) => (
                <li key={i} className="text-green-300/80 text-xs">• {v}</li>
              ))}
            </ul>
          </div>
        )}

        {/* Context (discret) */}
        {hasContext && (
          <div className="flex flex-wrap gap-x-3 gap-y-0.5 text-zinc-500 text-[11px] px-1">
            {insights.date !== "unknown" && <span>{insights.date}</span>}
            {insights.phase !== "unknown" && <span>{insights.phase}</span>}
            {insights.epic !== "unknown" && <span>{insights.epic}</span>}
            {insights.duration_secs > 0 && <span>{Math.round(insights.duration_secs / 60)}min</span>}
            {insights.agents_used.length > 0 && (
              <span>{insights.agents_used.join(", ")}</span>
            )}
          </div>
        )}

        {/* Suggested persona */}
        {suggestedPersona && (
          <p className="text-zinc-500 text-[11px] px-1 italic">
            Prochain role pertinent : {suggestedPersona}
          </p>
        )}
      </div>
    </div>
  );
}
