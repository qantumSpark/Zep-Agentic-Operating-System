import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { ApprovalRequestedEvent } from "../types/zaosEvents";

interface PermissionState {
  pendingRequests: ApprovalRequestedEvent[];

  // Actions
  addRequest: (request: ApprovalRequestedEvent) => void;
  removeRequest: (id: string) => void;
  respondToRequest: (id: string, allow: boolean) => Promise<void>;
}

export const usePermissionStore = create<PermissionState>((set, get) => ({
  pendingRequests: [],

  addRequest: (request: ApprovalRequestedEvent) =>
    set((state) => ({
      pendingRequests: [...state.pendingRequests, request],
    })),

  removeRequest: (id: string) =>
    set((state) => ({
      pendingRequests: state.pendingRequests.filter((r) => r.requestId !== id),
    })),

  respondToRequest: async (id: string, allow: boolean) => {
    try {
      // Find the pending request to get its tool input
      const pending = get().pendingRequests.find((r) => r.requestId === id);
      const toolInput = pending?.toolInput ?? null;

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
