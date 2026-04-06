import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { sendNotification, isPermissionGranted } from "@tauri-apps/plugin-notification";
import type { ZaosEvent } from "../types/zaosEvents";
import { useWorkflowStore, type BackendWorkflowPayload } from "../stores/workflowStore";
import { useMemoryStore, type MemoryStateResponse } from "../stores/memoryStore";
import { useSessionStore } from "../stores/sessionStore";
import { useScreenshotStore } from "../stores/screenshotStore";
import { useDiffStore } from "../stores/diffStore";
import { useWorkflowKitStore, type AgentDef } from "../stores/workflowKitStore";
import { useProductStore } from "../stores/productStore";
import { useAgentsStore } from "../stores/agentsStore";
import { loadProjectContext } from "../services/projectLoader";
import type { ProductContract } from "../types/productContract";
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
      const agentEventListener = await listen<ZaosEvent>(
        "agent-event",
        (event) => {
          const sessionStore = useSessionStore.getState();
          const ze = event.payload;

          switch (ze.type) {
            case "session_init": {
              if (ze.model) {
                sessionStore.setModel(ze.model);
              }
              if (ze.sessionId) {
                sessionStore.setSessionId(ze.sessionId);
                sessionStore.setStartTime(Date.now());
              }
              if (ze.mcpServers) {
                const hasGoPeak = (ze.mcpServers as { name?: string }[])?.some(
                  (srv) => srv.name?.includes("gopeak")
                );
                sessionStore.updateConnections({ gopeak: !!hasGoPeak });
              }
              if (!sessionStore.connections.cli) {
                sessionStore.updateConnections({ cli: true });
              }
              break;
            }

            case "rate_limited": {
              if (ze.status === "allowed") {
                if (!sessionStore.connections.cli) {
                  sessionStore.updateConnections({ cli: true });
                }
              }
              break;
            }

            case "token_usage": {
              sessionStore.updateTokenUsage(
                ze.inputTokens,
                ze.outputTokens,
                ze.cacheReadTokens
              );

              const phase = useWorkflowStore.getState().phase;
              if (phase) {
                sessionStore.recordPhaseTokens(phase, ze.inputTokens, ze.outputTokens);
              }
              break;
            }

            case "run_completed": {
              if (ze.durationMs) {
                sessionStore.setDuration(
                  Math.floor(ze.durationMs / 1000)
                );

                if (ze.durationMs > 120_000) {
                  notifyIfPermitted(
                    "Tache terminee",
                    `Session terminee en ${formatDuration(ze.durationMs)}`
                  );
                }
              }

              const { sessionId: sid, tokens: { input, output }, duration, agentTimings } = useSessionStore.getState();
              const currentPhase = useWorkflowStore.getState().phase;

              if (sid && (input > 0 || output > 0)) {
                invoke("save_session_log", {
                  data: {
                    session_id: ze.sessionId || sid,
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

                // Save session insights (best-effort, non-blocking)
                invoke("save_session_insights", {
                  duration_secs: duration,
                  tokens_input: input,
                  tokens_output: output,
                  agents_used: Object.keys(agentTimings),
                }).catch((e: unknown) =>
                  console.error("Failed to save session insights:", e)
                );
              }

              const agentsStore = useAgentsStore.getState();
              const staleDelegations = agentsStore.delegations.filter(
                (d) => d.status === "running"
              );
              for (const stale of staleDelegations) {
                agentsStore.completeDelegation(stale.id, "completed");
              }
              break;
            }

            default:
              break;
          }
        }
      );
      unlisteners.push(agentEventListener);
      // Workflow change events — batch-update store and derive pipeline progress
      const workflowChangeListener = await listen<BackendWorkflowPayload>(
        "workflow-change",
        (event) => {
          const workflowStore = useWorkflowStore.getState();
          // Diagnostic: log phase transitions
          const prevPhase = workflowStore.phase;
          workflowStore.setFullState(event.payload);

          // Refresh memory health (best-effort) — health compares state.json.epic vs current-epic
          const memoryStore = useMemoryStore.getState();
          memoryStore.loadHealth().catch((e: unknown) =>
            console.error("[workflow-change] loadHealth failed:", e)
          );

          if (prevPhase !== event.payload.phase) {
            console.info(
              `[Workflow] Phase transition: ${prevPhase ?? "null"} → ${event.payload.phase}`,
              { gate_ready: event.payload.gate_ready, gate_validated: event.payload.gate_validated }
            );
          }

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
          // Refresh health report (best-effort)
          memoryStore.loadHealth().catch((e) =>
            console.error("[memory-change] loadHealth failed:", e)
          );
        }
      );
      unlisteners.push(memoryChangeListener);

      // Product contract change events — update product store when .memory files change
      const productContractChangeListener = await listen<ProductContract>(
        "product-contract-change",
        (event) => {
          useProductStore.getState().setProductContract(event.payload);
        }
      );
      unlisteners.push(productContractChangeListener);

      // CLI health events
      const cliHealthListener = await listen("cli-health", (event) => {
        const sessionStore = useSessionStore.getState();
        const payload = event.payload as { healthy?: boolean };
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

      // Runtime directory change events — refresh agents list (only for agents/ changes)
      const runtimeDirChangeListener = await listen<{ file: string }>(
        "runtime-dir-change",
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
      unlisteners.push(runtimeDirChangeListener);

      // Project changed — delegate to centralized loader
      const projectChangedListener = await listen<{ path: string; name: string }>(
        "project-changed",
        (event) => {
          const { path, name } = event.payload;
          loadProjectContext({ projectPath: path, projectName: name })
            .catch((e) => console.error("[project-changed] loadProjectContext failed:", e));
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
