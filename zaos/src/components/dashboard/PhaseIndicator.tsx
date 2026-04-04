import { useWorkflowStore } from "../../stores/workflowStore";
import { ProductPhase, PRODUCT_PHASE_ORDER, PRODUCT_PHASE_LABELS } from "../../types/workflow";

export function PhaseIndicator() {
  const phase = useWorkflowStore((s) => s.phase);
  const productPhase = useWorkflowStore((s) => s.productPhase);

  if (productPhase && productPhase !== ProductPhase.None) {
    return (
      <div>
        <label className="text-zinc-400 text-xs uppercase tracking-wide">Phase</label>
        <div className="mt-1.5 flex items-center gap-1">
          {PRODUCT_PHASE_ORDER.map((pp) => {
            const idx = PRODUCT_PHASE_ORDER.indexOf(pp);
            const currentIdx = PRODUCT_PHASE_ORDER.indexOf(productPhase);
            let status: "done" | "active" | "pending" = "pending";
            if (currentIdx >= 0) {
              if (idx < currentIdx) status = "done";
              else if (idx === currentIdx) status = "active";
            }
            return (
              <div key={pp} className="flex items-center gap-1">
                <div
                  className={`flex items-center justify-center rounded-full text-[9px] font-bold transition-colors ${
                    status === "done"
                      ? "w-5 h-5 bg-green-600/80 text-green-100"
                      : status === "active"
                      ? "w-6 h-6 bg-blue-500 text-white ring-2 ring-blue-400/40"
                      : "w-5 h-5 bg-zinc-700 text-zinc-500"
                  }`}
                  title={PRODUCT_PHASE_LABELS[pp]}
                >
                  {PRODUCT_PHASE_LABELS[pp][0]}
                </div>
                {idx < PRODUCT_PHASE_ORDER.length - 1 && (
                  <div
                    className={`w-2 h-0.5 ${
                      status === "done" ? "bg-green-600/60" : "bg-zinc-700"
                    }`}
                  />
                )}
              </div>
            );
          })}
        </div>
        <p className="text-zinc-100 font-medium mt-1.5">{PRODUCT_PHASE_LABELS[productPhase]}</p>
        {phase && (
          <p className="text-zinc-500 text-xs">{phase}</p>
        )}
      </div>
    );
  }

  if (phase) {
    return (
      <div>
        <label className="text-zinc-400 text-xs uppercase tracking-wide">Phase</label>
        <p className="text-zinc-100 font-medium capitalize">{phase}</p>
      </div>
    );
  }

  return null;
}
