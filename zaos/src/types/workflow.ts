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

/**
 * Product-facing workflow phases (derived from technical phases).
 * Values match the snake_case serialization from the Rust backend.
 */
export enum ProductPhase {
  None = "none",
  Imagine = "imagine",
  Shape = "shape",
  Design = "design",
  Build = "build",
  Verify = "verify",
  Release = "release",
  Learn = "learn",
}

/** Display order for product phases (excludes None) */
export const PRODUCT_PHASE_ORDER: ProductPhase[] = [
  ProductPhase.Imagine,
  ProductPhase.Shape,
  ProductPhase.Design,
  ProductPhase.Build,
  ProductPhase.Verify,
  ProductPhase.Release,
  ProductPhase.Learn,
];

/**
 * Policy profiles for the ZAOS Policy Engine.
 * Values match the kebab-case serialization from the Rust backend.
 */
export enum PolicyProfile {
  Observe = "observe",
  GuidedBuild = "guided-build",
  AutopilotSafe = "autopilot-safe",
  ReleaseGuarded = "release-guarded",
}

/** Human-readable labels for policy profiles */
export const POLICY_PROFILE_LABELS: Record<PolicyProfile, string> = {
  [PolicyProfile.Observe]: "Observe",
  [PolicyProfile.GuidedBuild]: "Guided Build",
  [PolicyProfile.AutopilotSafe]: "Autopilot Safe",
  [PolicyProfile.ReleaseGuarded]: "Release Guarded",
};

/** Short descriptions for policy profiles */
export const POLICY_PROFILE_DESCRIPTIONS: Record<PolicyProfile, string> = {
  [PolicyProfile.Observe]: "Lecture seule, aucune action auto-approuvée",
  [PolicyProfile.GuidedBuild]: "Écritures avec approbation, docs auto-approuvés",
  [PolicyProfile.AutopilotSafe]: "Auto-approve sauf actions destructives",
  [PolicyProfile.ReleaseGuarded]: "Bloque tout sauf docs et tests",
};

/** Human-readable labels for product phases */
export const PRODUCT_PHASE_LABELS: Record<ProductPhase, string> = {
  [ProductPhase.None]: "—",
  [ProductPhase.Imagine]: "Imagine",
  [ProductPhase.Shape]: "Shape",
  [ProductPhase.Design]: "Design",
  [ProductPhase.Build]: "Build",
  [ProductPhase.Verify]: "Verify",
  [ProductPhase.Release]: "Release",
  [ProductPhase.Learn]: "Learn",
};
