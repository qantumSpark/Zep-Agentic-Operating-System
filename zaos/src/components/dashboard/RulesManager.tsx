import { invoke } from "@tauri-apps/api/core";
import { useWorkflowKitStore, RuleDef } from "../../stores/workflowKitStore";
import { StatusDot } from "../common/StatusDot";
import { Toggle } from "../common/Toggle";

const DEFAULT_RULES: RuleDef[] = [
  {
    name: "01-always.mdc",
    deployed: true,
    enabled: true,
    description: "Universal rules (always read memory, work incrementally)",
  },
  {
    name: "02-anti-hallucination.mdc",
    deployed: true,
    enabled: true,
    description: "Never invent APIs, verify imports",
  },
  {
    name: "03-code-review.mdc",
    deployed: true,
    enabled: true,
    description: "Post-modification review checklist",
  },
  {
    name: "04-memory-hygiene.mdc",
    deployed: true,
    enabled: true,
    description: "Memory file size limits, anti-duplication",
  },
];

// ---------------------------------------------------------------------------
// Rule card
// ---------------------------------------------------------------------------

function RuleCard({
  rule,
  onToggle,
}: {
  rule: RuleDef;
  onToggle: () => void;
}) {
  return (
    <div className="flex items-center gap-3 px-2 py-2 rounded bg-zinc-800/60 hover:bg-zinc-800/80 transition-colors">
      <StatusDot active={rule.enabled} />
      <div className="flex-1 min-w-0">
        <span className="text-xs font-bold text-zinc-200">{rule.name}</span>
        <p className="text-xs text-zinc-400 mt-0.5">{rule.description}</p>
      </div>
      <Toggle enabled={rule.enabled} onToggle={onToggle} />
    </div>
  );
}

// ---------------------------------------------------------------------------
// RulesManager
// ---------------------------------------------------------------------------

export function RulesManager() {
  const storeRules = useWorkflowKitStore((s) => s.rules);
  const config = useWorkflowKitStore((s) => s.config);
  const setConfig = useWorkflowKitStore((s) => s.setConfig);
  const setRules = useWorkflowKitStore((s) => s.setRules);

  // Use store rules if populated, otherwise fall back to defaults
  const rules =
    storeRules.length > 0
      ? storeRules
      : DEFAULT_RULES.map((r) => ({
          ...r,
          enabled: !config.disabled_rules.includes(r.name),
        }));

  const handleToggle = async (rule: RuleDef) => {
    const isCurrentlyEnabled = rule.enabled;
    const newDisabledRules = isCurrentlyEnabled
      ? [...config.disabled_rules, rule.name]
      : config.disabled_rules.filter((n) => n !== rule.name);

    const updatedConfig = { ...config, disabled_rules: newDisabledRules };
    const updatedRules = rules.map((r) =>
      r.name === rule.name ? { ...r, enabled: !isCurrentlyEnabled } : r,
    );

    // Optimistic update
    setConfig(updatedConfig);
    setRules(updatedRules);

    try {
      await invoke("update_workflow_kit_config", { config: updatedConfig });
    } catch {
      // Revert on failure
      setConfig(config);
      setRules(rules);
    }
  };

  return (
    <div className="space-y-3 text-sm">
      {/* ---- Header ---- */}
      <label className="text-zinc-400 text-xs uppercase tracking-wide">
        Rules
      </label>

      {/* ---- Rule cards ---- */}
      <div className="space-y-1">
        {rules.map((rule) => (
          <RuleCard
            key={rule.name}
            rule={rule}
            onToggle={() => handleToggle(rule)}
          />
        ))}
      </div>

      {rules.length === 0 && (
        <p className="text-zinc-500 text-xs italic">No rules defined</p>
      )}
    </div>
  );
}
