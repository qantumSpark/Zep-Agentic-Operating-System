import React from "react";
import { Circle } from "lucide-react";

interface Agent {
  name: string;
  filePath: string;
  status: "active" | "inactive" | "error";
  lastDelegation?: {
    taskDescription: string;
    timestamp: number;
  };
}

/**
 * Agent list with active/inactive/error states
 */
export function AgentsSection() {
  // TODO: Connect to actual agent data from stores or backend
  const agents: Agent[] = [
    {
      name: "Orchestrateur",
      filePath: ".claude/agents/orchestrator.md",
      status: "active",
    },
    {
      name: "Coder",
      filePath: ".claude/agents/coder.md",
      status: "active",
      lastDelegation: {
        taskDescription: "Implement hitbox system",
        timestamp: Date.now() - 60000,
      },
    },
    {
      name: "Reviewer",
      filePath: ".claude/agents/reviewer.md",
      status: "inactive",
    },
    {
      name: "Tester",
      filePath: ".claude/agents/tester.md",
      status: "inactive",
    },
  ];

  const getStatusColor = (status: "active" | "inactive" | "error") => {
    switch (status) {
      case "active":
        return "text-green-400";
      case "error":
        return "text-red-400";
      default:
        return "text-zinc-400";
    }
  };

  return (
    <div className="space-y-3">
      {agents.map((agent) => (
        <div
          key={agent.name}
          className="p-2 bg-zinc-800/30 rounded hover:bg-zinc-800/50 transition-colors"
        >
          <div className="flex items-center gap-2">
            <Circle
              size={10}
              className={`flex-shrink-0 ${getStatusColor(agent.status)}`}
              fill="currentColor"
            />
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-zinc-100">{agent.name}</p>
              <p className="text-xs text-zinc-500">{agent.filePath}</p>

              {agent.lastDelegation && (
                <div className="mt-1 pt-1 border-t border-zinc-700/50">
                  <p className="text-xs text-zinc-400">
                    Last: {agent.lastDelegation.taskDescription}
                  </p>
                  <p className="text-xs text-zinc-600">
                    {new Date(
                      agent.lastDelegation.timestamp
                    ).toLocaleTimeString()}
                  </p>
                </div>
              )}
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}
