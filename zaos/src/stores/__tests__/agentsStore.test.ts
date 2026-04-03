import { describe, it, expect } from "vitest";
import { mapAgentName } from "../agentsStore";

describe("mapAgentName", () => {
  it("returns agent from explicit prompt reference", () => {
    expect(mapAgentName("general-purpose", "do stuff", "Use .claude/agents/reviewer.md")).toBe("reviewer");
  });

  it("returns subagentType if it is a known agent", () => {
    expect(mapAgentName("coder", "implement stuff")).toBe("coder");
    expect(mapAgentName("researcher", "find info")).toBe("researcher");
  });

  it("scores reviewer higher than coder for review descriptions", () => {
    expect(mapAgentName("general-purpose", "review and validate quality of code")).toBe("reviewer");
  });

  it("scores architect for architecture descriptions", () => {
    expect(mapAgentName("general-purpose", "plan architecture breakdown")).toBe("architect");
  });

  it("scores researcher for research descriptions", () => {
    expect(mapAgentName("general-purpose", "research and verify docs")).toBe("researcher");
  });

  it("scores coder for implementation descriptions", () => {
    expect(mapAgentName("general-purpose", "implement fix for parser bug")).toBe("coder");
  });

  it("falls back to subagentType when no keywords match", () => {
    expect(mapAgentName("general-purpose", "something completely unrelated xyz")).toBe("general-purpose");
  });

  it("gives bonus for subagentType containing agent name", () => {
    // "coder-agent" contains "coder" → +2 bonus
    expect(mapAgentName("coder-agent", "do something")).toBe("coder");
  });
});
