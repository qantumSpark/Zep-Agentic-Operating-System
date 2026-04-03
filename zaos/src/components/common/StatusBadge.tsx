/* React 19 JSX transform */

export function statusColor(status: string): string {
  const s = status.toUpperCase().trim();
  if (s === "TERMINE" || s === "TERMINÉ" || s === "VALIDATED" || s === "DONE")
    return "bg-green-700/60 text-green-300";
  if (s === "EN COURS" || s === "EN_COURS" || s === "IN_PROGRESS")
    return "bg-blue-700/60 text-blue-300";
  if (s.startsWith("BLOCK") || s.startsWith("BLOQU"))
    return "bg-red-700/60 text-red-300";
  // Product contract statuses
  if (s === "PASS") return "bg-green-700/60 text-green-300";
  if (s === "FAIL") return "bg-red-700/60 text-red-300";
  if (s === "N/A") return "bg-zinc-500/40 text-zinc-400";
  if (s === "TODO") return "bg-amber-700/50 text-amber-300";
  // default: not started / A FAIRE / pending
  return "bg-zinc-600/60 text-zinc-300";
}

export function StatusBadge({ status }: { status: string }) {
  return (
    <span
      className={`inline-block px-1.5 py-0.5 rounded text-[10px] font-semibold uppercase leading-none ${statusColor(status)}`}
    >
      {status}
    </span>
  );
}
