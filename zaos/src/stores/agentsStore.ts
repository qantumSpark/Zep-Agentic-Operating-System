import { create } from "zustand";
import { useRuntimeStore } from "./runtimeStore";

// ---------------------------------------------------------------------------
// TypeScript interfaces
// ---------------------------------------------------------------------------

export interface DelegationEntry {
  id: string;
  agentType: string;
  mappedAgent: string;
  description: string;
  startedAt: number;
  endedAt: number | null;
  status: "running" | "completed" | "error";
  lastSeenAt: number;
}

export type NewDelegation = Omit<DelegationEntry, "endedAt" | "status" | "lastSeenAt">;

const AGENT_KEYWORDS: Record<string, string[]> = {
  researcher: ["research", "search", "verify", "doc", "find", "check", "info", "collect", "explore"],
  architect: ["architect", "design", "plan", "structure", "breakdown"],
  coder: ["implement", "code", "write", "create", "fix", "build", "task"],
  reviewer: ["review", "quality", "validate", "simplify", "clean"],
  tester: ["test", "spec", "scenario", "verify behavior", "edge case"],
};

/**
 * Map a subagent type and description to a known agent name.
 *
 * Priority:
 *   1. Explicit `.claude/agents/<name>.md` reference in prompt
 *   2. Direct match: subagentType is a known agent name
 *   3. Scoring: count keyword matches per agent, highest score wins
 *      - +1 per keyword found in description
 *      - +2 bonus if subagentType contains the agent name
 *
 * Expected mappings:
 *   ("general-purpose", "research and verify docs") → "researcher"
 *   ("general-purpose", "implement fix for parser bug") → "coder"
 *   ("general-purpose", "review and validate quality of code") → "reviewer" (2 hits vs 1 for coder)
 *   ("general-purpose", "plan architecture breakdown") → "architect" (3 hits)
 *   ("coder", "implement stuff") → "coder" (direct match on step 2)
 */
export function mapAgentName(subagentType: string, description: string, prompt?: string): string {
  // 1. Check prompt for explicit agent file reference
  if (prompt) {
    const agentsDir = useRuntimeStore.getState().info?.paths.agents_dir ?? ".claude/agents";
    // Escape special regex chars in the directory path and build pattern dynamically
    const escaped = agentsDir.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const agentFileRegex = new RegExp(`${escaped}[/\\\\](\\w+)\\.md`);
    const match = prompt.match(agentFileRegex);
    if (match) return match[1];
  }
  // 2. If subagent_type is a known agent name, use it directly
  if (subagentType in AGENT_KEYWORDS) return subagentType;
  // 3. Scoring-based keyword matching on description
  const desc = description.toLowerCase();
  let bestAgent = "";
  let bestScore = 0;

  for (const [agent, keywords] of Object.entries(AGENT_KEYWORDS)) {
    let score = 0;
    for (const kw of keywords) {
      if (desc.includes(kw)) score += 1;
    }
    // Bonus: if subagentType contains the agent name (e.g. "general-purpose" doesn't, but "coder-agent" would)
    if (subagentType.toLowerCase().includes(agent)) score += 2;

    if (score > bestScore) {
      bestScore = score;
      bestAgent = agent;
    }
  }

  return bestAgent || subagentType;
}

const MAX_DELEGATIONS = 50;

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

interface AgentsStoreState {
  // Data
  availableAgents: string[];
  activeAgent: string | null;
  delegations: DelegationEntry[];

  // Actions
  setAvailableAgents: (agents: string[]) => void;
  addDelegation: (entry: NewDelegation) => void;
  completeDelegation: (id: string, status: "completed" | "error") => void;
  updateLastSeen: (id: string) => void;
  setActiveAgent: (agent: string | null) => void;
  reset: () => void;
}

export const useAgentsStore = create<AgentsStoreState>((set) => ({
  availableAgents: [],
  activeAgent: null,
  delegations: [],

  setAvailableAgents: (agents: string[]) =>
    set({ availableAgents: agents }),

  addDelegation: (entry: NewDelegation) =>
    set((state) => {
      if (state.delegations.some((d) => d.id === entry.id)) return state;
      return {
        delegations: [
          ...state.delegations,
          { ...entry, endedAt: null, status: "running" as const, lastSeenAt: entry.startedAt },
        ].slice(-MAX_DELEGATIONS),
        activeAgent: entry.agentType,
      };
    }),

  updateLastSeen: (id: string) =>
    set((state) => {
      const target = state.delegations.find((d) => d.id === id && d.status === "running");
      if (!target || Date.now() - target.lastSeenAt < 1000) return state;
      return {
        delegations: state.delegations.map((d) =>
          d.id === id ? { ...d, lastSeenAt: Date.now() } : d
        ),
      };
    }),

  completeDelegation: (id: string, status: "completed" | "error") =>
    set((state) => {
      const target = state.delegations.find((d) => d.id === id);
      if (!target) return state;

      const now = Date.now();
      return {
        delegations: state.delegations.map((d) =>
          d.id === id ? { ...d, endedAt: now, status } : d
        ),
        activeAgent:
          state.activeAgent === target.agentType ? null : state.activeAgent,
      };
    }),

  setActiveAgent: (agent: string | null) =>
    set({ activeAgent: agent }),

  reset: () =>
    set({
      availableAgents: [],
      activeAgent: null,
      delegations: [],
    }),
}));
