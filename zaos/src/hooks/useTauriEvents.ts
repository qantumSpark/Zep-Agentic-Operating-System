import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { CliEvent, RateLimitEvent, ResultEvent } from "../types/events";
import { useWorkflowStore, type BackendWorkflowPayload } from "../stores/workflowStore";
import { useMemoryStore, type MemoryStateResponse } from "../stores/memoryStore";
import { useSessionStore } from "../stores/sessionStore";
import { useScreenshotStore } from "../stores/screenshotStore";
import { useDiffStore } from "../stores/diffStore";
import type { Screenshot, Iteration } from "../types/screenshots";

/**
 * Hook that listens to Tauri events for session/workflow updates.
 * Registered ONCE (empty deps). Uses getState() to avoid re-registrations.
 */
export function useTauriEvents() {
  useEffect(() => {
    let unlisteners: UnlistenFn[] = [];

    const setupListeners = async () => {
      const agentEventListener = await listen<CliEvent>(
        "agent-event",
        (event) => {
          const sessionStore = useSessionStore.getState();
          const payload = event.payload;
          // System event (initialization)
          if (payload.type === "system") {
            const sysEvent = payload as any;
            if (sysEvent.model) {
              sessionStore.setModel(sysEvent.model);
            }
            if (sysEvent.session_id) {
              sessionStore.setSessionId(sysEvent.session_id);
              sessionStore.setStartTime(Date.now());
            }
            if (sysEvent.mcp_servers) {
              const hasGoPeak = sysEvent.mcp_servers?.some(
                (srv: any) => srv.name?.includes("gopeak")
              );
              sessionStore.updateConnections({ gopeak: !!hasGoPeak });
            }
            sessionStore.updateConnections({ cli: true });
          }

          // Rate limit event
          else if (payload.type === "rate_limit_event") {
            const rle = payload as RateLimitEvent;
            if (rle.rate_limit_info?.status === "allowed") {
              sessionStore.updateConnections({ cli: true });
            }
          }

          // Result event (end of turn)
          else if (payload.type === "result") {
            const result = payload as ResultEvent;
            if (result.usage) {
              const u = result.usage as any;
              sessionStore.updateTokenUsage(
                u.input_tokens || 0,
                u.output_tokens || 0,
                u.cache_read_input_tokens || 0
              );
            }
            if (result.duration_ms) {
              sessionStore.setDuration(
                Math.floor(result.duration_ms / 1000)
              );
            }
          }
        }
      );
      unlisteners.push(agentEventListener);
      // Workflow change events — batch-update store and derive pipeline progress
      const workflowChangeListener = await listen<BackendWorkflowPayload>(
        "workflow-change",
        (event) => {
          const workflowStore = useWorkflowStore.getState();
          workflowStore.setFullState(event.payload);
        }
      );
      unlisteners.push(workflowChangeListener);

      // Memory change events — update memory store when .memory files change
      const memoryChangeListener = await listen<MemoryStateResponse>(
        "memory-change",
        (event) => {
          const memoryStore = useMemoryStore.getState();
          memoryStore.setMemoryState(event.payload);
        }
      );
      unlisteners.push(memoryChangeListener);

      // CLI health events
      const cliHealthListener = await listen("cli-health", (event) => {
        const sessionStore = useSessionStore.getState();
        const payload = event.payload as any;
        sessionStore.updateConnections({ cli: payload.healthy || false });
      });
      unlisteners.push(cliHealthListener);

      // Screenshot events — new screenshot captured
      const screenshotNewListener = await listen<Screenshot>(
        "screenshot-new",
        (event) => {
          useScreenshotStore.getState().addScreenshot(event.payload);
        }
      );
      unlisteners.push(screenshotNewListener);

      // Iteration events — iteration created or status updated
      const iterationUpdateListener = await listen<Iteration>(
        "iteration-update",
        (event) => {
          useScreenshotStore.getState().updateIteration(event.payload);
        }
      );
      unlisteners.push(iterationUpdateListener);

      // MCP notify events — show OS notification or log to console
      const mcpNotifyListener = await listen<{ title: string; message: string }>(
        "mcp-notify",
        (event) => {
          const { title, message } = event.payload;
          if ("Notification" in window && Notification.permission === "granted") {
            new Notification(title, { body: message });
          } else {
            console.info(`[MCP Notify] ${title}: ${message}`);
          }
        }
      );
      unlisteners.push(mcpNotifyListener);

      // MCP show-diff events — store diff for visual review
      const mcpShowDiffListener = await listen<{
        path: string;
        before: string;
        after: string;
        description?: string;
      }>("mcp-show-diff", (event) => {
        useDiffStore.getState().addDiff(event.payload);
      });
      unlisteners.push(mcpShowDiffListener);
    };

    setupListeners();

    return () => {
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, []); // Empty deps — listeners registered once
}