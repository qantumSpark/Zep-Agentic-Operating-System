import { useState, useCallback, useRef, useEffect } from "react";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface ComparisonViewProps {
  beforeSrc: string; // asset:// URL
  afterSrc: string; // asset:// URL
  beforeLabel?: string;
  afterLabel?: string;
  onClose: () => void;
}

type ViewMode = "side-by-side" | "slider";

// ---------------------------------------------------------------------------
// ComparisonView
// ---------------------------------------------------------------------------

export function ComparisonView({
  beforeSrc,
  afterSrc,
  beforeLabel = "Before",
  afterLabel = "After",
  onClose,
}: ComparisonViewProps) {
  const [mode, setMode] = useState<ViewMode>("side-by-side");
  const [sliderPos, setSliderPos] = useState(50);
  const [isDragging, setIsDragging] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  // ---- Keyboard: Escape closes ----
  useEffect(() => {
    function handleKey(e: KeyboardEvent) {
      if (e.key === "Escape") onClose();
    }
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [onClose]);

  // ---- Global mouse-up to end drag ----
  useEffect(() => {
    if (!isDragging) return;

    function handleUp() {
      setIsDragging(false);
    }
    window.addEventListener("mouseup", handleUp);
    return () => window.removeEventListener("mouseup", handleUp);
  }, [isDragging]);

  // ---- Slider drag handler ----
  const handleMouseMove = useCallback(
    (e: React.MouseEvent) => {
      if (!isDragging || !containerRef.current) return;
      const rect = containerRef.current.getBoundingClientRect();
      const x = ((e.clientX - rect.left) / rect.width) * 100;
      setSliderPos(Math.max(0, Math.min(100, x)));
    },
    [isDragging],
  );

  // ---- Toggle mode ----
  const toggleMode = useCallback(() => {
    setMode((prev) => (prev === "side-by-side" ? "slider" : "side-by-side"));
    setSliderPos(50);
  }, []);

  return (
    <div
      className="fixed inset-0 z-50 flex flex-col items-center justify-center bg-black/90"
      onClick={onClose}
    >
      {/* ---- Top bar ---- */}
      <div
        className="absolute top-4 left-1/2 -translate-x-1/2 flex items-center gap-3 z-10"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Mode toggle */}
        <button
          className="px-3 py-1.5 rounded text-xs font-medium bg-zinc-700/80 hover:bg-zinc-600 text-zinc-300 transition-colors"
          onClick={toggleMode}
        >
          {mode === "side-by-side" ? "Switch to Slider" : "Switch to Side-by-Side"}
        </button>
      </div>

      {/* ---- Close button ---- */}
      <button
        className="absolute top-4 right-4 w-8 h-8 flex items-center justify-center rounded bg-zinc-700/80 hover:bg-zinc-600 text-zinc-300 text-sm transition-colors z-10"
        onClick={onClose}
        title="Close comparison"
      >
        X
      </button>

      {/* ---- Content area ---- */}
      <div
        className="w-[90vw] h-[80vh] flex items-center justify-center"
        onClick={(e) => e.stopPropagation()}
      >
        {mode === "side-by-side" ? (
          <SideBySide
            beforeSrc={beforeSrc}
            afterSrc={afterSrc}
            beforeLabel={beforeLabel}
            afterLabel={afterLabel}
          />
        ) : (
          <SliderView
            containerRef={containerRef}
            beforeSrc={beforeSrc}
            afterSrc={afterSrc}
            beforeLabel={beforeLabel}
            afterLabel={afterLabel}
            sliderPos={sliderPos}
            isDragging={isDragging}
            onMouseMove={handleMouseMove}
            onDragStart={() => setIsDragging(true)}
          />
        )}
      </div>

      {/* ---- Bottom caption ---- */}
      <div
        className="absolute bottom-4 left-1/2 -translate-x-1/2 bg-zinc-900/90 px-4 py-2 rounded text-xs text-zinc-400 text-center"
        onClick={(e) => e.stopPropagation()}
      >
        {mode === "side-by-side"
          ? "Side-by-side comparison"
          : "Drag the slider to compare"}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Side-by-side sub-component
// ---------------------------------------------------------------------------

interface SideBySideProps {
  beforeSrc: string;
  afterSrc: string;
  beforeLabel: string;
  afterLabel: string;
}

function SideBySide({
  beforeSrc,
  afterSrc,
  beforeLabel,
  afterLabel,
}: SideBySideProps) {
  return (
    <div className="flex gap-4 w-full h-full">
      {/* Before */}
      <div className="flex-1 flex flex-col min-w-0">
        <span className="text-xs text-zinc-400 font-medium mb-2 text-center">
          {beforeLabel}
        </span>
        <div className="flex-1 flex items-center justify-center bg-zinc-900/50 rounded border border-zinc-700/50 overflow-hidden">
          <img
            src={beforeSrc}
            alt={beforeLabel}
            className="max-w-full max-h-full object-contain"
          />
        </div>
      </div>

      {/* Divider */}
      <div className="w-px bg-zinc-700 self-stretch" />

      {/* After */}
      <div className="flex-1 flex flex-col min-w-0">
        <span className="text-xs text-zinc-400 font-medium mb-2 text-center">
          {afterLabel}
        </span>
        <div className="flex-1 flex items-center justify-center bg-zinc-900/50 rounded border border-zinc-700/50 overflow-hidden">
          <img
            src={afterSrc}
            alt={afterLabel}
            className="max-w-full max-h-full object-contain"
          />
        </div>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Slider sub-component
// ---------------------------------------------------------------------------

interface SliderViewProps {
  containerRef: React.RefObject<HTMLDivElement | null>;
  beforeSrc: string;
  afterSrc: string;
  beforeLabel: string;
  afterLabel: string;
  sliderPos: number;
  isDragging: boolean;
  onMouseMove: (e: React.MouseEvent) => void;
  onDragStart: () => void;
}

function SliderView({
  containerRef,
  beforeSrc,
  afterSrc,
  beforeLabel,
  afterLabel,
  sliderPos,
  isDragging,
  onMouseMove,
  onDragStart,
}: SliderViewProps) {
  return (
    <div
      ref={containerRef}
      className="relative w-full h-full bg-zinc-900/50 rounded border border-zinc-700/50 overflow-hidden select-none"
      onMouseMove={onMouseMove}
      style={{ cursor: isDragging ? "ew-resize" : undefined }}
    >
      {/* After image (full, behind) */}
      <img
        src={afterSrc}
        alt={afterLabel}
        className="absolute inset-0 w-full h-full object-contain"
        draggable={false}
      />

      {/* Before image (clipped by slider) */}
      <div
        className="absolute inset-0"
        style={{ clipPath: `inset(0 ${100 - sliderPos}% 0 0)` }}
      >
        <img
          src={beforeSrc}
          alt={beforeLabel}
          className="w-full h-full object-contain"
          draggable={false}
        />
      </div>

      {/* Labels */}
      <span className="absolute top-3 left-3 px-2 py-0.5 rounded bg-black/60 text-[10px] text-zinc-300 font-medium pointer-events-none">
        {beforeLabel}
      </span>
      <span className="absolute top-3 right-3 px-2 py-0.5 rounded bg-black/60 text-[10px] text-zinc-300 font-medium pointer-events-none">
        {afterLabel}
      </span>

      {/* Slider line + handle */}
      <div
        className="absolute top-0 bottom-0 w-0.5 bg-white/80 cursor-ew-resize z-10"
        style={{ left: `${sliderPos}%` }}
        onMouseDown={(e) => {
          e.preventDefault();
          onDragStart();
        }}
      >
        {/* Drag handle */}
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-8 h-8 bg-white rounded-full shadow-lg flex items-center justify-center cursor-ew-resize">
          <svg
            width="14"
            height="14"
            viewBox="0 0 14 14"
            fill="none"
            className="text-zinc-700"
          >
            <path
              d="M4 3L1 7L4 11M10 3L13 7L10 11"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        </div>
      </div>
    </div>
  );
}
