import { useMemo } from "react";
import { useAgentsStore, DelegationEntry } from "../../stores/agentsStore";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function formatDuration(startedAt: number, endedAt: number | null): string {
  if (endedAt === null) return "running\u2026";
  const ms = endedAt - startedAt;
  if (ms < 1_000) return `${ms}ms`;
  const seconds = Math.floor(ms / 1_000);
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  return `${minutes}m ${remainingSeconds}s`;
}

// ---------------------------------------------------------------------------
// Status indicators
// ---------------------------------------------------------------------------

function AgentDot({ isActive }: { isActive: boolean }) {
  return (
    <span
      className={`inline-block w-2 h-2 rounded-full flex-shrink-0 ${
        isActive ? "bg-green-400" : "bg-zinc-500"
      }`}
    />
  );
}

function DelegationBadge({ status }: { status: DelegationEntry["status"] }) {
  const colors: Record<DelegationEntry["status"], string> = {
    running: "bg-blue-700/60 text-blue-300",
    completed: "bg-green-700/60 text-green-300",
    error: "bg-red-700/60 text-red-300",
  };
  return (
    <span
      className={`inline-block px-1.5 py-0.5 rounded text-[10px] font-semibold uppercase leading-none ${colors[status]}`}
    >
      {status}
    </span>
  );
}

// ---------------------------------------------------------------------------
// AgentsSection
// ---------------------------------------------------------------------------

export function AgentsSection() {
  const availableAgents = useAgentsStore((s) => s.availableAgents);
  const activeAgent = useAgentsStore((s) => s.activeAgent);
  const delegations = useAgentsStore((s) => s.delegations);

  const recentDelegations = useMemo(
    () => [...delegations].reverse().slice(0, 10),
    [delegations],
  );

  return (
    <div className="space-y-4 text-sm">
      {/* ---- Available Agents ---- */}
      <div>
        <label className="text-zinc-400 text-xs uppercase tracking-wide">
          Available Agents
        </label>

        {availableAgents.length === 0 ? (
          <p className="text-zinc-500 text-xs italic mt-1">
            No agents detected
          </p>
        ) : (
          <ul className="mt-1 space-y-1">
            {availableAgents.map((agent) => (
              <li
                key={agent}
                className="flex items-center gap-2 px-2 py-1.5 rounded bg-zinc-800/60 hover:bg-zinc-800/80 transition-colors"
              >
                <AgentDot isActive={agent === activeAgent} />
                <span className="text-xs text-zinc-200 truncate">{agent}</span>
                {agent === activeAgent && (
                  <DelegationBadge status="running" />
                )}
              </li>
            ))}
          </ul>
        )}
      </div>

      {/* ---- Delegation History ---- */}
      {recentDelegations.length > 0 && (
        <div>
          <label className="text-zinc-400 text-xs uppercase tracking-wide">
            Delegation History
          </label>

          <div className="mt-1 space-y-1 max-h-64 overflow-y-auto">
            {recentDelegations.map((d) => (
              <div
                key={d.id}
                className={`flex items-start gap-2 px-2 py-1.5 rounded transition-colors text-xs ${
                  d.status === "running"
                    ? "bg-blue-500/10 ring-1 ring-blue-500/30"
                    : "bg-zinc-800/30 hover:bg-zinc-800/50"
                }`}
              >
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="font-medium text-zinc-200">
                      {d.agentType}
                    </span>
                    <DelegationBadge status={d.status} />
                  </div>

                  <p className="text-zinc-400 mt-0.5 truncate">
                    {d.description}
                  </p>

                  <div className="flex items-center gap-2 text-zinc-500 mt-0.5">
                    <span>{formatDuration(d.startedAt, d.endedAt)}</span>
                    <span className="text-zinc-600">&middot;</span>
                    <span>
                      {new Date(d.startedAt).toLocaleTimeString()}
                    </span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* ---- Empty state ---- */}
      {availableAgents.length === 0 && recentDelegations.length === 0 && (
        <p className="text-zinc-500 text-xs italic">
          No agent activity yet
        </p>
      )}
    </div>
  );
}
