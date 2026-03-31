import { create } from "zustand";
import type { Message } from "../types/events";

interface ChatState {
  messages: Message[];
  streamingTextBuffer: string;
  lastStreamLine: string;
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
  lastStreamLine: "",
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
    set((state) => {
      const newBuffer = state.streamingTextBuffer + text;
      const lastNewline = newBuffer.lastIndexOf('\n');
      const lastLine = lastNewline === -1 ? newBuffer : newBuffer.slice(lastNewline + 1);
      const trimmed = lastLine.trim();
      const lastStreamLine = trimmed.length > 80 ? trimmed.slice(0, 80) + '...' : trimmed;
      return {
        streamingTextBuffer: newBuffer,
        lastStreamLine,
      };
    }),

  getStreamingTextBuffer: () => get().streamingTextBuffer,

  clearStreamingBuffer: () =>
    set({
      streamingTextBuffer: "",
      lastStreamLine: "",
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
      lastStreamLine: "",
      isStreaming: false,
      isThinking: false,
    }),
}));
