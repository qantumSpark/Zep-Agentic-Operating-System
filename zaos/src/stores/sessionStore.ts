import { create } from "zustand";

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

  // Session info
  duration: number; // in seconds
  model: string;
  sessionId: string;
  startTime: number;

  // Connections
  connections: Connections;

  // CLI auth
  cliAuthenticated: boolean;
  cliVersion: string;
  cliAuthMessage: string;

  // Actions
  updateTokenUsage: (input: number, output: number, cache?: number) => void;
  setDuration: (seconds: number) => void;
  setModel: (model: string) => void;
  setSessionId: (id: string) => void;
  setStartTime: (time: number) => void;
  updateConnections: (connections: Partial<Connections>) => void;
  setCliAuth: (authenticated: boolean, version: string, message: string) => void;
  resetSession: () => void;
}

export const useSessionStore = create<SessionStoreState>((set) => ({
  tokens: {
    input: 0,
    output: 0,
    cache: 0,
  },
  duration: 0,
  model: "",
  sessionId: "",
  startTime: 0,
  connections: {
    cli: false,
    gopeak: false,
    godot: false,
  },
  cliAuthenticated: false,
  cliVersion: "",
  cliAuthMessage: "",

  updateTokenUsage: (input: number, output: number, cache: number = 0) =>
    set((state) => ({
      tokens: {
        input: state.tokens.input + input,
        output: state.tokens.output + output,
        cache: state.tokens.cache + cache,
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
    set({ cliAuthenticated: authenticated, cliVersion: version, cliAuthMessage: message }),

  resetSession: () =>
    set({
      tokens: {
        input: 0,
        output: 0,
        cache: 0,
      },
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
}));
