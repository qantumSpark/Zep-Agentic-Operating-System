import React, { useEffect } from "react";
import { SplitPane } from "./components/common/SplitPane";
import { ChatPanel } from "./components/chat/ChatPanel";
import { DashboardPanel } from "./components/dashboard/DashboardPanel";
import { StatusBar } from "./components/statusbar/StatusBar";
import { useTauriEvents } from "./hooks/useTauriEvents";
import { useStreaming } from "./hooks/useStreaming";

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
    // Initialize app - can fetch initial data here
    console.log("ZAOS initialized");
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
