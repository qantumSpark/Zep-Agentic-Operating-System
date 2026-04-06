import { create } from "zustand";
import type { WorkflowMode, PhaseState } from "../types/workflow";
import { Phase, PHASE_ORDER, ProductPhase, PolicyProfile } from "../types/workflow";

/**
 * Shape of the backend WorkflowState payload (snake_case from Rust serde).
 */
export interface BackendWorkflowPayload {
  phase: string;
  epic: string;
  mode: WorkflowMode;
  gate_validated: boolean;
  gate_ready?: boolean;
  permission_mode?: string;
  policy_profile?: string;
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
  epic_status?: string;
  epic_objective?: string;
  epic_task_count?: number;
  epic_done_count?: number;
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
  epic: string | null;
  epicStatus: string | null;
  epicObjective: string | null;
  epicTaskCount: number | null;
  epicDoneCount: number | null;
  mode: WorkflowMode;
  gateValidated: boolean;
  gateReady: boolean;
  permissionMode: string;
  policyProfile: PolicyProfile;
  productPhase: ProductPhase;
  nextProductPhase: ProductPhase | null;
  nextTechPhase: string | null;
  pipelineProgress: Record<string, PhaseState>;

  // Actions
  setFullState: (payload: BackendWorkflowPayload) => void;
  setPhase: (phase: Phase) => void;
  setEpic: (epic: string) => void;
  setMode: (mode: WorkflowMode) => void;
  validateGate: (validated: boolean) => void;
  setPermissionMode: (mode: string) => void;
  setPolicyProfile: (profile: PolicyProfile) => void;
  updatePipelineProgress: (
    phase: Phase,
    state: PhaseState
  ) => void;
  resetWorkflow: () => void;
}

export const useWorkflowStore = create<WorkflowStoreState>((set) => ({
  phase: null,
  epic: null,
  epicStatus: null,
  epicObjective: null,
  epicTaskCount: null,
  epicDoneCount: null,
  mode: "pipeline",
  gateValidated: false,
  gateReady: false,
  permissionMode: "strict",
  policyProfile: PolicyProfile.GuidedBuild,
  productPhase: ProductPhase.None,
  nextProductPhase: null,
  nextTechPhase: null,
  pipelineProgress: {},

  setFullState: (payload: BackendWorkflowPayload) => {
    // Validate phase against known enum values
    const rawPhase = payload.phase || "idle";
    const validPhases = Object.values(Phase) as string[];
    let phase: Phase;
    if (validPhases.includes(rawPhase)) {
      phase = rawPhase as Phase;
    } else {
      console.warn("[workflowStore] Unknown phase received:", rawPhase);
      phase = Phase.Idle;
    }

    // Validate mode against known values
    const rawMode = payload.mode || "pipeline";
    const validModes: WorkflowMode[] = ["free", "pipeline"];
    let mode: WorkflowMode;
    if ((validModes as string[]).includes(rawMode)) {
      mode = rawMode as WorkflowMode;
    } else {
      console.warn("[workflowStore] Unknown mode received:", rawMode);
      // Default to "pipeline" to match backend WorkflowMode::default()
      mode = "pipeline";
    }

    const pipelineProgress = derivePipelineProgress(phase);

    set({
      phase,
      epic: payload.epic || null,
      epicStatus: payload.epic_status ?? null,
      epicObjective: payload.epic_objective ?? null,
      epicTaskCount: payload.epic_task_count ?? null,
      epicDoneCount: payload.epic_done_count ?? null,
      mode,
      gateValidated: payload.gate_validated ?? false,
      gateReady: payload.gate_ready ?? false,
      permissionMode: payload.permission_mode || "strict",
      policyProfile: Object.values(PolicyProfile).includes(payload.policy_profile as PolicyProfile)
        ? (payload.policy_profile as PolicyProfile)
        : PolicyProfile.GuidedBuild,
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

  setEpic: (epic: string) =>
    set({
      epic,
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

  setPolicyProfile: (profile: PolicyProfile) => set({ policyProfile: profile }),

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
      epicStatus: null,
      epicObjective: null,
      epicTaskCount: null,
      epicDoneCount: null,
      mode: "pipeline",
      gateValidated: false,
      gateReady: false,
      permissionMode: "strict",
      policyProfile: PolicyProfile.GuidedBuild,
      productPhase: ProductPhase.None,
      nextProductPhase: null,
      nextTechPhase: null,
      pipelineProgress: {},
    }),
}));
