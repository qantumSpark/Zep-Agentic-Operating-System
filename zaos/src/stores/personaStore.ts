import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { Persona } from "../types/persona";

interface PersonaStoreState {
  personas: Persona[];
  loaded: boolean;
  loadPersonas: () => Promise<void>;
  reset: () => void;
}

export const usePersonaStore = create<PersonaStoreState>((set) => ({
  personas: [],
  loaded: false,

  loadPersonas: async () => {
    try {
      const personas = await invoke<Persona[]>("get_personas");
      set({ personas, loaded: true });
    } catch (e) {
      console.error("Failed to load personas:", e);
      set({ loaded: true });
    }
  },

  reset: () => set({ personas: [], loaded: false }),
}));
