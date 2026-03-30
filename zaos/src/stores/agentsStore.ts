import { create } from "zustand";

// ---------------------------------------------------------------------------
// TypeScript interfaces
// ---------------------------------------------------------------------------

export interface DelegationEntry {
  id: string;
  agentType: string;
  description: string;
  startedAt: number;
  endedAt: number | null;
  status: "running" | "completed" | "error";
}

export type NewDelegation = Omit<DelegationEntry, "endedAt" | "status">;

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
    set((state) => ({
      delegations: [
        ...state.delegations,
        { ...entry, endedAt: null, status: "running" as const },
      ].slice(-MAX_DELEGATIONS),
      activeAgent: entry.agentType,
    })),

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
