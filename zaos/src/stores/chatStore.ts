import { create } from "zustand";
import type { Message } from "../types/events";

interface ChatState {
  messages: Message[];
  streamingTextBuffer: string;
  isStreaming: boolean;
  isThinking: boolean;

  // Actions
  addMessage: (message: Message) => void;
  updateMessage: (id: string, updates: Partial<Message>) => void;
  appendStreamText: (text: string) => void;
  getStreamingTextBuffer: () => string;
  clearStreamingBuffer: () => void;
  setStreaming: (streaming: boolean) => void;
  setThinking: (thinking: boolean) => void;
  clearMessages: () => void;
}

export const useChatStore = create<ChatState>((set, get) => ({
  messages: [],
  streamingTextBuffer: "",
  isStreaming: false,
  isThinking: false,

  addMessage: (message: Message) =>
    set((state) => {
      // Deduplicate: skip if message with same ID already exists
      if (state.messages.some((m) => m.id === message.id)) {
        return state;
      }
      return { messages: [...state.messages, message] };
    }),

  updateMessage: (id: string, updates: Partial<Message>) =>
    set((state) => ({
      messages: state.messages.map((msg) =>
        msg.id === id ? { ...msg, ...updates } : msg
      ),
    })),

  appendStreamText: (text: string) =>
    set((state) => ({
      streamingTextBuffer: state.streamingTextBuffer + text,
    })),

  getStreamingTextBuffer: () => get().streamingTextBuffer,

  clearStreamingBuffer: () =>
    set({
      streamingTextBuffer: "",
    }),

  setStreaming: (streaming: boolean) =>
    set({
      isStreaming: streaming,
    }),

  setThinking: (thinking: boolean) =>
    set({
      isThinking: thinking,
    }),

  clearMessages: () =>
    set({
      messages: [],
      streamingTextBuffer: "",
      isStreaming: false,
      isThinking: false,
    }),
}));
