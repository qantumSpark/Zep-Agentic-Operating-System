import React from "react";
import { useWorkflowStore } from "../../stores/workflowStore";
import { PHASE_ORDER } from "../../types/workflow";
import { Check, Circle } from "lucide-react";

/**
 * Vertical pipeline visualization: phases with state indicators
 */
export function PipelineSection() {
  const phase = useWorkflowStore((state) => state.phase);
  const pipelineProgress = useWorkflowStore((state) => state.pipelineProgress);

  return (
    <div className="space-y-2">
      {PHASE_ORDER.map((phaseKey) => {
        const isActive = phase === phaseKey;
        const state = pipelineProgress[phaseKey] || "pending";

        const statusIcon =
          state === "done" ? (
            <Check size={16} className="text-green-400" />
          ) : state === "active" ? (
            <div className="w-4 h-4 border-2 border-blue-400 border-t-transparent rounded-full animate-spin" />
          ) : (
            <Circle size={16} className="text-zinc-500" />
          );

        return (
          <div
            key={phaseKey}
            className={`flex items-center gap-2 px-2 py-1 rounded transition-colors ${
              isActive ? "bg-blue-900/30 text-blue-300" : "text-zinc-400"
            }`}
          >
            {statusIcon}
            <span className="text-sm capitalize">{phaseKey}</span>
          </div>
        );
      })}
    </div>
  );
}
