import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface CliSession {
  session_id: string;
  first_prompt: string | null;
  last_prompt: string | null;
  timestamp: string | null;
  last_modified: string | null;
}

interface ListSessionsResponse {
  sessions: CliSession[];
}

interface TokenUsage {
  input: number;
  output: number;
  cache: number;
}

interface Connections {
  cli: boolean;
  gopeak: boolean;
  godot: boolean;
}

interface SessionStoreState {
  // Token tracking
  tokens: TokenUsage;

  // Per-phase token tracking
  phaseTokens: Record<string, { input: number; output: number }>;

  // Per-agent timing tracking (total ms per agent)
  agentTimings: Record<string, number>;

  // Session info
  duration: number; // in seconds
  model: string;
  sessionId: string;
  startTime: number;

  // Connections
  connections: Connections;

  // CLI auth
  cliVersion: string;
  cliAuthMessage: string;

  // Session history
  sessions: CliSession[];
  sessionsLoaded: boolean;

  // Actions
  updateTokenUsage: (input: number, output: number, cache?: number) => void;
  recordPhaseTokens: (phase: string, input: number, output: number) => void;
  recordAgentTiming: (agent: string, durationMs: number) => void;
  setDuration: (seconds: number) => void;
  setModel: (model: string) => void;
  setSessionId: (id: string) => void;
  setStartTime: (time: number) => void;
  updateConnections: (connections: Partial<Connections>) => void;
  setCliAuth: (authenticated: boolean, version: string, message: string) => void;
  resetSession: () => void;
  loadSessions: () => Promise<void>;
}

export const useSessionStore = create<SessionStoreState>((set) => ({
  tokens: {
    input: 0,
    output: 0,
    cache: 0,
  },
  phaseTokens: {},
  agentTimings: {},
  duration: 0,
  model: "",
  sessionId: "",
  startTime: 0,
  connections: {
    cli: false,
    gopeak: false,
    godot: false,
  },
  cliVersion: "",
  cliAuthMessage: "",
  sessions: [],
  sessionsLoaded: false,

  updateTokenUsage: (input: number, output: number, cache: number = 0) =>
    set((state) => ({
      tokens: {
        input: state.tokens.input + input,
        output: state.tokens.output + output,
        cache: state.tokens.cache + cache,
      },
    })),

  recordPhaseTokens: (phase: string, input: number, output: number) =>
    set((state) => {
      const prev = state.phaseTokens[phase] ?? { input: 0, output: 0 };
      return {
        phaseTokens: {
          ...state.phaseTokens,
          [phase]: {
            input: prev.input + input,
            output: prev.output + output,
          },
        },
      };
    }),

  recordAgentTiming: (agent: string, durationMs: number) =>
    set((state) => ({
      agentTimings: {
        ...state.agentTimings,
        [agent]: (state.agentTimings[agent] ?? 0) + durationMs,
      },
    })),

  setDuration: (seconds: number) =>
    set({
      duration: seconds,
    }),

  setModel: (model: string) =>
    set({
      model,
    }),

  setSessionId: (id: string) =>
    set({
      sessionId: id,
    }),

  setStartTime: (time: number) =>
    set({
      startTime: time,
    }),

  updateConnections: (connections: Partial<Connections>) =>
    set((state) => ({
      connections: {
        ...state.connections,
        ...connections,
      },
    })),

  setCliAuth: (authenticated: boolean, version: string, message: string) =>
    set((state) => ({
      cliVersion: version,
      cliAuthMessage: message,
      connections: { ...state.connections, cli: authenticated },
    })),

  resetSession: () =>
    set({
      tokens: {
        input: 0,
        output: 0,
        cache: 0,
      },
      phaseTokens: {},
      agentTimings: {},
      duration: 0,
      model: "",
      sessionId: "",
      startTime: 0,
      connections: {
        cli: false,
        gopeak: false,
        godot: false,
      },
    }),

  loadSessions: async () => {
    try {
      const response = await invoke<ListSessionsResponse>("list_sessions");
      set({ sessions: response.sessions, sessionsLoaded: true });
    } catch (err) {
      console.error("Failed to load sessions:", err);
      set({ sessionsLoaded: true });
    }
  },
}));
