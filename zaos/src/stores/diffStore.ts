import { create } from "zustand";

const MAX_DIFFS = 20;

export interface DiffEntry {
  path: string;
  before: string;
  after: string;
  description?: string;
  timestamp: string;
}

interface DiffStore {
  diffs: DiffEntry[];
  activeDiffIndex: number | null;
  addDiff: (diff: Omit<DiffEntry, "timestamp">) => void;
  setActiveDiff: (index: number | null) => void;
  clearDiffs: () => void;
}

export const useDiffStore = create<DiffStore>((set) => ({
  diffs: [],
  activeDiffIndex: null,
  addDiff: (diff) =>
    set((state) => ({
      diffs: [...state.diffs, { ...diff, timestamp: new Date().toISOString() }].slice(-MAX_DIFFS),
      activeDiffIndex: state.diffs.length,
    })),
  setActiveDiff: (index) => set({ activeDiffIndex: index }),
  clearDiffs: () => set({ diffs: [], activeDiffIndex: null }),
}));
