import React, { useState } from "react";

interface CollapsibleSectionProps {
  title: string;
  children: React.ReactNode;
  defaultOpen?: boolean;
  className?: string;
}

export function CollapsibleSection({
  title,
  children,
  defaultOpen = true,
  className = "",
}: CollapsibleSectionProps) {
  const [isOpen, setIsOpen] = useState(defaultOpen);

  return (
    <div className={`border-b border-zinc-700 ${className}`}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="w-full flex items-center justify-between px-4 py-3 hover:bg-zinc-800 transition-colors"
      >
        <h3 className="text-sm font-semibold text-zinc-100">{title}</h3>
        <span className={`text-zinc-400 transition-transform ${isOpen ? "rotate-0" : "-rotate-90"}`}>
          ▼
        </span>
      </button>
      {isOpen && (
        <div className="px-4 py-3 bg-zinc-800/50 border-t border-zinc-700/50">
          {children}
        </div>
      )}
    </div>
  );
}