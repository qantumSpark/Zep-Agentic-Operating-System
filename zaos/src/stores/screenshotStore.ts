import { create } from "zustand";
import type {
  Screenshot,
  Iteration,
} from "../types/screenshots";

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const MAX_SCREENSHOTS = 100;
const MAX_ITERATIONS = 50;

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

interface ScreenshotStoreState {
  // Data
  screenshots: Screenshot[];
  iterations: Iteration[];
  activeIterationId: string | null;
  selectedScreenshotId: string | null;

  // Actions
  setScreenshots: (screenshots: Screenshot[]) => void;
  addScreenshot: (screenshot: Screenshot) => void;
  removeScreenshot: (id: string) => void;
  setIterations: (iterations: Iteration[]) => void;
  updateIteration: (iteration: Iteration) => void;
  setActiveIterationId: (id: string | null) => void;
  selectScreenshot: (id: string | null) => void;
  reset: () => void;
}

export const useScreenshotStore = create<ScreenshotStoreState>((set) => ({
  screenshots: [],
  iterations: [],
  activeIterationId: null,
  selectedScreenshotId: null,

  setScreenshots: (screenshots: Screenshot[]) =>
    set({ screenshots }),

  addScreenshot: (screenshot: Screenshot) =>
    set((state) => ({
      screenshots: [...state.screenshots, screenshot].slice(-MAX_SCREENSHOTS),
    })),

  removeScreenshot: (id: string) =>
    set((state) => ({
      screenshots: state.screenshots.filter((s) => s.id !== id),
      selectedScreenshotId:
        state.selectedScreenshotId === id ? null : state.selectedScreenshotId,
    })),

  setIterations: (iterations: Iteration[]) =>
    set({ iterations: iterations.slice(-MAX_ITERATIONS) }),

  updateIteration: (iteration: Iteration) =>
    set((state) => {
      const exists = state.iterations.some((i) => i.id === iteration.id);
      const updated = exists
        ? state.iterations.map((i) =>
            i.id === iteration.id ? iteration : i
          )
        : [...state.iterations, iteration].slice(-MAX_ITERATIONS);
      return { iterations: updated };
    }),

  setActiveIterationId: (id: string | null) =>
    set({ activeIterationId: id }),

  selectScreenshot: (id: string | null) =>
    set({ selectedScreenshotId: id }),

  reset: () =>
    set({
      screenshots: [],
      iterations: [],
      activeIterationId: null,
      selectedScreenshotId: null,
    }),
}));
