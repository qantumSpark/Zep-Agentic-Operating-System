import React from "react";

export function ScreenshotGallery() {
  const screenshots: { id: string; timestamp: number; src?: string }[] = [];

  return (
    <div>
      {screenshots.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-8 text-zinc-500">
          <span className="text-2xl mb-2 opacity-50">🖼</span>
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
                <img src={screenshot.src} alt="Screenshot" className="w-full h-full object-cover" />
              ) : (
                <div className="w-full h-full flex items-center justify-center text-zinc-600">🖼</div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}