import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { ApprovalRequestedEvent } from "../types/zaosEvents";

export interface PolicyLogEntry {
  timestamp: number;
  toolName: string;
  verdict: "allow" | "ask" | "deny";
  riskLevel: string;
  reason: string;
  wasAutoApproved: boolean;
  wasAutoDenied: boolean;
}

interface PermissionState {
  pendingRequests: ApprovalRequestedEvent[];

  // Actions
  addRequest: (request: ApprovalRequestedEvent) => void;
  removeRequest: (id: string) => void;
  respondToRequest: (id: string, allow: boolean) => Promise<void>;

  // Policy log
  policyLog: PolicyLogEntry[];
  addPolicyLogEntry: (entry: PolicyLogEntry) => void;
  clearPolicyLog: () => void;

  // Reset
  reset: () => void;
}

export const usePermissionStore = create<PermissionState>((set, get) => ({
  pendingRequests: [],
  policyLog: [],

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

  addPolicyLogEntry: (entry: PolicyLogEntry) =>
    set((state) => ({
      policyLog: [...state.policyLog, entry].slice(-200),
    })),

  clearPolicyLog: () => set({ policyLog: [] }),

  reset: () => set({ pendingRequests: [], policyLog: [] }),
}));
