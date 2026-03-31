export function Toggle({
  enabled,
  onToggle,
  label,
}: {
  enabled: boolean;
  onToggle: () => void;
  label?: boolean;
}) {
  return (
    <button
      onClick={onToggle}
      className="flex items-center gap-2 cursor-pointer"
    >
      {label !== false && (
        <span className="text-xs text-zinc-400">
          {enabled ? "Enabled" : "Disabled"}
        </span>
      )}
      <div
        className={`relative w-8 h-4 rounded-full transition-colors ${
          enabled ? "bg-green-600" : "bg-zinc-600"
        }`}
      >
        <div
          className={`absolute top-0.5 w-3 h-3 rounded-full bg-white transition-transform ${
            enabled ? "translate-x-4" : "translate-x-0.5"
          }`}
        />
      </div>
    </button>
  );
}
