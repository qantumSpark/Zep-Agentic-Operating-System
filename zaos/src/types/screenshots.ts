// ---------------------------------------------------------------------------
// TypeScript interfaces mirroring Rust screenshot types (camelCase via serde)
// ---------------------------------------------------------------------------

export interface Screenshot {
  id: string;
  filename: string;
  path: string;
  timestamp: string;
  sessionId: string | null;
  iterationId: string | null;
  source: CaptureSource;
  metadata: ScreenshotMetadata;
}

export type CaptureSource = "cliMcp" | "filesystem" | "manual";

export interface ScreenshotMetadata {
  url: string | null;
  viewport: Viewport | null;
  projectType: string | null;
  label: string | null;
}

export interface Viewport {
  width: number;
  height: number;
}

export interface Iteration {
  id: string;
  startedAt: string;
  status: IterationStatus;
  screenshots: string[]; // screenshot ids
  findings: string[];
  targetDescription: string | null;
}

export type IterationStatus =
  | "capturing"
  | "evaluating"
  | "fixing"
  | "comparing"
  | "done";
