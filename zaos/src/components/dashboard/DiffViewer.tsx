import { useState, useMemo } from "react";
import { useDiffStore, type DiffEntry } from "../../stores/diffStore";

function computeLineDiff(
  before: string,
  after: string,
): Array<{ type: "added" | "removed" | "unchanged"; text: string }> {
  const beforeLines = before.split("\n");
  const afterLines = after.split("\n");
  const result: Array<{
    type: "added" | "removed" | "unchanged";
    text: string;
  }> = [];

  const maxLen = Math.max(beforeLines.length, afterLines.length);

  for (let i = 0; i < maxLen; i++) {
    const bLine = i < beforeLines.length ? beforeLines[i] : undefined;
    const aLine = i < afterLines.length ? afterLines[i] : undefined;

    if (bLine === aLine) {
      result.push({ type: "unchanged", text: bLine ?? "" });
    } else {
      if (bLine !== undefined) {
        result.push({ type: "removed", text: bLine });
      }
      if (aLine !== undefined) {
        result.push({ type: "added", text: aLine });
      }
    }
  }

  return result;
}

function formatTimestamp(iso: string): string {
  const d = new Date(iso);
  return d.toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

function DiffEntryCard({
  entry,
  index,
  isActive,
  onSelect,
}: {
  entry: DiffEntry;
  index: number;
  isActive: boolean;
  onSelect: () => void;
}) {
  const lines = useMemo(
    () => (isActive ? computeLineDiff(entry.before, entry.after) : []),
    [isActive, entry.before, entry.after],
  );

  return (
    <div className="border border-zinc-700 rounded-md overflow-hidden">
      <button
        onClick={onSelect}
        className={`w-full flex items-center justify-between px-3 py-2 text-left text-xs transition-colors ${
          isActive
            ? "bg-zinc-700 text-zinc-100"
            : "bg-zinc-800 text-zinc-400 hover:bg-zinc-700/60 hover:text-zinc-300"
        }`}
      >
        <span className="font-mono truncate flex-1">{entry.path}</span>
        <span className="ml-2 shrink-0 text-zinc-500">
          {formatTimestamp(entry.timestamp)}
        </span>
      </button>

      {entry.description && isActive && (
        <div className="px-3 py-1.5 bg-zinc-800 text-zinc-400 text-xs border-b border-zinc-700">
          {entry.description}
        </div>
      )}

      {isActive && (
        <div className="bg-zinc-900 overflow-x-auto max-h-80 overflow-y-auto">
          <table className="w-full text-xs font-mono border-collapse">
            <tbody>
              {lines.map((line, idx) => (
                <tr
                  key={idx}
                  className={
                    line.type === "added"
                      ? "bg-green-900/30"
                      : line.type === "removed"
                        ? "bg-red-900/30"
                        : ""
                  }
                >
                  <td className="w-6 px-2 text-right select-none text-zinc-600 border-r border-zinc-700">
                    {line.type === "added"
                      ? "+"
                      : line.type === "removed"
                        ? "-"
                        : " "}
                  </td>
                  <td className="px-2 py-0.5 whitespace-pre">
                    <span
                      className={
                        line.type === "added"
                          ? "text-green-400"
                          : line.type === "removed"
                            ? "text-red-400"
                            : "text-zinc-400"
                      }
                    >
                      {line.text}
                    </span>
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

export function DiffViewer() {
  const diffs = useDiffStore((s) => s.diffs);
  const [activeIndex, setActiveIndex] = useState<number | null>(null);

  if (diffs.length === 0) {
    return (
      <div className="px-3 py-4 text-center text-xs text-zinc-500">
        No diffs received yet.
      </div>
    );
  }

  // Most recent first
  const reversed = [...diffs].reverse();

  return (
    <div className="flex flex-col gap-2 p-2">
      <div className="text-xs text-zinc-500 px-1">
        {diffs.length} diff{diffs.length !== 1 ? "s" : ""}
      </div>
      {reversed.map((entry, i) => {
        const originalIndex = diffs.length - 1 - i;
        return (
          <DiffEntryCard
            key={`${entry.path}-${entry.timestamp}`}
            entry={entry}
            index={originalIndex}
            isActive={activeIndex === originalIndex}
            onSelect={() =>
              setActiveIndex((prev) =>
                prev === originalIndex ? null : originalIndex,
              )
            }
          />
        );
      })}
    </div>
  );
}
