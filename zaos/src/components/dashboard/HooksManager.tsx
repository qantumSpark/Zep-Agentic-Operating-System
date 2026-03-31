import { invoke } from "@tauri-apps/api/core";
import { useWorkflowKitStore, HookDef } from "../../stores/workflowKitStore";
import { StatusDot } from "../common/StatusDot";
import { Toggle } from "../common/Toggle";

// ---------------------------------------------------------------------------
// Hook card
// ---------------------------------------------------------------------------

function HookCard({ hook }: { hook: HookDef }) {
  return (
    <div className="flex items-start gap-3 px-2 py-2 rounded bg-zinc-800/60 hover:bg-zinc-800/80 transition-colors">
      <StatusDot active={hook.active} />
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-xs font-bold text-zinc-200">{hook.name}</span>
          <span
            className={`inline-block px-1.5 py-0.5 rounded text-[10px] font-semibold uppercase leading-none ${
              hook.active
                ? "bg-green-700/60 text-green-300"
                : "bg-zinc-700/60 text-zinc-400"
            }`}
          >
            {hook.active ? "Active" : "Inactive"}
          </span>
        </div>
        <p className="text-[11px] font-mono text-zinc-500 mt-0.5">
          {hook.event}
        </p>
        <p className="text-xs text-zinc-400 mt-0.5">{hook.description}</p>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// HooksManager
// ---------------------------------------------------------------------------

export function HooksManager() {
  const hooks = useWorkflowKitStore((s) => s.hooks);
  const config = useWorkflowKitStore((s) => s.config);
  const setConfig = useWorkflowKitStore((s) => s.setConfig);

  const handleToggle = async () => {
    const updated = { ...config, hooks_enabled: !config.hooks_enabled };
    setConfig(updated);
    try {
      await invoke("update_workflow_kit_config", { config: updated });
    } catch {
      // Revert on failure
      setConfig(config);
    }
  };

  return (
    <div className="space-y-3 text-sm">
      {/* ---- Header with master toggle ---- */}
      <div className="flex items-center justify-between">
        <label className="text-zinc-400 text-xs uppercase tracking-wide">
          Hooks
        </label>
        <Toggle enabled={config.hooks_enabled} onToggle={handleToggle} />
      </div>

      {/* ---- Hook cards ---- */}
      <div className="space-y-1">
        {hooks.map((hook) => (
          <HookCard key={hook.name} hook={hook} />
        ))}
      </div>

      {hooks.length === 0 && (
        <p className="text-zinc-500 text-xs italic">No hooks defined</p>
      )}
    </div>
  );
}
