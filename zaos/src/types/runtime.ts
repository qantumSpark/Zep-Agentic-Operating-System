export interface RuntimePaths {
  base_dir: string;
  agents_dir: string;
  rules_dir: string;
  settings_file: string;
}

export interface RuntimeInfo {
  /** Current runtime identifier. New runtimes add their variant here (e.g. "codex", "gemini"). */
  kind: "claude" | (string & {});
  name: string;
  paths: RuntimePaths;
}
