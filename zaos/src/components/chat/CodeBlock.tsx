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

  return (
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
  );
}
