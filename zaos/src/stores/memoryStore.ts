import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

// ---------------------------------------------------------------------------
// TypeScript interfaces matching Rust serde types from memory/reader.rs
// ---------------------------------------------------------------------------

export interface MemoryIndexEntry {
  title: string;
  filename: string;
  description: string;
}

export interface MemoryIndexSection {
  heading: string;
  entries: MemoryIndexEntry[];
}

export interface MilestoneEntry {
  number: number;
  name: string;
  status: string;
  epics: string;
}

export interface EpicTask {
  number: number;
  name: string;
  files: string[];
  status: string;
  notes: string;
}

export interface CurrentEpic {
  name: string;
  milestone: string;
  status: string;
  objective: string;
  tasks: EpicTask[];
}

export interface MemoryState {
  milestones: MilestoneEntry[];
  active_epic: string;
  blocages: string;
}

// ---------------------------------------------------------------------------
// Memory health report (from get_memory_health IPC)
// ---------------------------------------------------------------------------

export interface FileHealthEntry {
  name: string;
  present: boolean;
  parseable: boolean;
  warnings: Array<{ field: string; message: string }>;
}

export interface MemoryHealthReport {
  files: FileHealthEntry[];
  epic_name_match: boolean;
  epic_name_workflow: string;
  epic_name_memory: string;
  warnings: string[];
}

/**
 * Shape of the backend get_memory_state IPC response (snake_case from Rust serde).
 */
export interface MemoryStateResponse {
  index: { sections: MemoryIndexSection[] };
  state: MemoryState | null;
  current_epic: CurrentEpic | null;
}

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

interface MemoryStoreState {
  // Data
  index: MemoryIndexSection[];
  milestones: MilestoneEntry[];
  activeEpic: string;
  blocages: string;
  currentEpic: CurrentEpic | null;
  health: MemoryHealthReport | null;
  loaded: boolean;

  // Actions
  setMemoryState: (response: MemoryStateResponse) => void;
  setHealth: (health: MemoryHealthReport) => void;
  loadHealth: () => Promise<void>;
  reset: () => void;
}

function applyMemoryResponse(response: MemoryStateResponse) {
  return {
    index: response.index.sections,
    milestones: response.state?.milestones ?? [],
    activeEpic: response.state?.active_epic ?? "",
    blocages: response.state?.blocages ?? "",
    currentEpic: response.current_epic,
    loaded: true,
  };
}

export const useMemoryStore = create<MemoryStoreState>((set) => ({
  index: [],
  milestones: [],
  activeEpic: "",
  blocages: "",
  currentEpic: null,
  health: null,
  loaded: false,

  setMemoryState: (response: MemoryStateResponse) =>
    set(applyMemoryResponse(response)),

  setHealth: (health: MemoryHealthReport) => set({ health }),

  loadHealth: async () => {
    try {
      const report = await invoke<MemoryHealthReport>("get_memory_health");
      set({ health: report });
    } catch (e) {
      console.error("[memoryStore] loadHealth failed:", e);
    }
  },

  reset: () =>
    set({
      index: [],
      milestones: [],
      activeEpic: "",
      blocages: "",
      currentEpic: null,
      health: null,
      loaded: false,
    }),
}));
