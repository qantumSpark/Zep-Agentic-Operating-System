/* React 19 JSX transform */
import { useWorkflowStore } from "../../stores/workflowStore";
import { PHASE_ORDER } from "../../types/workflow";

export function PipelineSection() {
  const phase = useWorkflowStore((state) => state.phase);
  const pipelineProgress = useWorkflowStore((state) => state.pipelineProgress);

  return (
    <div className="space-y-2">
      {PHASE_ORDER.map((phaseKey) => {
        const isActive = phase === phaseKey;
        const state = pipelineProgress[phaseKey] || "pending";

        const statusIcon =
          state === "done" ? "✅" :
          state === "active" ? "🔄" :
          "⬜";

        return (
          <div
            key={phaseKey}
            className={`flex items-center gap-2 px-2 py-1 rounded transition-colors ${
              isActive ? "bg-blue-900/30 text-blue-300" : "text-zinc-400"
            }`}
          >
            <span className="text-sm">{statusIcon}</span>
            <span className="text-sm capitalize">{phaseKey}</span>
          </div>
        );
      })}
    </div>
  );
}