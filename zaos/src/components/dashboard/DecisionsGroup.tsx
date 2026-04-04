import { useMemo } from "react";
import { usePermissionStore } from "../../stores/permissionStore";
import { useProductStore } from "../../stores/productStore";
import { useMemoryStore } from "../../stores/memoryStore";
import { useWorkflowStore } from "../../stores/workflowStore";
import { StatusBadge } from "../common/StatusBadge";
import { GateControl } from "./GateControl";

const DONE_STATUSES = new Set(["done", "ok", "validé", "valide", "terminé", "termine", "pass"]);
const MAX_VISIBLE_CHECKS = 5;

export function DecisionsGroup() {
  const gateReady = useWorkflowStore((s) => s.gateReady);
  const pendingRequests = usePermissionStore((s) => s.pendingRequests);
  const acceptanceChecks = useProductStore(
    (s) => s.productContract?.acceptance_checks ?? null,
  );
  const blocages = useMemoryStore((s) => s.blocages);

  const pendingChecks = useMemo(() => {
    if (!acceptanceChecks?.checks) return [];
    return acceptanceChecks.checks.filter(
      (c) => !DONE_STATUSES.has(c.status.trim().toLowerCase()),
    );
  }, [acceptanceChecks]);

  const hasGate = gateReady;
  const hasPermissions = pendingRequests.length > 0;
  const hasChecks = pendingChecks.length > 0;
  const hasBlocages =
    !!blocages && blocages.trim().toLowerCase() !== "aucun" && blocages.trim() !== "";

  const isEmpty = !hasGate && !hasPermissions && !hasChecks && !hasBlocages;

  if (isEmpty) {
    return (
      <div className="space-y-3 text-sm">
        <p className="text-green-500/70 text-xs italic">
          Aucune decision en attente.
        </p>
      </div>
    );
  }

  const visibleChecks = pendingChecks.slice(0, MAX_VISIBLE_CHECKS);
  const remainingCount = pendingChecks.length - MAX_VISIBLE_CHECKS;

  return (
    <div className="space-y-3 text-sm">
      {/* Bloc 1 — Gate pending (only when gate is ready) */}
      {hasGate && <GateControl />}

      {/* Bloc 2 — Permissions pending */}
      {hasPermissions && (
        <div className="flex items-center gap-2 px-2 py-1.5 rounded bg-amber-900/20 border border-amber-700/40">
          <span className="text-amber-400 text-sm font-medium">
            {pendingRequests.length}
          </span>
          <span className="text-amber-300 text-xs">
            permission{pendingRequests.length > 1 ? "s" : ""} en attente
          </span>
        </div>
      )}

      {/* Bloc 3 — Acceptance checks non valides */}
      {hasChecks && (
        <div>
          <label className="text-zinc-400 text-xs uppercase tracking-wide">
            Checks en attente
          </label>
          <ul className="mt-1 space-y-1">
            {visibleChecks.map((check) => (
              <li
                key={check.number}
                className="flex items-center justify-between px-2 py-1 rounded bg-zinc-800/60"
              >
                <span className="text-zinc-200 text-xs truncate">
                  {check.check}
                </span>
                <StatusBadge status={check.status} />
              </li>
            ))}
          </ul>
          {remainingCount > 0 && (
            <p className="text-zinc-500 text-xs mt-1 px-2">
              et {remainingCount} autre{remainingCount > 1 ? "s" : ""}
            </p>
          )}
        </div>
      )}

      {/* Bloc 4 — Blocages */}
      {hasBlocages && (
        <div className="px-2 py-1.5 rounded bg-red-900/20 border border-red-700/40">
          <label className="text-red-400 text-xs uppercase tracking-wide">
            Blocage
          </label>
          <p className="text-red-300 text-xs mt-0.5">{blocages}</p>
        </div>
      )}
    </div>
  );
}
