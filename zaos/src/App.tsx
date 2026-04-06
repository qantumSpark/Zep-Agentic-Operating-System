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
import { useThemeStore } from "./stores/themeStore";
import { loadProjectContext } from "./services/projectLoader";

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

      // Global auth check (not project-specific)
      invoke<{ authenticated: boolean; version: string; message: string }>("check_cli_auth")
        .then((auth) => useSessionStore.getState().setCliAuth(auth.authenticated, auth.version, auth.message))
        .catch((e) => {
          console.error("check_cli_auth failed:", e);
          useSessionStore.getState().setCliAuth(false, "", "CLI check failed");
        });

      // Load all project context via single entry point
      await loadProjectContext();
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
