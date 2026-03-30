import React, { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { SplitPane } from "./components/common/SplitPane";
import { ChatPanel } from "./components/chat/ChatPanel";
import { DashboardPanel } from "./components/dashboard/DashboardPanel";
import { StatusBar } from "./components/statusbar/StatusBar";
import { useTauriEvents } from "./hooks/useTauriEvents";
import { useStreaming } from "./hooks/useStreaming";
import { useSessionStore } from "./stores/sessionStore";
import { useMemoryStore, type MemoryStateResponse } from "./stores/memoryStore";

/**
 * Main App component
 * Layout: split-panel (chat left, dashboard right) with StatusBar at bottom
 * Sets up Tauri event listeners
 */
export function App() {
  // Initialize event listeners
  useTauriEvents();
  useStreaming();

  useEffect(() => {
    console.log("ZAOS initialized");
    (async () => {
      try {
        const result = await invoke<{ authenticated: boolean; version: string; message: string }>("check_cli_auth");
        useSessionStore.getState().setCliAuth(result.authenticated, result.version, result.message);
      } catch (err) {
        console.error("check_cli_auth failed:", err);
        useSessionStore.getState().setCliAuth(false, "", "CLI check failed");
      }
      try {
        const memState = await invoke<MemoryStateResponse>("get_memory_state");
        useMemoryStore.getState().setMemoryState(memState);
      } catch (err) {
        console.error("get_memory_state failed:", err);
      }
    })();
  }, []);

  return (
    <div className="flex flex-col h-screen w-screen bg-zinc-900">
      {/* Main content area with split pane */}
      <div className="flex-1 min-h-0">
        <SplitPane
          left={<ChatPanel />}
          right={<DashboardPanel />}
          initialSplit={50}
          minSize={250}
        />
      </div>

      {/* Status bar */}
      <StatusBar />
    </div>
  );
}
