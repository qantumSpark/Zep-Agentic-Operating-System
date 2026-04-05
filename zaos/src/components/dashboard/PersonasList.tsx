import { usePersonaStore } from "../../stores/personaStore";

export function PersonasList() {
  const personas = usePersonaStore((s) => s.personas);

  if (personas.length === 0) return null;

  return (
    <div>
      <label className="text-zinc-400 text-xs uppercase tracking-wide">Personas</label>
      <div className="mt-1 space-y-1">
        {personas.map((p) => (
          <div
            key={p.name}
            className="flex items-start gap-2 px-2 py-1.5 rounded bg-zinc-800/60"
          >
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2">
                <span className="text-zinc-200 text-xs font-medium">{p.name}</span>
                <span className="text-zinc-500 text-xs">{p.role}</span>
                {p.maps_to !== "none" && (
                  <span className="text-zinc-600 text-[10px] px-1 py-0.5 rounded bg-zinc-700/50">
                    {p.maps_to}
                  </span>
                )}
              </div>
              <p className="text-zinc-500 text-[11px] mt-0.5 truncate">{p.description}</p>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
