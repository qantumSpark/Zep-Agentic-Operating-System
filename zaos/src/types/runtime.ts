export interface RuntimePaths {
  base_dir: string;
  agents_dir: string;
  rules_dir: string;
  settings_file: string;
}

export interface RuntimeInfo {
  kind: "claude";
  name: string;
  paths: RuntimePaths;
}
