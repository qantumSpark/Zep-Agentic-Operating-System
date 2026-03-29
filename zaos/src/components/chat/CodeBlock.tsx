import React from "react";

interface CodeBlockProps {
  code: string;
  language?: string;
}

/**
 * Syntax-highlighted code block
 * Uses CSS class for language-specific styling
 */
export function CodeBlock({ code, language = "" }: CodeBlockProps) {
  return (
    <pre className="bg-zinc-950 border border-zinc-700 rounded px-3 py-2 overflow-x-auto my-2">
      <code className={`language-${language} text-sm text-zinc-300 font-mono`}>
        {code}
      </code>
    </pre>
  );
}
