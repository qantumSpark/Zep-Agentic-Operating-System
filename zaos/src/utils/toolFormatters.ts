export function formatToolSummary(name: string, input: Record<string, unknown>): string {
  switch (name) {
    case "Read":
      return `Read ${input.file_path || "file"}`;
    case "Write":
      return `Write ${input.file_path || "file"}`;
    case "Edit":
      return `Edit ${input.file_path || "file"}`;
    case "Bash":
      return `Run: ${typeof input.command === "string" ? input.command.slice(0, 60) : "command"}`;
    case "Glob":
      return `Search files: ${input.pattern || ""}`;
    case "Grep":
      return `Search content: ${input.pattern || ""}`;
    case "Agent":
      return `Agent: ${input.description || "sub-task"}`;
    default:
      return name;
  }
}
