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
  addDelegation: (
    entry: Omit<DelegationEntry, "endedAt" | "status"> & {
      status?: "running";
    }
  ) => void;
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

  addDelegation: (
    entry: Omit<DelegationEntry, "endedAt" | "status"> & {
      status?: "running";
    }
  ) =>
    set((state) => ({
      delegations: [
        ...state.delegations,
        {
          ...entry,
          endedAt: null,
          status: entry.status ?? "running",
        },
      ],
      activeAgent: entry.agentType,
    })),

  completeDelegation: (id: string, status: "completed" | "error") =>
    set((state) => {
      const now = Date.now();
      const updatedDelegations = state.delegations.map((d) =>
        d.id === id ? { ...d, endedAt: now, status } : d
      );
      const completed = state.delegations.find((d) => d.id === id);
      const newActiveAgent =
        completed && state.activeAgent === completed.agentType
          ? null
          : state.activeAgent;

      return {
        delegations: updatedDelegations,
        activeAgent: newActiveAgent,
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
