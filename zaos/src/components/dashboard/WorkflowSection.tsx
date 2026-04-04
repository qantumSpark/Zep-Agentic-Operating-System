import { useState, useEffect, useRef } from "react";
import { useWorkflowStore } from "../../stores/workflowStore";
import { invoke } from "@tauri-apps/api/core";
import { ProductPhase, PRODUCT_PHASE_ORDER, PRODUCT_PHASE_LABELS, PolicyProfile, POLICY_PROFILE_LABELS, POLICY_PROFILE_DESCRIPTIONS } from "../../types/workflow";

/**
 * Shows epic, phase, mode, task, and gate button
 */
export function WorkflowSection() {
  const phase = useWorkflowStore((state) => state.phase);
  const epic = useWorkflowStore((state) => state.epic);
  const task = useWorkflowStore((state) => state.task);
  const mode = useWorkflowStore((state) => state.mode);
  const setMode = useWorkflowStore((state) => state.setMode);
  const gateReady = useWorkflowStore((state) => state.gateReady);
  const permissionMode = useWorkflowStore((state) => state.permissionMode);
  const setPermissionMode = useWorkflowStore((state) => state.setPermissionMode);
  const policyProfile = useWorkflowStore((state) => state.policyProfile);
  const setPolicyProfile = useWorkflowStore((state) => state.setPolicyProfile);
  const productPhase = useWorkflowStore((state) => state.productPhase);
  const nextProductPhase = useWorkflowStore((state) => state.nextProductPhase);
  const nextTechPhase = useWorkflowStore((state) => state.nextTechPhase);

  const [epicName, setEpicName] = useState("");
  const [epicDesc, setEpicDesc] = useState("");
  const [isStarting, setIsStarting] = useState(false);
  const [isValidating, setIsValidating] = useState(false);
  const [gateMessage, setGateMessage] = useState<string | null>(null);
  const gateTimerRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  useEffect(() => () => clearTimeout(gateTimerRef.current), []);

  const handleStartEpic = async () => {
    if (!epicName.trim() || isStarting) return;
    setIsStarting(true);
    try {
      await invoke("start_epic", { name: epicName.trim(), description: epicDesc.trim() });
      setEpicName("");
      setEpicDesc("");
    } catch (err) {
      console.error("Failed to start epic:", err);
    } finally {
      setIsStarting(false);
    }
  };

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

  const selectMode = async (newMode: "free" | "pipeline") => {
    setMode(newMode);
    try {
      await invoke("set_mode", { mode: newMode });
    } catch (error) {
      console.error("Failed to set workflow mode:", error);
    }
  };

  const togglePermissionMode = async () => {
    const newMode = permissionMode === "accept-edits" ? "strict" : "accept-edits";
    setPermissionMode(newMode);
    try {
      await invoke("set_permission_mode", { mode: newMode });
    } catch (error) {
      console.error("Failed to set permission mode:", error);
    }
  };

  const selectPolicyProfile = async (profile: PolicyProfile) => {
    const previousProfile = policyProfile;
    setPolicyProfile(profile);
    try {
      await invoke("set_policy_profile", { profile: profile });
    } catch (error) {
      console.error("Failed to set policy profile:", error);
      setPolicyProfile(previousProfile);
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
        <div className="space-y-2">
          <label className="text-zinc-400 text-xs uppercase tracking-wide">Nouvel epic</label>
          <input
            type="text"
            value={epicName}
            onChange={(e) => setEpicName(e.target.value)}
            placeholder="Nom de l'epic"
            className="w-full bg-zinc-800 border border-zinc-700 rounded px-2 py-1 text-sm text-zinc-100 placeholder-zinc-500 focus:border-blue-500 outline-none"
            onKeyDown={(e) => e.key === "Enter" && handleStartEpic()}
          />
          <textarea
            value={epicDesc}
            onChange={(e) => setEpicDesc(e.target.value)}
            placeholder="Description (optionnel)"
            rows={2}
            className="w-full bg-zinc-800 border border-zinc-700 rounded px-2 py-1 text-sm text-zinc-100 placeholder-zinc-500 focus:border-blue-500 outline-none resize-none"
          />
          <button
            onClick={handleStartEpic}
            disabled={!epicName.trim() || isStarting}
            className="w-full bg-blue-600 hover:bg-blue-700 text-white rounded px-3 py-1.5 text-sm font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isStarting ? "Demarrage..." : "Demarrer l'epic"}
          </button>
        </div>
      )}

      {/* Product Phase Progress */}
      {productPhase && productPhase !== ProductPhase.None && (
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
      )}
      {/* Fallback: show tech phase only when no product phase */}
      {(!productPhase || productPhase === ProductPhase.None) && phase && (
        <div>
          <label className="text-zinc-400 text-xs uppercase tracking-wide">Phase</label>
          <p className="text-zinc-100 font-medium capitalize">{phase}</p>
        </div>
      )}

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

      {/* Policy Profile */}
      <div>
        <label className="text-zinc-400 text-xs uppercase tracking-wide">Policy</label>
        <div className="mt-1 flex flex-wrap gap-1.5">
          {Object.values(PolicyProfile).map((p) => (
            <button
              key={p}
              onClick={() => selectPolicyProfile(p)}
              className={`px-2.5 py-1 rounded text-xs font-medium transition-colors ${
                policyProfile === p
                  ? "bg-blue-600 text-white"
                  : "bg-zinc-800 text-zinc-300 hover:bg-zinc-700"
              }`}
              title={POLICY_PROFILE_DESCRIPTIONS[p]}
            >
              {POLICY_PROFILE_LABELS[p]}
            </button>
          ))}
        </div>
        <p className="text-zinc-500 text-xs mt-1">{POLICY_PROFILE_DESCRIPTIONS[policyProfile]}</p>
      </div>

      {/* Permissions */}
      <div>
        <label className="text-zinc-400 text-xs uppercase tracking-wide">Permissions</label>
        <div className="mt-1 flex gap-2">
          <button
            onClick={togglePermissionMode}
            className={`px-3 py-1 rounded text-xs font-medium transition-colors ${
              permissionMode === "accept-edits"
                ? "bg-green-600 text-white"
                : "bg-zinc-800 text-zinc-300 hover:bg-zinc-700"
            }`}
          >
            {permissionMode === "accept-edits" ? "Accept Edits" : "Strict"}
          </button>
        </div>
        <p className="text-zinc-500 text-xs mt-1">Auto-approve Write, Edit, WebSearch</p>
      </div>

      {/* Task */}
      {task ? (
        <div>
          <label className="text-zinc-400 text-xs uppercase tracking-wide">Current Task</label>
          <p className="text-zinc-100 text-sm">{task}</p>
        </div>
      ) : null}

      {/* Gate Button */}
      {phase && phase !== "idle" && (
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
                    : "Valider le gate ▶"}
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
      )}
    </div>
  );
}