/* React 19 JSX transform */
import { useMemoryStore } from "../../stores/memoryStore";
import { useWorkflowStore } from "../../stores/workflowStore";
import { useSessionStore } from "../../stores/sessionStore";
import { StatusBadge } from "../common/StatusBadge";

export function StartupDashboard() {
  // Memory
  const memoryLoaded = useMemoryStore((s) => s.loaded);
  const milestones = useMemoryStore((s) => s.milestones);
  const currentEpic = useMemoryStore((s) => s.currentEpic);
  const activeEpic = useMemoryStore((s) => s.activeEpic);

  // Workflow
  const phase = useWorkflowStore((s) => s.phase);
  const mode = useWorkflowStore((s) => s.mode);
  const task = useWorkflowStore((s) => s.task);

  // Session
  const cliConnected = useSessionStore((s) => s.connections.cli);
  const cliVersion = useSessionStore((s) => s.cliVersion);

  return (
    <div className="h-full flex items-center justify-center px-6">
      <div className="w-full max-w-md space-y-6">
        {/* ---- Branding ---- */}
        <div className="text-center">
          <h1 className="text-2xl font-bold tracking-tight text-zinc-100">
            ZAOS
          </h1>
          <p className="text-xs text-zinc-500 mt-1">
            Zep Agentic Operating System
          </p>
        </div>

        {/* ---- CLI Status ---- */}
        <div className="flex items-center justify-center gap-2 text-xs">
          <span
            className={`inline-block w-2 h-2 rounded-full ${
              cliConnected ? "bg-green-500" : "bg-zinc-600"
            }`}
          />
          <span className={cliConnected ? "text-zinc-300" : "text-zinc-500"}>
            {cliConnected
              ? `CLI connecte${cliVersion ? ` (${cliVersion})` : ""}`
              : "CLI non connecte"}
          </span>
        </div>

        {/* ---- Current Epic ---- */}
        {memoryLoaded && (currentEpic || activeEpic) && (
          <div className="rounded-lg border border-zinc-700/60 bg-zinc-800/40 p-4 space-y-2">
            <label className="text-zinc-400 text-[10px] uppercase tracking-widest font-semibold">
              Epic en cours
            </label>
            <p className="text-zinc-100 text-sm font-medium">
              {currentEpic?.name ?? activeEpic}
            </p>
            {currentEpic?.status && (
              <StatusBadge status={currentEpic.status} />
            )}
            {currentEpic?.objective && (
              <p className="text-zinc-400 text-xs leading-relaxed">
                {currentEpic.objective}
              </p>
            )}
          </div>
        )}

        {/* ---- Milestones ---- */}
        {memoryLoaded && milestones.length > 0 && (
          <div className="rounded-lg border border-zinc-700/60 bg-zinc-800/40 p-4 space-y-2">
            <label className="text-zinc-400 text-[10px] uppercase tracking-widest font-semibold">
              Milestones
            </label>
            <ul className="space-y-1">
              {milestones.map((m) => (
                <li
                  key={m.number}
                  className="flex items-center justify-between gap-2 text-xs"
                >
                  <span className="text-zinc-300 truncate">
                    <span className="text-zinc-500 mr-1">#{m.number}</span>
                    {m.name}
                  </span>
                  <StatusBadge status={m.status} />
                </li>
              ))}
            </ul>
          </div>
        )}

        {/* ---- Start Epic CTA — only when no epic active ---- */}
        {!currentEpic && !activeEpic && (!phase || phase === "idle") && (
          <div className="rounded-lg border border-zinc-700/60 bg-zinc-800/40 p-4 text-center space-y-3">
            <p className="text-zinc-300 text-sm">
              Aucun epic en cours
            </p>
            <p className="text-zinc-500 text-xs">
              Demarrez un epic depuis la section Workflow du dashboard, ou tapez directement un message.
            </p>
          </div>
        )}

        {/* ---- Workflow State ---- */}
        {(phase || task) && (
          <div className="flex flex-wrap items-center justify-center gap-2">
            {phase && (
              <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-zinc-800 border border-zinc-700 text-xs text-zinc-300">
                <span className="w-1.5 h-1.5 rounded-full bg-blue-400" />
                {phase}
              </span>
            )}
            <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-zinc-800 border border-zinc-700 text-xs text-zinc-300">
              <span
                className={`w-1.5 h-1.5 rounded-full ${
                  mode === "pipeline" ? "bg-amber-400" : "bg-zinc-500"
                }`}
              />
              {mode}
            </span>
            {task && (
              <span className="inline-flex items-center px-2.5 py-1 rounded-full bg-zinc-800 border border-zinc-700 text-xs text-zinc-400 max-w-[260px] truncate">
                {task}
              </span>
            )}
          </div>
        )}

        {/* ---- Hint ---- */}
        <p className="text-center text-zinc-600 text-xs select-none">
          Tapez un message pour commencer...
        </p>
      </div>
    </div>
  );
}
