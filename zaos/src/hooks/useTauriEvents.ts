import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { CliEvent, AssistantEvent, UserEvent, RateLimitEvent, ResultEvent } from "../types/events";
import { useChatStore } from "../stores/chatStore";
import { useWorkflowStore } from "../stores/workflowStore";
import { useActionsStore } from "../stores/actionsStore";
import { useSessionStore } from "../stores/sessionStore";

/**
 * Hook that listens to Tauri events and dispatches to appropriate stores
 */
export function useTauriEvents() {
  const chatStore = useChatStore();
  const workflowStore = useWorkflowStore();
  const actionsStore = useActionsStore();
  const sessionStore = useSessionStore();

  useEffect(() => {
    let unlisteners: UnlistenFn[] = [];

    const setupListeners = async () => {
      // Listen to agent-event (main CLI event stream)
      const agentEventListener = await listen<CliEvent>(
        "agent-event",
        (event) => {
          const payload = event.payload;

          // Handle system event (initialization)
          if (payload.type === "system") {
            const sysEvent = payload;
            if ("model" in sysEvent && sysEvent.model) {
              sessionStore.setModel(sysEvent.model);
            }
            if ("session_id" in sysEvent) {
              sessionStore.setSessionId(sysEvent.session_id);
              sessionStore.setStartTime(Date.now());
            }
            if ("mcp_servers" in sysEvent && sysEvent.mcp_servers) {
              // Check if GoPeak is connected
              const hasGoPeak = sysEvent.mcp_servers?.some((srv: any) =>
                srv.name?.includes("gopeak")
              );
              sessionStore.updateConnections({
                gopeak: !!hasGoPeak,
              });
            }
            sessionStore.updateConnections({ cli: true });
          }

          // Handle assistant event (messages and tool uses)
          else if (payload.type === "assistant") {
            const assistantEvent = payload as AssistantEvent;
            // Messages are added via stream events or handled by streaming hook
          }

          // Handle user event (tool results)
          else if (payload.type === "user") {
            const userEvent = payload as UserEvent;
            // Tool results are tracked in actions feed
          }

          // Handle rate limit event
          else if (payload.type === "rate_limit_event") {
            const rateLimitEvent = payload as RateLimitEvent;
            // Update connection status
            if (rateLimitEvent.rate_limit_info.status === "allowed") {
              sessionStore.updateConnections({ cli: true });
            }
          }

          // Handle result event (end of turn/session)
          else if (payload.type === "result") {
            const resultEvent = payload as ResultEvent;
            // Update token usage from final result
            if (resultEvent.usage) {
              sessionStore.updateTokenUsage(
                resultEvent.usage.input_tokens || 0,
                resultEvent.usage.output_tokens || 0,
                resultEvent.usage.cache_read_input_tokens || 0
              );
            }
            // Update duration
            sessionStore.setDuration(Math.floor(resultEvent.duration_ms / 1000));
          }
        }
      );
      unlisteners.push(agentEventListener);

      // Listen to workflow-change events
      const workflowChangeListener = await listen(
        "workflow-change",
        (event) => {
          const payload = event.payload as any;
          if (payload.phase) {
            workflowStore.setPhase(payload.phase);
          }
          if (payload.epic) {
            workflowStore.setEpic(payload.epic);
          }
          if (payload.task) {
            workflowStore.setTask(payload.task);
          }
          if (payload.mode) {
            workflowStore.setMode(payload.mode);
          }
          if (payload.gateValidated !== undefined) {
            workflowStore.validateGate(payload.gateValidated);
          }
        }
      );
      unlisteners.push(workflowChangeListener);

      // Listen to cli-health events
      const cliHealthListener = await listen(
        "cli-health",
        (event) => {
          const payload = event.payload as any;
          sessionStore.updateConnections({
            cli: payload.healthy || false,
          });
        }
      );
      unlisteners.push(cliHealthListener);
    };

    setupListeners();

    // Cleanup listeners on unmount
    return () => {
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, [chatStore, workflowStore, actionsStore, sessionStore]);
}
