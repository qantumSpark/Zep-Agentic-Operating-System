import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { ControlRequest } from "../types/events";

interface PermissionState {
  pendingRequests: ControlRequest[];

  // Actions
  addRequest: (request: ControlRequest) => void;
  removeRequest: (id: string) => void;
  respondToRequest: (id: string, allow: boolean) => Promise<void>;
}

export const usePermissionStore = create<PermissionState>((set, get) => ({
  pendingRequests: [],

  addRequest: (request: ControlRequest) =>
    set((state) => ({
      pendingRequests: [...state.pendingRequests, request],
    })),

  removeRequest: (id: string) =>
    set((state) => ({
      pendingRequests: state.pendingRequests.filter((r) => r.request_id !== id),
    })),

  respondToRequest: async (id: string, allow: boolean) => {
    try {
      // Find the pending request to get its tool input
      const pending = get().pendingRequests.find((r) => r.request_id === id);
      const toolInput = pending?.request?.input ?? pending?.input ?? null;

      await invoke("respond_permission", {
        id,
        allow,
        toolInput: allow ? toolInput : null,
      });
      get().removeRequest(id);
    } catch (error: unknown) {
      console.error("Failed to send permission response:", error);
    }
  },
}));
