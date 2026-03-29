import React from "react";

interface Agent {
  name: string;
  filePath: string;
  status: "active" | "inactive" | "error";
  lastDelegation?: { taskDescription: string; timestamp: number };
}

export function AgentsSection() {
  const agents: Agent[] = [
    { name: "Orchestrateur", filePath: "CLAUDE.md", status: "active" },
    { name: "Coder", filePath: ".claude/agents/coder.md", status: "inactive" },
    { name: "Reviewer", filePath: ".claude/agents/reviewer.md", status: "inactive" },
    { name: "Tester", filePath: ".claude/agents/tester.md", status: "inactive" },
  ];

  const statusDot = (status: string) => {
    if (status === "active") return "bg-green-400";
    if (status === "error") return "bg-red-400";
    return "bg-zinc-500";
  };

  return (
    <div className="space-y-3">
      {agents.map((agent) => (
        <div key={agent.name} className="p-2 bg-zinc-800/30 rounded hover:bg-zinc-800/50 transition-colors">
          <div className="flex items-center gap-2">
            <div className={`w-2.5 h-2.5 rounded-full flex-shrink-0 ${statusDot(agent.status)}`} />
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-zinc-100">{agent.name}</p>
              <p className="text-xs text-zinc-500">{agent.filePath}</p>
              {agent.lastDelegation && (
                <div className="mt-1 pt-1 border-t border-zinc-700/50">
                  <p className="text-xs text-zinc-400">Last: {agent.lastDelegation.taskDescription}</p>
                </div>
              )}
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}