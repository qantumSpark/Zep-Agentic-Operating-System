import { useState, useCallback, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useScreenshotStore } from "../../stores/screenshotStore";
import type { Screenshot } from "../../types/screenshots";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function formatTimestamp(iso: string): string {
  const d = new Date(iso);
  return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

function sourceLabel(source: Screenshot["source"]): string {
  switch (source) {
    case "cliMcp":
      return "MCP";
    case "filesystem":
      return "File";
    case "manual":
      return "Manual";
    default:
      return String(source);
  }
}

// ---------------------------------------------------------------------------
// Thumbnail card
// ---------------------------------------------------------------------------

interface ThumbnailProps {
  screenshot: Screenshot;
  onSelect: (id: string) => void;
  onDelete: (id: string) => void;
}

function Thumbnail({ screenshot, onSelect, onDelete }: ThumbnailProps) {
  const [imgError, setImgError] = useState(false);
  const imgSrc = convertFileSrc(screenshot.path);

  return (
    <div
      className="relative aspect-video bg-zinc-800 rounded border border-zinc-700 overflow-hidden cursor-pointer hover:border-blue-500 transition-colors group"
      onClick={() => onSelect(screenshot.id)}
    >
      {!imgError ? (
        <img
          src={imgSrc}
          alt={screenshot.metadata.label ?? screenshot.filename}
          className="w-full h-full object-cover"
          onError={() => setImgError(true)}
        />
      ) : (
        <div className="w-full h-full flex items-center justify-center text-zinc-600 text-xs">
          Failed to load
        </div>
      )}

      {/* Overlay with metadata on hover */}
      <div className="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/80 to-transparent px-2 py-1.5 opacity-0 group-hover:opacity-100 transition-opacity">
        <p className="text-[10px] text-zinc-300 truncate">
          {screenshot.metadata.label ?? screenshot.filename}
        </p>
        <div className="flex items-center gap-2 text-[10px] text-zinc-500">
          <span>{formatTimestamp(screenshot.timestamp)}</span>
          <span>{sourceLabel(screenshot.source)}</span>
        </div>
      </div>

      {/* Delete button on hover */}
      <button
        className="absolute top-1 right-1 w-5 h-5 flex items-center justify-center rounded bg-red-600/80 hover:bg-red-500 text-white text-xs leading-none opacity-0 group-hover:opacity-100 transition-opacity"
        onClick={(e) => {
          e.stopPropagation();
          onDelete(screenshot.id);
        }}
        title="Delete screenshot"
      >
        X
      </button>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Zoom modal
// ---------------------------------------------------------------------------

interface ZoomModalProps {
  screenshot: Screenshot;
  onClose: () => void;
}

function ZoomModal({ screenshot, onClose }: ZoomModalProps) {
  const imgSrc = convertFileSrc(screenshot.path);

  useEffect(() => {
    function handleKey(e: KeyboardEvent) {
      if (e.key === "Escape") onClose();
    }
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [onClose]);

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/80"
      onClick={onClose}
    >
      {/* Close button */}
      <button
        className="absolute top-4 right-4 w-8 h-8 flex items-center justify-center rounded bg-zinc-700/80 hover:bg-zinc-600 text-zinc-300 text-sm transition-colors"
        onClick={onClose}
        title="Close"
      >
        X
      </button>

      {/* Image */}
      <img
        src={imgSrc}
        alt={screenshot.metadata.label ?? screenshot.filename}
        className="max-w-[90vw] max-h-[90vh] object-contain rounded shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      />

      {/* Caption */}
      <div
        className="absolute bottom-4 left-1/2 -translate-x-1/2 bg-zinc-900/90 px-4 py-2 rounded text-xs text-zinc-300 max-w-md text-center"
        onClick={(e) => e.stopPropagation()}
      >
        <span className="font-medium">{screenshot.filename}</span>
        <span className="text-zinc-500 ml-2">
          {formatTimestamp(screenshot.timestamp)}
        </span>
        {screenshot.metadata.label && (
          <span className="text-zinc-400 ml-2">
            {screenshot.metadata.label}
          </span>
        )}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// ScreenshotGallery
// ---------------------------------------------------------------------------
// NOTE: ComparisonView (./ComparisonView.tsx) provides before/after comparison
// with side-by-side and slider modes. It can be integrated here once iterations
// are fully wired — e.g. opening it with the first and last screenshot of an
// iteration to show visual progress.

export function ScreenshotGallery() {
  const screenshots = useScreenshotStore((s) => s.screenshots);
  const selectedScreenshotId = useScreenshotStore(
    (s) => s.selectedScreenshotId,
  );
  const selectScreenshot = useScreenshotStore((s) => s.selectScreenshot);
  const removeScreenshot = useScreenshotStore((s) => s.removeScreenshot);

  const [capturing, setCapturing] = useState(false);

  const selectedScreenshot = useMemo(
    () => screenshots.find((s) => s.id === selectedScreenshotId),
    [screenshots, selectedScreenshotId],
  );

  // ---- Actions ----

  const handleSelect = useCallback(
    (id: string) => {
      selectScreenshot(id);
    },
    [selectScreenshot],
  );

  const handleClose = useCallback(() => {
    selectScreenshot(null);
  }, [selectScreenshot]);

  const handleDelete = useCallback(
    async (id: string) => {
      try {
        await invoke("delete_screenshot", { id });
      } catch (err) {
        console.error("delete_screenshot failed:", err);
      }
      removeScreenshot(id);
    },
    [removeScreenshot],
  );

  const handleCapture = useCallback(async () => {
    setCapturing(true);
    try {
      await invoke("request_capture");
    } catch (err) {
      console.error("request_capture failed:", err);
    } finally {
      setCapturing(false);
    }
  }, []);

  // ---- Render ----

  return (
    <div className="space-y-3">
      {/* ---- Header buttons ---- */}
      <div className="flex items-center gap-2">
        <button
          className="flex items-center gap-1 px-2 py-1 rounded text-xs bg-zinc-700/60 hover:bg-zinc-700 text-zinc-300 transition-colors"
          onClick={handleCapture}
          disabled={capturing}
          title="Capture screenshot"
        >
          {capturing ? "Capturing..." : "Capture"}
        </button>

        <span className="text-[10px] text-zinc-500 ml-auto">
          {screenshots.length > 0 && `${screenshots.length} screenshot${screenshots.length !== 1 ? "s" : ""}`}
        </span>
      </div>

      {/* ---- Empty state ---- */}
      {screenshots.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-8 text-zinc-500">
          <span className="text-2xl mb-2 opacity-50">&#128444;</span>
          <p className="text-sm italic">No screenshots captured yet</p>
          <p className="text-xs mt-1 text-zinc-600">
            Drop images in .screenshots/ or click Capture
          </p>
        </div>
      ) : (
        /* ---- Grid ---- */
        <div className="grid grid-cols-2 gap-2">
          {screenshots.map((screenshot) => (
            <Thumbnail
              key={screenshot.id}
              screenshot={screenshot}
              onSelect={handleSelect}
              onDelete={handleDelete}
            />
          ))}
        </div>
      )}

      {/* ---- Zoom modal ---- */}
      {selectedScreenshot && (
        <ZoomModal screenshot={selectedScreenshot} onClose={handleClose} />
      )}
    </div>
  );
}
