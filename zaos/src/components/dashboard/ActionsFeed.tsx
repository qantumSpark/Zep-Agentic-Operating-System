import React from "react";
import { useActionsStore } from "../../stores/actionsStore";
import {
  FileText,
  Edit,
  Terminal,
  Image,
  Search,
  Zap,
  Globe,
  CheckCircle,
  AlertCircle,
  Clock,
} from "lucide-react";

/**
 * Chronological action log with icons, timestamps, status
 */
export function ActionsFeed() {
  const actions = useActionsStore((state) => state.actions);

  const getToolIcon = (tool: string) => {
    switch (tool) {
      case "Read":
        return <Search size={14} />;
      case "Write":
        return <FileText size={14} />;
      case "Edit":
        return <Edit size={14} />;
      case "Bash":
        return <Terminal size={14} />;
      case "Screenshot":
        return <Image size={14} />;
      case "Glob":
        return <Search size={14} />;
      case "Grep":
        return <Search size={14} />;
      case "WebSearch":
      case "WebFetch":
        return <Globe size={14} />;
      case "Task":
        return <Zap size={14} />;
      default:
        return <FileText size={14} />;
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case "success":
        return <CheckCircle size={14} className="text-green-400" />;
      case "error":
        return <AlertCircle size={14} className="text-red-400" />;
      case "running":
        return <Clock size={14} className="text-blue-400 animate-spin" />;
      default:
        return <Clock size={14} className="text-zinc-400" />;
    }
  };

  return (
    <div className="space-y-2 max-h-96 overflow-y-auto">
      {actions.length === 0 ? (
        <div className="text-zinc-500 text-sm italic">No actions yet</div>
      ) : (
        [...actions].reverse().map((action) => (
          <div
            key={action.id}
            className="flex items-start gap-2 p-2 bg-zinc-800/30 rounded hover:bg-zinc-800/50 transition-colors text-xs"
          >
            <div className="flex-shrink-0 mt-0.5 text-zinc-400">
              {getToolIcon(action.tool)}
            </div>

            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2">
                <span className="font-medium text-zinc-200 truncate">
                  {action.summary}
                </span>
                {getStatusIcon(action.status)}
              </div>
              <div className="text-zinc-500 mt-0.5">
                {new Date(action.timestamp).toLocaleTimeString()}
              </div>
            </div>
          </div>
        ))
      )}
    </div>
  );
}
