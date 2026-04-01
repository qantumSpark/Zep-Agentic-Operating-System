import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { sendNotification, isPermissionGranted } from "@tauri-apps/plugin-notification";
import type { CliEvent, RateLimitEvent, ResultEvent } from "../types/events";
import { useWorkflowStore, type BackendWorkflowPayload } from "../stores/workflowStore";
import { useMemoryStore, type MemoryStateResponse } from "../stores/memoryStore";
import { useSessionStore } from "../stores/sessionStore";
import { useScreenshotStore } from "../stores/screenshotStore";
import { useDiffStore } from "../stores/diffStore";
import { useWorkflowKitStore, applyKitStatus, type AgentDef, type KitStatusResponse } from "../stores/workflowKitStore";
import { useProjectStore } from "../stores/projectStore";
import { useAgentsStore } from "../stores/agentsStore";
import type { Screenshot, Iteration } from "../types/screenshots";
import { formatDuration } from "../utils/formatDuration";

/** Cached notification permission — doesn't change at runtime */
let notificationPermissionCached: boolean | null = null;

/** Send a native notification if permission is granted, otherwise log to console */
async function notifyIfPermitted(title: string, body: string): Promise<void> {
  if (notificationPermissionCached === null) {
    notificationPermissionCached = await isPermissionGranted();
  }
  if (notificationPermissionCached) {
    sendNotification({ title, body });
  } else {
    console.info(`[Notify] ${title}: ${body}`);
  }
}

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
            if (!sessionStore.connections.cli) {
              sessionStore.updateConnections({ cli: true });
            }
          }

          // Rate limit event
          else if (payload.type === "rate_limit_event") {
            const rle = payload as RateLimitEvent;
            if (rle.rate_limit_info?.status === "allowed") {
              if (!sessionStore.connections.cli) {
                sessionStore.updateConnections({ cli: true });
              }
            }
          }

          // Result event (end of turn)
          else if (payload.type === "result") {
            const result = payload as ResultEvent;
            if (result.usage) {
              const u = result.usage as any;
              const inputTokens = u.input_tokens || 0;
              const outputTokens = u.output_tokens || 0;
              sessionStore.updateTokenUsage(
                inputTokens,
                outputTokens,
                u.cache_read_input_tokens || 0
              );

              // Record tokens per workflow phase
              const phase = useWorkflowStore.getState().phase;
              if (phase) {
                sessionStore.recordPhaseTokens(phase, inputTokens, outputTokens);
              }
            }
            if (result.duration_ms) {
              sessionStore.setDuration(
                Math.floor(result.duration_ms / 1000)
              );

              // Notify for long-running tasks (> 2 minutes)
              if (result.duration_ms > 120_000) {
                notifyIfPermitted(
                  "Tache terminee",
                  `Session terminee en ${formatDuration(result.duration_ms)}`
                );
              }
            }

            // Save session log to .memory/sessions/
            const { sessionId: sid, tokens: { input, output }, duration, agentTimings } = useSessionStore.getState();
            const currentPhase = useWorkflowStore.getState().phase;

            if (sid && (input > 0 || output > 0)) {
              invoke("save_session_log", {
                data: {
                  session_id: sid,
                  phase: currentPhase || "unknown",
                  tokens_input: input,
                  tokens_output: output,
                  duration_secs: duration,
                  agent_timings: Object.entries(agentTimings).map(
                    ([name, duration_ms]) => ({ name, duration_ms })
                  ),
                },
              }).catch((e: unknown) =>
                console.error("Failed to save session log:", e)
              );
            }

            // Cleanup stale running delegations — result event means turn is over
            const agentsStore = useAgentsStore.getState();
            const staleDelegations = agentsStore.delegations.filter(
              (d) => d.status === "running"
            );
            for (const stale of staleDelegations) {
              agentsStore.completeDelegation(stale.id, "completed");
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

          // Notify when gate becomes ready for validation
          const state = event.payload;
          if (state.gate_ready && !state.gate_validated) {
            notifyIfPermitted(
              "Gate prete",
              `Phase "${state.phase}" en attente de validation`
            );
          }
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

      // MCP notify events — show OS notification via tauri-plugin-notification
      const mcpNotifyListener = await listen<{ title: string; message: string }>(
        "mcp-notify",
        (event) => {
          const { title, message } = event.payload;
          notifyIfPermitted(title, message);
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

      // .claude/ directory change events — refresh agents list (only for agents/ changes)
      const claudeDirChangeListener = await listen<{ file: string }>(
        "claude-dir-change",
        (event) => {
          const file = (event.payload as { file?: string })?.file ?? "";
          if (file && !file.includes("agents")) return;
          invoke<{ name: string; description: string }[]>("list_agents")
            .then((raw) => {
              const mapped: AgentDef[] = raw.map((a) => ({
                name: a.name,
                description: a.description,
                deployed: true,
                custom: false,
              }));
              useWorkflowKitStore.getState().setAgents(mapped);
            })
            .catch(console.error);
        }
      );
      unlisteners.push(claudeDirChangeListener);

      // Project changed — reset all stores and reload data
      const projectChangedListener = await listen<{ path: string; name: string }>(
        "project-changed",
        (event) => {
          const { path, name } = event.payload;

          // Atomic update: project info + clear loading in one set()
          useProjectStore.setState({ projectDir: path, projectName: name, isLoading: false });

          // Reset all stores
          useWorkflowStore.getState().resetWorkflow();
          useMemoryStore.getState().reset();
          useSessionStore.getState().resetSession();
          useScreenshotStore.getState().reset();
          useWorkflowKitStore.getState().reset();

          // Reload all project data in parallel
          Promise.all([
            invoke<MemoryStateResponse>("get_memory_state")
              .then((mem) => useMemoryStore.getState().setMemoryState(mem)),
            invoke<{ screenshots: Screenshot[] }>("get_screenshots")
              .then((res) => useScreenshotStore.getState().setScreenshots(res.screenshots)),
            invoke<KitStatusResponse>("get_workflow_kit_status")
              .then(applyKitStatus),
            invoke<{ name: string; description: string }[]>("list_agents")
              .then((agents) => {
                const mapped: AgentDef[] = agents.map((a) => ({
                  name: a.name,
                  description: a.description,
                  deployed: true,
                  custom: false,
                }));
                useWorkflowKitStore.getState().setAgents(mapped);
              }),
          ]).catch((e) => console.error("Failed to reload project data:", e));
        }
      );
      unlisteners.push(projectChangedListener);
    };

    setupListeners();

    return () => {
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, []); // Empty deps — listeners registered once
}