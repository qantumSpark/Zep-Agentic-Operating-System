import { useState } from "react";
import { useWorkflowStore } from "../../stores/workflowStore";
import { invoke } from "@tauri-apps/api/core";
import { PolicyProfile, POLICY_PROFILE_LABELS, POLICY_PROFILE_DESCRIPTIONS } from "../../types/workflow";
import { PhaseIndicator } from "./PhaseIndicator";
import { GateControl } from "./GateControl";

/**
 * Shows epic, phase, mode, and gate button
 */
export function WorkflowSection() {
  const epic = useWorkflowStore((state) => state.epic);
  const epicObjective = useWorkflowStore((state) => state.epicObjective);
  const mode = useWorkflowStore((state) => state.mode);
  const setMode = useWorkflowStore((state) => state.setMode);
  const permissionMode = useWorkflowStore((state) => state.permissionMode);
  const setPermissionMode = useWorkflowStore((state) => state.setPermissionMode);
  const policyProfile = useWorkflowStore((state) => state.policyProfile);
  const setPolicyProfile = useWorkflowStore((state) => state.setPolicyProfile);

  const [epicName, setEpicName] = useState("");
  const [epicDesc, setEpicDesc] = useState("");
  const [isStarting, setIsStarting] = useState(false);

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
          <p className="text-zinc-100 font-medium">{epic}</p>
          {epicObjective && (
            <p className="text-zinc-400 text-xs mt-1">{epicObjective}</p>
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

      {/* Phase indicator (product phases + fallback tech phase) */}
      <PhaseIndicator />

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

      {/* Gate Button */}
      <GateControl />
    </div>
  );
}