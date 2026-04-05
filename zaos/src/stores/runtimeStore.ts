import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { RuntimeInfo } from "../types/runtime";

interface RuntimeStoreState {
  info: RuntimeInfo | null;
  loaded: boolean;
  loadRuntimeInfo: () => Promise<void>;
  reset: () => void;
}

export const useRuntimeStore = create<RuntimeStoreState>((set) => ({
  info: null,
  loaded: false,
  loadRuntimeInfo: async () => {
    try {
      const info = await invoke<RuntimeInfo>("get_runtime_info");
      set({ info, loaded: true });
    } catch (e) {
      console.error("Failed to load runtime info:", e);
      set({ loaded: true });
    }
  },
  reset: () => set({ info: null, loaded: false }),
}));
