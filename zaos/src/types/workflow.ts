/**
 * Type definitions for workflow state
 */

export enum Phase {
  Idle = "idle",
  Comprehension = "comprehension",
  Specification = "specification",
  Architecture = "architecture",
  Implementation = "implementation",
  Review = "review",
  Test = "test",
  Closure = "closure",
}

export type PhaseState = "idle" | "done" | "active" | "pending";

export interface WorkflowState {
  epic: Epic;
  phase: Phase;
  mode: WorkflowMode;
  task: string;
  gateValidated: boolean;
  pipelineProgress: Record<Phase, PhaseState>;
  currentAgents: string[];
  timestamp: number;
}

export interface Epic {
  name: string;
  description: string;
  startTime: number;
}

export type WorkflowMode = "free" | "pipeline";

export interface AgentInfo {
  name: string;
  filePath: string;
  status: "active" | "inactive" | "error";
  lastDelegation?: {
    taskDescription: string;
    timestamp: number;
  };
}

export interface PipelinePhaseInfo {
  phase: Phase;
  state: PhaseState;
  estimatedDuration?: number;
  completedAt?: number;
  agentResponsible?: string;
  summary?: string;
}

export const PHASE_ORDER: Phase[] = [
  Phase.Idle,
  Phase.Comprehension,
  Phase.Specification,
  Phase.Architecture,
  Phase.Implementation,
  Phase.Review,
  Phase.Test,
  Phase.Closure,
];

export function phaseIndex(phase: Phase): number {
  return PHASE_ORDER.indexOf(phase);
}

export function nextPhase(current: Phase): Phase | null {
  const idx = phaseIndex(current);
  if (idx >= 0 && idx < PHASE_ORDER.length - 1) {
    return PHASE_ORDER[idx + 1];
  }
  return null;
}
