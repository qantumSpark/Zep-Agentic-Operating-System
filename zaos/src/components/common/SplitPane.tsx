import React, { useRef, useEffect, useState } from "react";

interface SplitPaneProps {
  left: React.ReactNode;
  right: React.ReactNode;
  initialSplit?: number; // percentage, default 50
  minSize?: number; // minimum pixel width for each pane
}

/**
 * Resizable split pane component with horizontal divider
 */
export function SplitPane({
  left,
  right,
  initialSplit = 50,
  minSize = 200,
}: SplitPaneProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [split, setSplit] = useState(initialSplit);
  const [isDragging, setIsDragging] = useState(false);

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!isDragging || !containerRef.current) return;

      const container = containerRef.current;
      const rect = container.getBoundingClientRect();
      const newSplit = ((e.clientX - rect.left) / rect.width) * 100;

      // Constrain between minSize and (100 - minSize)
      const minPercent = (minSize / rect.width) * 100;
      const maxPercent = 100 - minPercent;

      if (newSplit >= minPercent && newSplit <= maxPercent) {
        setSplit(newSplit);
      }
    };

    const handleMouseUp = () => {
      setIsDragging(false);
    };

    if (isDragging) {
      document.addEventListener("mousemove", handleMouseMove);
      document.addEventListener("mouseup", handleMouseUp);

      return () => {
        document.removeEventListener("mousemove", handleMouseMove);
        document.removeEventListener("mouseup", handleMouseUp);
      };
    }
  }, [isDragging, minSize]);

  return (
    <div
      ref={containerRef}
      className="flex h-full w-full overflow-hidden bg-zinc-900"
    >
      {/* Left pane */}
      <div style={{ width: `${split}%` }} className="overflow-auto border-r border-zinc-700">
        {left}
      </div>

      {/* Divider */}
      <div
        onMouseDown={() => setIsDragging(true)}
        className="w-1 cursor-col-resize bg-zinc-700 hover:bg-blue-500 transition-colors"
      />

      {/* Right pane */}
      <div style={{ width: `${100 - split}%` }} className="overflow-auto">
        {right}
      </div>
    </div>
  );
}
