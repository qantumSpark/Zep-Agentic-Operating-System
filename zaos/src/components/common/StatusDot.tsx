export function StatusDot({ active }: { active: boolean }) {
  return (
    <span
      className={`inline-block w-2 h-2 rounded-full flex-shrink-0 ${
        active ? "bg-green-400" : "bg-zinc-500"
      }`}
    />
  );
}
