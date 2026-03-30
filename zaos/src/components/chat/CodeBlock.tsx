import { useState } from "react";
import { Highlight, themes } from "prism-react-renderer";

interface CodeBlockProps {
  code: string;
  language?: string;
}

const LANGUAGE_ALIASES: Record<string, string> = {
  gdscript: "python",
  gd: "python",
};

export function CodeBlock({ code, language = "" }: CodeBlockProps) {
  const resolvedLang = LANGUAGE_ALIASES[language] || language;
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="relative group">
      <button
        onClick={handleCopy}
        className="absolute top-2 right-2 p-1 rounded bg-zinc-700/50 text-zinc-400 opacity-0 group-hover:opacity-100 hover:bg-zinc-600 hover:text-zinc-200 transition-opacity cursor-pointer z-10"
        aria-label="Copy code"
      >
        {copied ? (
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <polyline points="20 6 9 17 4 12" />
          </svg>
        ) : (
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
          </svg>
        )}
      </button>
      <Highlight theme={themes.vsDark} code={code} language={resolvedLang}>
        {({ className, style, tokens, getLineProps, getTokenProps }) => (
          <pre
            className="bg-zinc-950 border border-zinc-700 rounded px-3 py-2 overflow-x-auto my-2"
            style={{ ...style, backgroundColor: "transparent" }}
          >
            <code className={`${className} text-sm font-mono`}>
              {tokens.map((line, i) => {
                const lineProps = getLineProps({ line, key: i });
                return (
                  <div key={i} {...lineProps}>
                    {line.map((token, j) => (
                      <span key={j} {...getTokenProps({ token, key: j })} />
                    ))}
                  </div>
                );
              })}
            </code>
          </pre>
        )}
      </Highlight>
    </div>
  );
}
