import { create } from "zustand";
import type { WorkflowState, Epic, WorkflowMode, Phase, PhaseState } from "../types/workflow";
import { PHASE_ORDER, ProductPhase } from "../types/workflow";

/**
 * Shape of the backend WorkflowState payload (snake_case from Rust serde).
 */
export interface BackendWorkflowPayload {
  phase: string;
  epic: string;
  task: string;
  mode: WorkflowMode;
  gate_validated: boolean;
  gate_ready?: boolean;
  permission_mode?: string;
  last_updated: string;
  history?: Array<{
    from_phase: string;
    to_phase: string;
    timestamp: string;
    reason?: string;
  }>;
  product_phase?: string;
  next_product_phase?: string | null;
  next_tech_phase?: string | null;
  session?: {
    session_id: string;
    started_at: string;
    updated_at: string;
    tokens_used: {
      input: number;
      output: number;
      cache_read: number;
      cache_creation: number;
    };
  } | null;
}

/**
 * Derive pipeline progress from the current phase.
 * Phases before the current one are "done", the current is "active",
 * phases after are "pending". If phase is "idle" or unrecognized, all are "pending".
 */
function derivePipelineProgress(
  currentPhase: string
): Record<string, PhaseState> {
  const progress: Record<string, PhaseState> = {};
  const idx = PHASE_ORDER.indexOf(currentPhase as Phase);

  for (let i = 0; i < PHASE_ORDER.length; i++) {
    const p = PHASE_ORDER[i];
    if (idx < 0) {
      // Unknown phase or idle — everything pending
      progress[p] = "pending";
    } else if (i < idx) {
      progress[p] = "done";
    } else if (i === idx) {
      progress[p] = "active";
    } else {
      progress[p] = "pending";
    }
  }

  return progress;
}

interface WorkflowStoreState {
  phase: Phase | null;
  epic: Epic | null;
  task: string;
  mode: WorkflowMode;
  gateValidated: boolean;
  gateReady: boolean;
  permissionMode: string;
  productPhase: ProductPhase;
  nextProductPhase: ProductPhase | null;
  nextTechPhase: string | null;
  pipelineProgress: Record<string, PhaseState>;

  // Actions
  updateState: (state: Partial<WorkflowState>) => void;
  setFullState: (payload: BackendWorkflowPayload) => void;
  setPhase: (phase: Phase) => void;
  setEpic: (epic: Epic) => void;
  setTask: (task: string) => void;
  setMode: (mode: WorkflowMode) => void;
  validateGate: (validated: boolean) => void;
  setPermissionMode: (mode: string) => void;
  updatePipelineProgress: (
    phase: Phase,
    state: PhaseState
  ) => void;
  resetWorkflow: () => void;
}

export const useWorkflowStore = create<WorkflowStoreState>((set) => ({
  phase: null,
  epic: null,
  task: "",
  mode: "pipeline",
  gateValidated: false,
  gateReady: false,
  permissionMode: "strict",
  productPhase: ProductPhase.None,
  nextProductPhase: null,
  nextTechPhase: null,
  pipelineProgress: {},

  updateState: (state: Partial<WorkflowState>) =>
    set({
      phase: state.phase,
      epic: state.epic,
      task: state.task,
      mode: state.mode,
      gateValidated: state.gateValidated,
      pipelineProgress: state.pipelineProgress || {},
    }),

  setFullState: (payload: BackendWorkflowPayload) => {
    const phase = (payload.phase || "idle") as Phase;
    const pipelineProgress = derivePipelineProgress(phase);

    // Map backend epic string to Epic object
    const epic: Epic | null = payload.epic
      ? { name: payload.epic, description: "", startTime: 0 }
      : null;

    set({
      phase,
      epic,
      task: payload.task || "",
      mode: payload.mode || "free",
      gateValidated: payload.gate_validated ?? false,
      gateReady: payload.gate_ready ?? false,
      permissionMode: payload.permission_mode || "strict",
      productPhase: Object.values(ProductPhase).includes(payload.product_phase as ProductPhase)
        ? (payload.product_phase as ProductPhase)
        : ProductPhase.None,
      nextProductPhase: payload.next_product_phase && Object.values(ProductPhase).includes(payload.next_product_phase as ProductPhase)
        ? (payload.next_product_phase as ProductPhase)
        : null,
      nextTechPhase: payload.next_tech_phase || null,
      pipelineProgress,
    });
  },

  setPhase: (phase: Phase) =>
    set({
      phase,
    }),

  setEpic: (epic: Epic) =>
    set({
      epic,
    }),

  setTask: (task: string) =>
    set({
      task,
    }),

  setMode: (mode: WorkflowMode) =>
    set({
      mode,
    }),

  validateGate: (validated: boolean) =>
    set({
      gateValidated: validated,
    }),

  setPermissionMode: (mode: string) => set({ permissionMode: mode }),

  updatePipelineProgress: (phase: Phase, state: "idle" | "done" | "active" | "pending") =>
    set((current) => ({
      pipelineProgress: {
        ...current.pipelineProgress,
        [phase]: state,
      },
    })),

  resetWorkflow: () =>
    set({
      phase: null,
      epic: null,
      task: "",
      mode: "pipeline",
      gateValidated: false,
      gateReady: false,
      permissionMode: "strict",
      productPhase: ProductPhase.None,
      nextProductPhase: null,
      nextTechPhase: null,
      pipelineProgress: {},
    }),
}));
