import { create } from "zustand";
import type { Action } from "../types/events";

const MAX_ACTIONS = 200;

interface ActionsStoreState {
  actions: Action[];

  // Actions
  addAction: (action: Action) => void;
  updateAction: (id: string, updates: Partial<Action>) => void;
  clearActions: () => void;
}

export const useActionsStore = create<ActionsStoreState>((set) => ({
  actions: [],

  addAction: (action: Action) =>
    set((state) => ({
      actions: [...state.actions, action].slice(-MAX_ACTIONS),
    })),

  updateAction: (id: string, updates: Partial<Action>) =>
    set((state) => ({
      actions: state.actions.map((action) =>
        action.id === id ? { ...action, ...updates } : action
      ),
    })),

  clearActions: () =>
    set({
      actions: [],
    }),
}));
