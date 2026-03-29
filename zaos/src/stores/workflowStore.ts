import { create } from "zustand";
import type { WorkflowState, Epic, WorkflowMode, Phase } from "../types/workflow";

interface WorkflowStoreState {
  phase: Phase | null;
  epic: Epic | null;
  task: string;
  mode: WorkflowMode;
  gateValidated: boolean;
  pipelineProgress: Record<string, "idle" | "done" | "active" | "pending">;

  // Actions
  updateState: (state: Partial<WorkflowState>) => void;
  setPhase: (phase: Phase) => void;
  setEpic: (epic: Epic) => void;
  setTask: (task: string) => void;
  setMode: (mode: WorkflowMode) => void;
  validateGate: (validated: boolean) => void;
  updatePipelineProgress: (
    phase: Phase,
    state: "idle" | "done" | "active" | "pending"
  ) => void;
  resetWorkflow: () => void;
}

export const useWorkflowStore = create<WorkflowStoreState>((set) => ({
  phase: null,
  epic: null,
  task: "",
  mode: "free",
  gateValidated: false,
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
      mode: "free",
      gateValidated: false,
      pipelineProgress: {},
    }),
}));
