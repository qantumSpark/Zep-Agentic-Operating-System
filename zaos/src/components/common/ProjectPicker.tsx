import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { useProjectStore } from "../../stores/projectStore";

/**
 * Compact project picker for the status bar.
 * Shows current project name; click to open a native folder picker
 * and switch to a different project directory.
 */
export function ProjectPicker() {
  const projectName = useProjectStore((state) => state.projectName);
  const isLoading = useProjectStore((state) => state.isLoading);

  const handlePickProject = async () => {
    if (isLoading) return;

    const selected = await open({ directory: true });
    if (!selected) return;

    useProjectStore.getState().setLoading(true);
    try {
      await invoke("switch_project", { path: selected });
    } catch (err) {
      console.error("Failed to switch project:", err);
    } finally {
      useProjectStore.getState().setLoading(false);
    }
  };

  const displayName = projectName || "No project";

  return (
    <button
      onClick={handlePickProject}
      disabled={isLoading}
      className="flex items-center gap-1.5 px-2 py-0.5 rounded hover:bg-zinc-700 transition-colors text-zinc-300 hover:text-zinc-100 disabled:opacity-50 disabled:cursor-wait max-w-[160px]"
      title={projectName ? `Project: ${projectName} — click to change` : "Select a project folder"}
    >
      {isLoading ? (
        <span className="w-3 h-3 border border-zinc-400 border-t-transparent rounded-full animate-spin shrink-0" />
      ) : (
        <svg
          className="w-3 h-3 shrink-0 text-zinc-400"
          viewBox="0 0 16 16"
          fill="currentColor"
        >
          <path d="M1 3.5A1.5 1.5 0 0 1 2.5 2h3.879a1.5 1.5 0 0 1 1.06.44l1.122 1.12A1.5 1.5 0 0 0 9.62 4H13.5A1.5 1.5 0 0 1 15 5.5v7a1.5 1.5 0 0 1-1.5 1.5h-11A1.5 1.5 0 0 1 1 12.5v-9Z" />
        </svg>
      )}
      <span className="truncate">{displayName}</span>
    </button>
  );
}
