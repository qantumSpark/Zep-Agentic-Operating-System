import { useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useChatStore } from "../stores/chatStore";

interface ShortcutDef {
  key: string;
  ctrl: boolean;
  shift: boolean;
  label: string;
  description: string;
}

export const SHORTCUTS: ShortcutDef[] = [
  {
    key: "n",
    ctrl: true,
    shift: false,
    label: "Ctrl/Cmd+N",
    description: "Focus the input bar",
  },
  {
    key: "k",
    ctrl: true,
    shift: true,
    label: "Ctrl/Cmd+Shift+K",
    description: "Clear chat history",
  },
  {
    key: "g",
    ctrl: true,
    shift: false,
    label: "Ctrl/Cmd+G",
    description: "Validate workflow gate",
  },
  {
    key: "Escape",
    ctrl: false,
    shift: false,
    label: "Escape",
    description: "Clear input (when not typing)",
  },
];

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  const tag = target.tagName.toLowerCase();
  if (tag === "input" || tag === "textarea") return true;
  if (target.isContentEditable) return true;
  return false;
}

export function useKeyboardShortcuts() {
  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    // Prevent rapid-fire from held keys
    if (e.repeat) return;

    const mod = e.metaKey || e.ctrlKey;

    // Escape — works even inside editable fields
    if (e.key === "Escape") {
      const input = document.getElementById(
        "main-input"
      ) as HTMLTextAreaElement | null;
      if (input) {
        // Use the native setter to update value, then dispatch so React picks it up
        const nativeSetter = Object.getOwnPropertyDescriptor(
          HTMLTextAreaElement.prototype,
          "value"
        )?.set;
        if (nativeSetter) {
          nativeSetter.call(input, "");
          input.dispatchEvent(new Event("input", { bubbles: true }));
        }
      }
      return;
    }

    // All other shortcuts are suppressed while typing in editable fields
    if (isEditableTarget(e.target)) return;

    // Ctrl/Cmd + N — focus input bar
    if (mod && !e.shiftKey && e.key.toLowerCase() === "n") {
      e.preventDefault();
      document.getElementById("main-input")?.focus();
      return;
    }

    // Ctrl/Cmd + Shift + K — clear chat
    if (mod && e.shiftKey && e.key.toLowerCase() === "k") {
      e.preventDefault();
      useChatStore.getState().clearMessages();
      return;
    }

    // Ctrl/Cmd + G — validate gate
    if (mod && !e.shiftKey && e.key.toLowerCase() === "g") {
      e.preventDefault();
      invoke("validate_gate").catch((err: unknown) => {
        console.error("validate_gate failed:", err);
      });
      return;
    }
  }, []);

  useEffect(() => {
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [handleKeyDown]);
}
