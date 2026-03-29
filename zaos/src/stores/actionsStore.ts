import { create } from "zustand";
import type { Action } from "../types/events";

interface ActionsStoreState {
  actions: Action[];

  // Actions
  addAction: (action: Action) => void;
  updateActionStatus: (
    id: string,
    status: "pending" | "running" | "success" | "error"
  ) => void;
  updateAction: (id: string, updates: Partial<Action>) => void;
  clearActions: () => void;
}

export const useActionsStore = create<ActionsStoreState>((set) => ({
  actions: [],

  addAction: (action: Action) =>
    set((state) => ({
      actions: [...state.actions, action],
    })),

  updateActionStatus: (
    id: string,
    status: "pending" | "running" | "success" | "error"
  ) =>
    set((state) => ({
      actions: state.actions.map((action) =>
        action.id === id ? { ...action, status } : action
      ),
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
