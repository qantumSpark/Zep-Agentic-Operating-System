export const TOOL_ICONS: Record<string, string> = {
  Read: "\u{1F4C4}",
  Write: "\u{270F}\u{FE0F}",
  Edit: "\u{270F}\u{FE0F}",
  Bash: "\u{1F5A5}\u{FE0F}",
  Glob: "\u{1F50D}",
  Grep: "\u{1F50E}",
  Agent: "\u{1F916}",
  Skill: "\u{26A1}",
  WebSearch: "\u{1F310}",
  WebFetch: "\u{1F310}",
  TodoWrite: "\u{1F4DD}",
  TodoRead: "\u{1F4CB}",
};

export function getToolIcon(toolName: string): string {
  return TOOL_ICONS[toolName] || "\u{1F527}";
}
