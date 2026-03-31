import React, { useMemo } from "react";
import { useScreenshotStore } from "../../stores/screenshotStore";
import type { IterationStatus, Iteration } from "../../types/screenshots";

// ---------------------------------------------------------------------------
// Status badge color map
// ---------------------------------------------------------------------------

const STATUS_COLORS: Record<IterationStatus, string> = {
  capturing: "bg-blue-500/20 text-blue-400",
  evaluating: "bg-yellow-500/20 text-yellow-400",
  fixing: "bg-orange-500/20 text-orange-400",
  comparing: "bg-purple-500/20 text-purple-400",
  done: "bg-green-500/20 text-green-400",
};

const STEPS: IterationStatus[] = [
  "capturing",
  "evaluating",
  "fixing",
  "comparing",
  "done",
];

// ---------------------------------------------------------------------------
// StatusBadge
// ---------------------------------------------------------------------------

function StatusBadge({ status }: { status: IterationStatus }) {
  return (
    <span
      className={`inline-block px-1.5 py-0.5 rounded text-[10px] font-semibold uppercase leading-none ${STATUS_COLORS[status]}`}
    >
      {status}
    </span>
  );
}

// ---------------------------------------------------------------------------
// StatusFlow — horizontal step visualization
// ---------------------------------------------------------------------------

function StatusFlow({ current }: { current: IterationStatus }) {
  return (
    <div className="flex items-center gap-1 flex-wrap">
      {STEPS.map((step, i) => (
        <React.Fragment key={step}>
          <span
            className={`text-xs px-2 py-0.5 rounded ${
              step === current ? STATUS_COLORS[step] : "text-zinc-600"
            }`}
          >
            {step}
          </span>
          {i < STEPS.length - 1 && (
            <span className="text-zinc-600 text-xs">&rarr;</span>
          )}
        </React.Fragment>
      ))}
    </div>
  );
}

// ---------------------------------------------------------------------------
// IterationRow — compact display of a single iteration
// ---------------------------------------------------------------------------

function IterationRow({ iteration }: { iteration: Iteration }) {
  return (
    <div className="flex items-center justify-between gap-2 px-2 py-1 rounded bg-zinc-800/60">
      <div className="flex items-center gap-2 min-w-0">
        <StatusBadge status={iteration.status} />
        <span className="text-zinc-200 text-xs truncate">
          {iteration.targetDescription ?? iteration.id}
        </span>
      </div>
      <span className="text-zinc-500 text-[10px] whitespace-nowrap">
        {iteration.screenshots.length} shot{iteration.screenshots.length !== 1 ? "s" : ""}
      </span>
    </div>
  );
}

// ---------------------------------------------------------------------------
// IterationTracker
// ---------------------------------------------------------------------------

export function IterationTracker() {
  const iterations = useScreenshotStore((s) => s.iterations);
  const activeIterationId = useScreenshotStore((s) => s.activeIterationId);

  const activeIteration = useMemo(
    () => iterations.find((i) => i.id === activeIterationId) ?? null,
    [iterations, activeIterationId],
  );

  const recentIterations = useMemo(
    () =>
      iterations
        .filter((i) => i.id !== activeIterationId)
        .sort(
          (a, b) =>
            new Date(b.startedAt).getTime() - new Date(a.startedAt).getTime(),
        )
        .slice(0, 5),
    [iterations, activeIterationId],
  );

  // ---- Empty state ----
  if (iterations.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-8 text-zinc-500">
        <span className="text-2xl mb-2 opacity-50">🔄</span>
        <p className="text-sm italic">No iteration loop active</p>
      </div>
    );
  }

  return (
    <div className="space-y-4 text-sm">
      {/* ---- Active Iteration ---- */}
      {activeIteration && (
        <div className="space-y-2">
          <label className="text-zinc-400 text-xs uppercase tracking-wide">
            Active Iteration
          </label>

          {/* Status flow visualization */}
          <StatusFlow current={activeIteration.status} />

          {/* Details */}
          <div className="px-2 py-2 rounded bg-zinc-800/60 space-y-1">
            <div className="flex items-center gap-2">
              <StatusBadge status={activeIteration.status} />
              {activeIteration.targetDescription && (
                <span className="text-zinc-200 text-xs truncate">
                  {activeIteration.targetDescription}
                </span>
              )}
            </div>

            <div className="flex gap-3 text-[10px] text-zinc-500">
              <span>
                {activeIteration.screenshots.length} screenshot
                {activeIteration.screenshots.length !== 1 ? "s" : ""}
              </span>
              <span>
                {activeIteration.findings.length} finding
                {activeIteration.findings.length !== 1 ? "s" : ""}
              </span>
            </div>

            {/* Findings list */}
            {activeIteration.findings.length > 0 && (
              <ul className="mt-1 space-y-0.5">
                {activeIteration.findings.map((finding, idx) => (
                  <li
                    key={idx}
                    className="text-[11px] text-zinc-300 pl-2 border-l border-zinc-700"
                  >
                    {finding}
                  </li>
                ))}
              </ul>
            )}
          </div>
        </div>
      )}

      {/* ---- Iteration History ---- */}
      {recentIterations.length > 0 && (
        <div>
          <label className="text-zinc-400 text-xs uppercase tracking-wide">
            Recent Iterations
          </label>
          <ul className="mt-1 space-y-1">
            {recentIterations.map((iter) => (
              <li key={iter.id}>
                <IterationRow iteration={iter} />
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
