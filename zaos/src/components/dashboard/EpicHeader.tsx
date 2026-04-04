import { useMemoryStore } from "../../stores/memoryStore";

export function EpicHeader() {
  const currentEpic = useMemoryStore((s) => s.currentEpic);

  if (!currentEpic) return null;

  return (
    <div>
      <label className="text-zinc-400 text-xs uppercase tracking-wide">Epic</label>
      <p className="text-zinc-100 font-medium mt-0.5">{currentEpic.name}</p>
      {currentEpic.objective && (
        <p className="text-zinc-400 text-xs mt-0.5">{currentEpic.objective}</p>
      )}
    </div>
  );
}
