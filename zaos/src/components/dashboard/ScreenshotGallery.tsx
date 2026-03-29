import React from "react";
import { Image } from "lucide-react";

/**
 * Screenshot gallery - stub/placeholder
 * Shows thumbnails of captured screenshots
 */
export function ScreenshotGallery() {
  // TODO: Connect to actual screenshot data
  const screenshots: { id: string; timestamp: number; src?: string }[] = [];

  return (
    <div>
      {screenshots.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-8 text-zinc-500">
          <Image size={32} className="mb-2 opacity-50" />
          <p className="text-sm italic">No screenshots captured yet</p>
        </div>
      ) : (
        <div className="grid grid-cols-2 gap-2">
          {screenshots.map((screenshot) => (
            <div
              key={screenshot.id}
              className="aspect-square bg-zinc-800 rounded border border-zinc-700 overflow-hidden cursor-pointer hover:border-blue-500 transition-colors"
            >
              {screenshot.src ? (
                <img
                  src={screenshot.src}
                  alt="Screenshot"
                  className="w-full h-full object-cover"
                />
              ) : (
                <div className="w-full h-full flex items-center justify-center text-zinc-600">
                  <Image size={20} />
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
