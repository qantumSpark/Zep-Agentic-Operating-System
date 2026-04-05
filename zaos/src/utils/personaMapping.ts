const PHASE_PERSONA_MAP: Record<string, string> = {
  comprehension: "Product Architect",
  specification: "Product Architect",
  architecture: "Product Architect",
  implementation: "Builder",
  review: "Reviewer",
  test: "Tester",
  closure: "Release Manager",
};

export function getSuggestedPersonaForPhase(phase: string): string | null {
  return PHASE_PERSONA_MAP[phase] ?? null;
}
