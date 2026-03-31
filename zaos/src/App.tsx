import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { isPermissionGranted, requestPermission } from "@tauri-apps/plugin-notification";
import { SplitPane } from "./components/common/SplitPane";
import { ChatPanel } from "./components/chat/ChatPanel";
import { DashboardPanel } from "./components/dashboard/DashboardPanel";
import { StatusBar } from "./components/statusbar/StatusBar";
import { useTauriEvents } from "./hooks/useTauriEvents";
import { useStreaming } from "./hooks/useStreaming";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import { useSessionStore } from "./stores/sessionStore";
import { useMemoryStore, type MemoryStateResponse } from "./stores/memoryStore";
import { useScreenshotStore } from "./stores/screenshotStore";
import { useThemeStore } from "./stores/themeStore";
import type { Screenshot } from "./types/screenshots";

/**
 * Main App component
 * Layout: split-panel (chat left, dashboard right) with StatusBar at bottom
 * Sets up Tauri event listeners
 */
export function App() {
  // Initialize event listeners
  useTauriEvents();
  useStreaming();
  useKeyboardShortcuts();

  const theme = useThemeStore((s) => s.theme);

  // Sync theme attribute to <html> whenever it changes (including on mount)
  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
  }, [theme]);

  useEffect(() => {
    console.log("ZAOS initialized");
    (async () => {
      // Request notification permission on startup
      const notifGranted = await isPermissionGranted();
      if (!notifGranted) {
        await requestPermission();
      }

      const [authResult, memResult, screenshotResult] = await Promise.allSettled([
        invoke<{ authenticated: boolean; version: string; message: string }>("check_cli_auth"),
        invoke<MemoryStateResponse>("get_memory_state"),
        invoke<{ screenshots: Screenshot[] }>("get_screenshots"),
      ]);
      if (authResult.status === "fulfilled") {
        useSessionStore.getState().setCliAuth(authResult.value.authenticated, authResult.value.version, authResult.value.message);
      } else {
        console.error("check_cli_auth failed:", authResult.reason);
        useSessionStore.getState().setCliAuth(false, "", "CLI check failed");
      }
      if (memResult.status === "fulfilled") {
        useMemoryStore.getState().setMemoryState(memResult.value);
      } else {
        console.error("get_memory_state failed:", memResult.reason);
      }
      if (screenshotResult.status === "fulfilled") {
        useScreenshotStore.getState().setScreenshots(screenshotResult.value.screenshots);
      } else {
        console.error("get_screenshots failed:", screenshotResult.reason);
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
