import { useMemo } from "react";
import { useSessionStore } from "../../stores/sessionStore";
import { formatDuration } from "../../utils/formatDuration";

function formatTokenCount(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
  return String(n);
}

// ---------------------------------------------------------------------------
// SessionMetrics
// ---------------------------------------------------------------------------

export function SessionMetrics() {
  const phaseTokens = useSessionStore((s) => s.phaseTokens);
  const agentTimings = useSessionStore((s) => s.agentTimings);

  const sortedPhases = useMemo(() => {
    return Object.entries(phaseTokens)
      .map(([phase, tokens]) => ({
        phase,
        input: tokens.input,
        output: tokens.output,
        total: tokens.input + tokens.output,
      }))
      .sort((a, b) => b.total - a.total);
  }, [phaseTokens]);

  const sortedAgents = useMemo(() => {
    return Object.entries(agentTimings)
      .map(([agent, time]) => ({ agent, time }))
      .sort((a, b) => b.time - a.time);
  }, [agentTimings]);

  if (sortedPhases.length === 0 && sortedAgents.length === 0) {
    return (
      <p className="text-zinc-500 text-xs italic">
        Aucune metrique pour cette session
      </p>
    );
  }

  return (
    <div className="space-y-4 text-sm">
      {/* ---- Tokens par phase ---- */}
      {sortedPhases.length > 0 && (
        <div>
          <label className="text-zinc-400 text-xs uppercase tracking-wide">
            Tokens par phase
          </label>
          <table className="w-full mt-1 text-xs">
            <thead>
              <tr className="text-zinc-500 text-left">
                <th className="py-1 pr-2 font-medium">Phase</th>
                <th className="py-1 pr-2 font-medium text-right">Input</th>
                <th className="py-1 font-medium text-right">Output</th>
              </tr>
            </thead>
            <tbody>
              {sortedPhases.map(({ phase, input, output }) => (
                <tr
                  key={phase}
                  className="border-t border-zinc-800/50 hover:bg-zinc-800/30 transition-colors"
                >
                  <td className="py-1 pr-2 text-zinc-200 truncate max-w-[120px]">
                    {phase}
                  </td>
                  <td className="py-1 pr-2 text-zinc-400 text-right tabular-nums">
                    {formatTokenCount(input)}
                  </td>
                  <td className="py-1 text-zinc-400 text-right tabular-nums">
                    {formatTokenCount(output)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* ---- Temps par agent ---- */}
      {sortedAgents.length > 0 && (
        <div>
          <label className="text-zinc-400 text-xs uppercase tracking-wide">
            Temps par agent
          </label>
          <table className="w-full mt-1 text-xs">
            <thead>
              <tr className="text-zinc-500 text-left">
                <th className="py-1 pr-2 font-medium">Agent</th>
                <th className="py-1 font-medium text-right">Temps total</th>
              </tr>
            </thead>
            <tbody>
              {sortedAgents.map(({ agent, time }) => (
                <tr
                  key={agent}
                  className="border-t border-zinc-800/50 hover:bg-zinc-800/30 transition-colors"
                >
                  <td className="py-1 pr-2 text-zinc-200 truncate max-w-[160px] capitalize">
                    {agent}
                  </td>
                  <td className="py-1 text-zinc-400 text-right tabular-nums">
                    {formatDuration(time)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
