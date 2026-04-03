// TypeScript interfaces matching Rust serde types from memory/models.rs

export interface ProductBrief {
  vision: string;
  audience: string;
  rationale: string;
  constraints: string[];
  out_of_scope: string[];
  success_definition: string;
}

export interface ExperienceGoal {
  number: number;
  quality: string;
  criterion: string;
  priority: string;
}

export interface ExperienceGoals {
  goals: ExperienceGoal[];
  ux_standards: string[];
  anti_patterns: string[];
}

export interface AcceptanceCheck {
  number: number;
  check: string;
  status: string;
  notes: string;
}

export interface AcceptanceChecks {
  checks: AcceptanceCheck[];
  manual_validations: string[];
}

export interface ReleaseChecklistItem {
  number: number;
  item: string;
  status: string;
  blocking: string;
  notes: string;
}

export interface ReleaseReadiness {
  overall_state_summary: string;
  checklist: ReleaseChecklistItem[];
  open_risks: string[];
}

export interface SessionInsights {
  date: string;
  epic: string;
  phase: string;
  decisions: string[];
  learnings: string[];
  risks: string[];
  next_validations: string[];
}

export interface ProductContract {
  brief: ProductBrief | null;
  experience_goals: ExperienceGoals | null;
  acceptance_checks: AcceptanceChecks | null;
  release_readiness: ReleaseReadiness | null;
  session_insights: SessionInsights | null;
}
