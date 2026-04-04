import { useState, useEffect, useRef } from "react";
import { useWorkflowStore } from "../../stores/workflowStore";
import { invoke } from "@tauri-apps/api/core";
import { ProductPhase, PRODUCT_PHASE_LABELS } from "../../types/workflow";

/**
 * Gate validation button extracted from WorkflowSection.
 * Shows a contextual validate button, tech-phase transition info,
 * and an animated success message.
 * Renders nothing when phase is idle or absent.
 */
export function GateControl() {
  const phase = useWorkflowStore((state) => state.phase);
  const gateReady = useWorkflowStore((state) => state.gateReady);
  const productPhase = useWorkflowStore((state) => state.productPhase);
  const nextProductPhase = useWorkflowStore((state) => state.nextProductPhase);
  const nextTechPhase = useWorkflowStore((state) => state.nextTechPhase);

  const [isValidating, setIsValidating] = useState(false);
  const [gateMessage, setGateMessage] = useState<string | null>(null);
  const gateTimerRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  useEffect(() => () => clearTimeout(gateTimerRef.current), []);

  const handleValidateGate = async () => {
    if (isValidating || !gateReady) return;
    setIsValidating(true);
    try {
      const response = await invoke<{ success: boolean; next_phase: string; message: string }>("validate_gate");
      if (response.success) {
        setGateMessage(`Phase avancée à : ${response.next_phase}`);
        gateTimerRef.current = setTimeout(() => setGateMessage(null), 2000);
      }
    } catch (error) {
      console.error("Failed to validate gate:", error);
    } finally {
      setIsValidating(false);
    }
  };

  if (!phase || phase === "idle") return null;

  return (
    <>
      <button
        onClick={handleValidateGate}
        disabled={!gateReady || isValidating}
        className={`w-full rounded px-3 py-2 text-sm font-medium transition-colors flex flex-col items-center justify-center gap-0.5 mt-2 ${
          gateReady && !isValidating
            ? "bg-green-600 hover:bg-green-700 text-white"
            : "bg-zinc-700 text-zinc-500 cursor-not-allowed"
        }`}
      >
        {isValidating ? (
          "Validation..."
        ) : (
          <>
            <span>
              {nextProductPhase && nextProductPhase !== productPhase
                ? `Valider → ${PRODUCT_PHASE_LABELS[nextProductPhase]}`
                : productPhase && productPhase !== ProductPhase.None
                ? `Valider → continuer ${PRODUCT_PHASE_LABELS[productPhase]}`
                : "Valider le gate \u25B6"}
            </span>
          </>
        )}
      </button>
      {gateReady && !isValidating && phase && nextTechPhase && (
        <p className="text-zinc-500 text-xs text-center mt-0.5">
          {phase} → {nextTechPhase}
        </p>
      )}
      {gateMessage && (
        <p className="text-green-400 text-xs text-center mt-1 animate-pulse">
          {gateMessage}
        </p>
      )}
    </>
  );
}
