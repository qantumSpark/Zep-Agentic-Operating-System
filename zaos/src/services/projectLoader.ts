import { invoke } from "@tauri-apps/api/core";
import { useRuntimeStore } from "../stores/runtimeStore";
import { useMemoryStore, type MemoryStateResponse, type MemoryHealthReport } from "../stores/memoryStore";
import { useScreenshotStore } from "../stores/screenshotStore";
import { useWorkflowStore, type BackendWorkflowPayload } from "../stores/workflowStore";
import { useWorkflowKitStore, applyKitStatus, type AgentDef, type KitStatusResponse } from "../stores/workflowKitStore";
import { useProductStore } from "../stores/productStore";
import { usePersonaStore } from "../stores/personaStore";
import { useSessionStore } from "../stores/sessionStore";
import { useChatStore } from "../stores/chatStore";
import { useActionsStore } from "../stores/actionsStore";
import { useDiffStore } from "../stores/diffStore";
import { usePermissionStore } from "../stores/permissionStore";
import { useAgentsStore } from "../stores/agentsStore";
import { useProjectStore } from "../stores/projectStore";
import type { Screenshot } from "../types/screenshots";
import type { ProductContract } from "../types/productContract";

/**
 * Reset synchrone de tous les stores lies au projet.
 * themeStore est preserve (preference utilisateur globale).
 */
export function resetAllProjectStores(): void {
  // Stores avec donnees IPC (reset + reload ensuite)
  useRuntimeStore.getState().reset();
  useMemoryStore.getState().reset();
  useScreenshotStore.getState().reset();
  useWorkflowStore.getState().resetWorkflow();
  useWorkflowKitStore.getState().reset();
  useProductStore.getState().reset();
  usePersonaStore.getState().reset();

  // Stores sans reload IPC (reset seulement)
  useSessionStore.getState().resetSession();
  useChatStore.getState().clearMessages();
  useActionsStore.getState().clearActions();
  useDiffStore.getState().clearDiffs();
  usePermissionStore.getState().reset();
  useAgentsStore.getState().reset();
  useProjectStore.getState().reset();
}

interface LoadProjectContextOptions {
  projectPath?: string;
  projectName?: string;
}

/**
 * Point d'entree unique pour charger le contexte d'un projet.
 * Appele au boot ET lors d'un changement de projet.
 */
export async function loadProjectContext(opts?: LoadProjectContextOptions): Promise<void> {
  // 1. Reset tous les stores
  resetAllProjectStores();

  // 2. Charger ou appliquer le project info
  if (opts?.projectPath && opts?.projectName) {
    useProjectStore.getState().setProject(opts.projectPath, opts.projectName);
  } else {
    try {
      const projectInfo = await invoke<{ path: string; name: string }>("get_project_info");
      useProjectStore.getState().setProject(projectInfo.path, projectInfo.name);
    } catch (e) {
      console.error("get_project_info failed:", e);
    }
  }

  // 3. Charger runtime en premier (source de verite pour les paths)
  await useRuntimeStore.getState().loadRuntimeInfo();

  // 4. Charger tout le reste en parallele (tolerant aux erreurs individuelles)
  const results = await Promise.allSettled([
    invoke<MemoryStateResponse>("get_memory_state")
      .then((mem) => useMemoryStore.getState().setMemoryState(mem)),
    invoke<{ screenshots: Screenshot[] }>("get_screenshots")
      .then((res) => useScreenshotStore.getState().setScreenshots(res.screenshots)),
    invoke<BackendWorkflowPayload>("get_workflow_state")
      .then((wf) => useWorkflowStore.getState().setFullState(wf)),
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
    invoke<ProductContract>("get_product_contract")
      .then((contract) => useProductStore.getState().setProductContract(contract)),
    usePersonaStore.getState().loadPersonas(),
    invoke<MemoryHealthReport>("get_memory_health")
      .then((report) => useMemoryStore.getState().setHealth(report)),
  ]);

  // Log les echecs individuels
  const labels = ["memory", "screenshots", "workflow", "kit-status", "agents", "product-contract", "personas", "memory-health"];
  results.forEach((r, i) => {
    if (r.status === "rejected") {
      console.error(`[loadProjectContext] ${labels[i]} failed:`, r.reason);
    }
  });
}
