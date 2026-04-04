/* React 19 JSX transform */
import { useMemo } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useProductStore } from "../../stores/productStore";
import { useScreenshotStore } from "../../stores/screenshotStore";
import { usePermissionStore } from "../../stores/permissionStore";
import { StatusBadge } from "../common/StatusBadge";
import { PolicyLogSummary } from "./PolicyLogSummary";

export function EvidenceGroup() {
  const contract = useProductStore((s) => s.productContract);
  const screenshots = useScreenshotStore((s) => s.screenshots);

  const checks = contract?.acceptance_checks ?? null;
  const readiness = contract?.release_readiness ?? null;

  const recentScreenshots = useMemo(
    () =>
      [...screenshots]
        .sort((a, b) => b.timestamp.localeCompare(a.timestamp))
        .slice(0, 3),
    [screenshots],
  );

  const hasValidation =
    (checks && (checks.checks.length > 0 || checks.manual_validations.length > 0)) ||
    (readiness &&
      (readiness.overall_state_summary ||
        readiness.checklist.length > 0 ||
        readiness.open_risks.length > 0));

  const hasScreenshots = recentScreenshots.length > 0;
  const hasDenies = usePermissionStore((s) => s.policyLog).some((e) => e.wasAutoDenied);

  const hasObservation = hasScreenshots || hasDenies;

  // Global empty state
  if (!hasValidation && !hasObservation) {
    return (
      <p className="text-zinc-500 text-xs italic">
        Aucune preuve disponible
      </p>
    );
  }

  return (
    <div className="space-y-3 text-sm">
      {/* ---- Sous-section 1 : Validation ---- */}
      {hasValidation && (
        <>
          <h4 className="text-zinc-300 text-xs font-semibold uppercase tracking-wide mb-2">
            Validation
          </h4>

          {/* Acceptance Checks */}
          {checks && checks.checks.length > 0 && (
            <div className="rounded border border-zinc-700 overflow-hidden">
              <table className="w-full text-xs">
                <thead>
                  <tr className="bg-zinc-800 text-zinc-400">
                    <th className="text-left px-2 py-1 font-medium">#</th>
                    <th className="text-left px-2 py-1 font-medium">Check</th>
                    <th className="text-right px-2 py-1 font-medium">Statut</th>
                    <th className="text-left px-2 py-1 font-medium">Notes</th>
                  </tr>
                </thead>
                <tbody>
                  {checks.checks.map((c) => (
                    <tr
                      key={c.number}
                      className="border-t border-zinc-700/50 hover:bg-zinc-800/40"
                    >
                      <td className="px-2 py-1 text-zinc-500">{c.number}</td>
                      <td className="px-2 py-1 text-zinc-200">{c.check}</td>
                      <td className="px-2 py-1 text-right">
                        <StatusBadge status={c.status} />
                      </td>
                      <td className="px-2 py-1 text-zinc-400 truncate max-w-[120px]">
                        {c.notes}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {/* Manual validations */}
          {checks && checks.manual_validations.length > 0 && (
            <div className="px-2 py-1.5 rounded bg-zinc-800/60">
              <span className="text-zinc-500 text-xs">Validations manuelles</span>
              <ul className="mt-0.5">
                {checks.manual_validations.map((v, i) => (
                  <li key={i} className="text-zinc-300 text-xs">
                    • {v}
                  </li>
                ))}
              </ul>
            </div>
          )}

          {/* Release Readiness */}
          {readiness && (
            <>
              {readiness.overall_state_summary && (
                <div className="px-2 py-1.5 rounded bg-zinc-800/60">
                  <p className="text-zinc-200 text-xs">
                    {readiness.overall_state_summary}
                  </p>
                </div>
              )}

              {readiness.checklist.length > 0 && (
                <div className="rounded border border-zinc-700 overflow-hidden">
                  <table className="w-full text-xs">
                    <thead>
                      <tr className="bg-zinc-800 text-zinc-400">
                        <th className="text-left px-2 py-1 font-medium">#</th>
                        <th className="text-left px-2 py-1 font-medium">Item</th>
                        <th className="text-center px-2 py-1 font-medium">
                          Bloquant
                        </th>
                        <th className="text-right px-2 py-1 font-medium">
                          Statut
                        </th>
                        <th className="text-left px-2 py-1 font-medium">Notes</th>
                      </tr>
                    </thead>
                    <tbody>
                      {readiness.checklist.map((item) => (
                        <tr
                          key={item.number}
                          className="border-t border-zinc-700/50 hover:bg-zinc-800/40"
                        >
                          <td className="px-2 py-1 text-zinc-500">{item.number}</td>
                          <td className="px-2 py-1 text-zinc-200">{item.item}</td>
                          <td className="px-2 py-1 text-center">
                            {item.blocking === "YES" ? (
                              <span className="text-red-400 text-[10px] font-semibold">
                                YES
                              </span>
                            ) : (
                              <span className="text-zinc-500 text-[10px]">no</span>
                            )}
                          </td>
                          <td className="px-2 py-1 text-right">
                            <StatusBadge status={item.status} />
                          </td>
                          <td className="px-2 py-1 text-zinc-400 truncate max-w-[120px]">
                            {item.notes}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              )}

              {readiness.open_risks.length > 0 && (
                <div className="px-2 py-1.5 rounded bg-zinc-800/60">
                  <span className="text-zinc-500 text-xs">Risques ouverts</span>
                  <ul className="mt-0.5">
                    {readiness.open_risks.map((r, i) => (
                      <li key={i} className="text-amber-400/80 text-xs">
                        • {r}
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </>
          )}
        </>
      )}

      {/* ---- Sous-section 2 : Observation ---- */}
      {hasObservation && (
        <h4 className="text-zinc-500 text-xs font-medium uppercase tracking-wide mb-2 mt-4">
          Observation
        </h4>
      )}

      {/* Screenshots recents (3 derniers) */}
      {hasScreenshots && (
        <div>
          <label className="text-zinc-500 text-xs uppercase tracking-wide">
            Screenshots
          </label>
          <div className="mt-1 grid grid-cols-3 gap-2">
            {recentScreenshots.map((s) => (
              <div
                key={s.id}
                className="aspect-video rounded overflow-hidden bg-zinc-800 border border-zinc-700/50"
              >
                <img
                  src={convertFileSrc(s.path)}
                  alt={s.filename}
                  className="w-full h-full object-cover"
                />
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Policy Log Summary */}
      <PolicyLogSummary />
    </div>
  );
}
