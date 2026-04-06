/* React 19 JSX transform */
import { useRuntimeStore } from "../../stores/runtimeStore";

/**
 * Animated "thinking" indicator shown when the active runtime is thinking
 */
export function ThinkingIndicator() {
  const runtimeName = useRuntimeStore((s) => s.info?.name) ?? "Agent";

  return (
    <div className="flex items-center gap-2 text-zinc-400">
      <span className="animate-pulse text-base" role="img" aria-label="brain">&#x1F9E0;</span>
      <span className="text-sm italic">{runtimeName} is thinking...</span>
      <div className="flex gap-1">
        <div className="w-1 h-1 bg-blue-500 rounded-full animate-bounce" style={{ animationDelay: "0ms" }} />
        <div className="w-1 h-1 bg-blue-500 rounded-full animate-bounce" style={{ animationDelay: "150ms" }} />
        <div className="w-1 h-1 bg-blue-500 rounded-full animate-bounce" style={{ animationDelay: "300ms" }} />
      </div>
    </div>
  );
}
