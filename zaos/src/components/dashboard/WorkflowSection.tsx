import React from "react";
import { useWorkflowStore } from "../../stores/workflowStore";
import { invoke } from "@tauri-apps/api/core";

/**
 * Shows epic, phase, mode, task, and gate button
 */
export function WorkflowSection() {
  const phase = useWorkflowStore((state) => state.phase);
  const epic = useWorkflowStore((state) => state.epic);
  const task = useWorkflowStore((state) => state.task);
  const mode = useWorkflowStore((state) => state.mode);
  const setMode = useWorkflowStore((state) => state.setMode);

  const handleValidateGate = async () => {
    try {
      await invoke("validate_gate");
    } catch (error) {
      console.error("Failed to validate gate:", error);
    }
  };

  const selectMode = async (newMode: "free" | "pipeline") => {
    setMode(newMode);
    try {
      await invoke("set_mode", { mode: newMode });
    } catch (error) {
      console.error("Failed to set workflow mode:", error);
    }
  };

  return (
    <div className="space-y-3 text-sm">
      {/* Epic */}
      {epic ? (
        <div>
          <label className="text-zinc-400 text-xs uppercase tracking-wide">Epic</label>
          <p className="text-zinc-100 font-medium">{epic.name}</p>
          {epic.description && (
            <p className="text-zinc-400 text-xs mt-1">{epic.description}</p>
          )}
        </div>
      ) : (
        <div className="text-zinc-500 italic">No epic in progress</div>
      )}

      {/* Phase */}
      {phase ? (
        <div>
          <label className="text-zinc-400 text-xs uppercase tracking-wide">Phase</label>
          <p className="text-zinc-100 font-medium capitalize">{phase}</p>
        </div>
      ) : null}

      {/* Mode */}
      <div>
        <label className="text-zinc-400 text-xs uppercase tracking-wide">Mode</label>
        <div className="mt-1 flex gap-2">
          <button
            onClick={() => selectMode("free")}
            className={`px-3 py-1 rounded text-xs font-medium transition-colors ${
              mode === "free"
                ? "bg-blue-600 text-white"
                : "bg-zinc-800 text-zinc-300 hover:bg-zinc-700"
            }`}
          >
            Free
          </button>
          <button
            onClick={() => selectMode("pipeline")}
            className={`px-3 py-1 rounded text-xs font-medium transition-colors ${
              mode === "pipeline"
                ? "bg-blue-600 text-white"
                : "bg-zinc-800 text-zinc-300 hover:bg-zinc-700"
            }`}
          >
            Pipeline
          </button>
        </div>
      </div>

      {/* Task */}
      {task ? (
        <div>
          <label className="text-zinc-400 text-xs uppercase tracking-wide">Current Task</label>
          <p className="text-zinc-100 text-sm">{task}</p>
        </div>
      ) : null}

      {/* Gate Button */}
      {phase && (
        <button
          onClick={handleValidateGate}
          className="w-full bg-green-600 hover:bg-green-700 text-white rounded px-3 py-2 text-sm font-medium transition-colors flex items-center justify-center gap-2 mt-2"
        >
          Validate Gate ▶
        </button>
      )}
    </div>
  );
}