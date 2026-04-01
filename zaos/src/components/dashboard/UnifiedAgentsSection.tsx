import { useMemo, useState, useEffect } from "react";
import { useWorkflowKitStore } from "../../stores/workflowKitStore";
import { useAgentsStore, DelegationEntry } from "../../stores/agentsStore";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function timeAgo(ms: number, now: number): string {
  const sec = Math.floor((now - ms) / 1000);
  if (sec < 60) return `${sec}s`;
  const min = Math.floor(sec / 60);
  if (min < 60) return `${min}m`;
  return `${Math.floor(min / 60)}h${min % 60}m`;
}

function capitalize(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}

function truncate(s: string, max: number): string {
  if (s.length <= max) return s;
  return s.slice(0, max) + "\u2026";
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

function StatusDot({ active }: { active: boolean }) {
  return (
    <span
      className={`inline-block w-2 h-2 rounded-full flex-shrink-0 ${
        active ? "bg-green-400" : "bg-zinc-500"
      }`}
    />
  );
}

const STALE_THRESHOLD_MS = 600_000; // 10 minutes

function DelegationBadge({ status, stale }: { status: DelegationEntry["status"]; stale?: boolean }) {
  const colors: Record<DelegationEntry["status"], string> = {
    running: "bg-blue-700/60 text-blue-300",
    completed: "bg-green-700/60 text-green-300",
    error: "bg-red-700/60 text-red-300",
  };
  if (stale) {
    return (
      <span className="inline-block px-1.5 py-0.5 rounded text-[10px] font-semibold uppercase leading-none bg-zinc-700/60 text-zinc-400">
        stale
      </span>
    );
  }
  return (
    <span
      className={`inline-block px-1.5 py-0.5 rounded text-[10px] font-semibold uppercase leading-none ${colors[status]}`}
    >
      {status}
    </span>
  );
}

// ---------------------------------------------------------------------------
// UnifiedAgentsSection
// ---------------------------------------------------------------------------

export function UnifiedAgentsSection() {
  const agents = useWorkflowKitStore((s) => s.agents);
  const delegations = useAgentsStore((s) => s.delegations);

  // Tick for live "time ago" updates
  const hasRunning = delegations.some((d) => d.status === "running");
  const [now, setNow] = useState(Date.now());

  useEffect(() => {
    if (!hasRunning) return;
    const id = setInterval(() => setNow(Date.now()), 1_000);
    return () => clearInterval(id);
  }, [hasRunning]);

  // Build a set of agent names with running delegations
  const runningAgentNames = useMemo(() => {
    const names = new Set<string>();
    for (const d of delegations) {
      if (d.status === "running") names.add(d.mappedAgent);
    }
    return names;
  }, [delegations]);

  // Sorted agents: running first, then alphabetical
  const sortedAgents = useMemo(() => {
    return [...agents].sort((a, b) => {
      const aRunning = runningAgentNames.has(a.name);
      const bRunning = runningAgentNames.has(b.name);
      if (aRunning && !bRunning) return -1;
      if (!aRunning && bRunning) return 1;
      return a.name.localeCompare(b.name);
    });
  }, [agents, runningAgentNames]);

  // Last 10 delegations, most recent first
  const recentDelegations = useMemo(
    () =>
      [...delegations]
        .sort((a, b) => b.startedAt - a.startedAt)
        .slice(0, 10),
    [delegations],
  );

  return (
    <div className="space-y-4 text-sm">
      {/* ---- Agent List ---- */}
      <div>
        <label className="text-zinc-400 text-xs uppercase tracking-wide">
          Agents
        </label>

        {sortedAgents.length === 0 ? (
          <p className="text-zinc-500 text-xs italic mt-1">
            No agents configured
          </p>
        ) : (
          <ul className="mt-1 space-y-1">
            {sortedAgents.map((agent) => {
              const isActive = runningAgentNames.has(agent.name);
              return (
                <li
                  key={agent.name}
                  className="flex items-center gap-2 px-2 py-1.5 rounded bg-zinc-800/60 hover:bg-zinc-800/80 transition-colors"
                >
                  <StatusDot active={isActive} />
                  <span className="text-xs text-zinc-200 truncate flex-1">
                    {agent.name}
                  </span>
                  {agent.custom && (
                    <span className="text-[10px] text-zinc-500">custom</span>
                  )}
                  {isActive && <DelegationBadge status="running" />}
                </li>
              );
            })}
          </ul>
        )}
      </div>

      {/* ---- Historique ---- */}
      {recentDelegations.length > 0 && (
        <div>
          <label className="text-zinc-400 text-xs uppercase tracking-wide">
            Historique
          </label>

          <div className="mt-1 space-y-1 max-h-64 overflow-y-auto">
            {recentDelegations.map((d) => {
              const isStale = d.status === "running" && now - d.startedAt > STALE_THRESHOLD_MS;
              return (
                <div
                  key={d.id}
                  className={`flex items-start gap-2 px-2 py-1.5 rounded transition-colors text-xs ${
                    d.status === "running"
                      ? isStale
                        ? "bg-zinc-800/30 opacity-50"
                        : "bg-blue-500/10 ring-1 ring-blue-500/30"
                      : "bg-zinc-800/30 hover:bg-zinc-800/50"
                  }`}
                >
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="font-medium text-zinc-200">
                        {capitalize(d.mappedAgent)}
                      </span>
                      <DelegationBadge status={d.status} stale={isStale} />
                      <span className="ml-auto text-zinc-500 flex-shrink-0">
                        {timeAgo(d.startedAt, now)}
                      </span>
                    </div>
                    <p className="text-zinc-400 mt-0.5 truncate">
                      {truncate(d.description, 80)}
                    </p>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* ---- Empty state ---- */}
      {sortedAgents.length === 0 && recentDelegations.length === 0 && (
        <p className="text-zinc-500 text-xs italic">
          No agent activity yet
        </p>
      )}
    </div>
  );
}
