import { create } from "zustand";

export interface AgentDef {
  name: string;        // filename without .md (e.g., "architect")
  deployed: boolean;   // exists in .claude/agents/
  description: string; // first line of the .md file
  custom: boolean;     // user-created (not from default set)
}

export interface HookDef {
  name: string;        // e.g., "inject-context", "block-code"
  event: string;       // e.g., "UserPromptSubmit", "PreToolUse"
  active: boolean;
  description: string;
}

export interface RuleDef {
  name: string;        // filename (e.g., "01-always.mdc")
  deployed: boolean;
  enabled: boolean;    // not in disabled_rules list
  description: string;
}

export interface WorkflowKitConfig {
  blocked_extensions: string[];
  auto_sync: boolean;
  hooks_enabled: boolean;
  disabled_rules: string[];
}

interface WorkflowKitState {
  // State
  deployed: boolean;
  agents: AgentDef[];
  hooks: HookDef[];
  rules: RuleDef[];
  config: WorkflowKitConfig;
  lastDeployedAt: string | null;

  // Actions
  setDeployed: (deployed: boolean) => void;
  setAgents: (agents: AgentDef[]) => void;
  setHooks: (hooks: HookDef[]) => void;
  setRules: (rules: RuleDef[]) => void;
  setConfig: (config: WorkflowKitConfig) => void;
  setLastDeployedAt: (timestamp: string) => void;
  reset: () => void;
}

const DEFAULT_HOOKS: HookDef[] = [
  { name: "inject-context", event: "UserPromptSubmit", active: false, description: "Injecte le role Orchestrateur et les regles a chaque prompt" },
  { name: "block-code", event: "PreToolUse (Write|Edit)", active: false, description: "Bloque l'ecriture de code sans plan valide" },
  { name: "on-compact", event: "SessionStart (compact)", active: false, description: "Reinjecte le contexte apres compaction" },
];

const DEFAULT_CONFIG: WorkflowKitConfig = {
  blocked_extensions: [".rs", ".ts", ".tsx", ".js", ".jsx", ".css"],
  auto_sync: true,
  hooks_enabled: true,
  disabled_rules: [],
};

export const useWorkflowKitStore = create<WorkflowKitState>((set) => ({
  deployed: false,
  agents: [],
  hooks: DEFAULT_HOOKS,
  rules: [],
  config: DEFAULT_CONFIG,
  lastDeployedAt: null,

  setDeployed: (deployed) => set({ deployed }),
  setAgents: (agents) => set({ agents }),
  setHooks: (hooks) => set({ hooks }),
  setRules: (rules) => set({ rules }),
  setConfig: (config) => set({ config }),
  setLastDeployedAt: (timestamp) => set({ lastDeployedAt: timestamp }),

  reset: () =>
    set({
      deployed: false,
      agents: [],
      hooks: DEFAULT_HOOKS,
      rules: [],
      config: DEFAULT_CONFIG,
      lastDeployedAt: null,
    }),
}));
