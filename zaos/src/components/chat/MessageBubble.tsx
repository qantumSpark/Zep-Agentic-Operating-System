import React from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { CodeBlock } from "./CodeBlock";
import type { Message } from "../../types/events";

interface MessageBubbleProps {
  message: Message;
}

/**
 * Message bubble with different styles for user/assistant/system
 * Renders markdown for assistant messages
 */
export function MessageBubble({ message }: MessageBubbleProps) {
  const isAssistant = message.role === "assistant";
  const isUser = message.role === "user";
  const isSystem = message.role === "system";

  const bubbleClass = isUser
    ? "ml-auto bg-blue-600 text-white max-w-2xl"
    : isSystem
      ? "mx-auto bg-zinc-700 text-zinc-200 max-w-2xl text-sm italic"
      : "mr-auto bg-zinc-800 text-zinc-100 max-w-2xl";

  return (
    <div className={`flex ${isUser ? "justify-end" : "justify-start"}`}>
      <div className={`rounded-lg px-4 py-2 ${bubbleClass}`}>
        {/* Agent badge */}
        {message.agent && (
          <div className="text-xs font-semibold text-blue-300 mb-1">
            [{message.agent}]
          </div>
        )}

        {/* Content */}
        {isAssistant ? (
          <div className="prose prose-invert prose-sm max-w-none">
            <ReactMarkdown
              remarkPlugins={[remarkGfm]}
              components={{
                code: ({ node, inline, className, children, ...props }) => {
                  if (inline) {
                    return (
                      <code className="bg-zinc-900 px-1 rounded text-blue-300">
                        {children}
                      </code>
                    );
                  }
                  return (
                    <CodeBlock
                      code={String(children).replace(/\n$/, "")}
                      language={className?.replace(/language-/, "") || ""}
                    />
                  );
                },
                p: ({ children }) => <p className="my-1">{children}</p>,
                ul: ({ children }) => (
                  <ul className="list-disc list-inside my-1">{children}</ul>
                ),
                ol: ({ children }) => (
                  <ol className="list-decimal list-inside my-1">{children}</ol>
                ),
              }}
            >
              {message.content}
            </ReactMarkdown>
          </div>
        ) : (
          <div className="whitespace-pre-wrap break-words">{message.content}</div>
        )}

        {/* Timestamp */}
        <div className="text-xs text-zinc-400 mt-1">
          {new Date(message.timestamp).toLocaleTimeString()}
        </div>
      </div>
    </div>
  );
}
