import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useWorkflowKitStore, AgentDef } from "../../stores/workflowKitStore";

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

function DeployedBadge() {
  return (
    <span className="inline-block px-1.5 py-0.5 rounded text-[10px] font-semibold uppercase leading-none bg-green-700/60 text-green-300">
      deployed
    </span>
  );
}

function CustomBadge() {
  return (
    <span className="inline-block px-1.5 py-0.5 rounded text-[10px] font-semibold uppercase leading-none bg-violet-700/60 text-violet-300">
      custom
    </span>
  );
}

// ---------------------------------------------------------------------------
// Inline Add Form
// ---------------------------------------------------------------------------

function AddAgentForm({
  onSubmit,
  onCancel,
}: {
  onSubmit: (name: string, content: string) => void;
  onCancel: () => void;
}) {
  const [name, setName] = useState("");
  const [content, setContent] = useState("");

  const handleSubmit = () => {
    const trimmedName = name.trim();
    const trimmedContent = content.trim();
    if (!trimmedName) return;
    onSubmit(trimmedName, trimmedContent);
  };

  return (
    <div className="rounded border border-zinc-700 bg-zinc-800/60 p-3 space-y-2">
      <input
        type="text"
        placeholder="Agent name"
        value={name}
        onChange={(e) => setName(e.target.value)}
        className="w-full bg-zinc-900 border border-zinc-700 rounded px-2 py-1 text-xs text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-zinc-500"
        autoFocus
      />
      <textarea
        placeholder="Agent description / instructions..."
        value={content}
        onChange={(e) => setContent(e.target.value)}
        rows={4}
        className="w-full bg-zinc-900 border border-zinc-700 rounded px-2 py-1 text-xs text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-zinc-500 resize-y"
      />
      <div className="flex gap-2 justify-end">
        <button
          onClick={onCancel}
          className="px-3 py-1 rounded text-xs font-medium bg-zinc-700 text-zinc-300 hover:bg-zinc-600 transition-colors"
        >
          Cancel
        </button>
        <button
          onClick={handleSubmit}
          disabled={!name.trim()}
          className="px-3 py-1 rounded text-xs font-medium bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
        >
          Create
        </button>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Agent Card
// ---------------------------------------------------------------------------

function AgentCard({
  agent,
  onEdit,
  onDelete,
}: {
  agent: AgentDef;
  onEdit: (name: string, content: string) => void;
  onDelete: (name: string) => void;
}) {
  const [editing, setEditing] = useState(false);
  const [editContent, setEditContent] = useState("");
  const [loading, setLoading] = useState(false);

  const startEditing = async () => {
    setLoading(true);
    try {
      const content = await invoke<string>("read_agent", { name: agent.name });
      setEditContent(content);
    } catch {
      setEditContent(agent.description);
    }
    setLoading(false);
    setEditing(true);
  };

  const handleSave = () => {
    onEdit(agent.name, editContent.trim());
    setEditing(false);
  };

  const handleCancel = () => {
    setEditContent("");
    setEditing(false);
  };

  return (
    <div className="flex flex-col gap-1.5 px-2 py-2 rounded bg-zinc-800/60 hover:bg-zinc-800/80 transition-colors">
      {/* Top row: icon + name + badges */}
      <div className="flex items-center gap-2">
        <span className="text-zinc-500 text-xs flex-shrink-0">&#9679;</span>
        <span className="text-xs text-zinc-200 font-medium truncate">
          {agent.name}
        </span>
        <div className="flex items-center gap-1 ml-auto flex-shrink-0">
          {agent.deployed && <DeployedBadge />}
          {agent.custom && <CustomBadge />}
        </div>
      </div>

      {/* Description or edit textarea */}
      {editing ? (
        <div className="space-y-2 ml-4">
          <textarea
            value={editContent}
            onChange={(e) => setEditContent(e.target.value)}
            rows={4}
            className="w-full bg-zinc-900 border border-zinc-700 rounded px-2 py-1 text-xs text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-zinc-500 resize-y"
            autoFocus
          />
          <div className="flex gap-2 justify-end">
            <button
              onClick={handleCancel}
              className="px-2 py-0.5 rounded text-[10px] font-medium bg-zinc-700 text-zinc-300 hover:bg-zinc-600 transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={handleSave}
              className="px-2 py-0.5 rounded text-[10px] font-medium bg-blue-600 text-white hover:bg-blue-700 transition-colors"
            >
              Save
            </button>
          </div>
        </div>
      ) : (
        <>
          {agent.description && (
            <p className="text-zinc-400 text-xs ml-4 leading-relaxed">
              {agent.description}
            </p>
          )}
          {/* Action buttons for custom agents */}
          {agent.custom && (
            <div className="flex gap-2 ml-4 mt-0.5">
              <button
                onClick={startEditing}
                disabled={loading}
                className="text-[10px] text-zinc-500 hover:text-zinc-300 transition-colors"
              >
                Edit
              </button>
              <button
                onClick={() => onDelete(agent.name)}
                className="text-[10px] text-zinc-500 hover:text-red-400 transition-colors"
              >
                Delete
              </button>
            </div>
          )}
        </>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// AgentsManager
// ---------------------------------------------------------------------------

export function AgentsManager() {
  const agents = useWorkflowKitStore((s) => s.agents);
  const setAgents = useWorkflowKitStore((s) => s.setAgents);
  const [showAddForm, setShowAddForm] = useState(false);
  const [deploying, setDeploying] = useState(false);

  // -- Deploy workflow kit --
  const handleDeploy = useCallback(async () => {
    setDeploying(true);
    try {
      await invoke("deploy_workflow_kit");
      const status = await invoke<{ agents?: AgentDef[] }>(
        "get_workflow_kit_status",
      );
      if (status?.agents) {
        setAgents(status.agents);
      }
    } catch (error) {
      console.error("Failed to deploy workflow kit:", error);
    } finally {
      setDeploying(false);
    }
  }, [setAgents]);

  // -- Create agent --
  const handleCreate = useCallback(
    async (name: string, content: string) => {
      const newAgent: AgentDef = {
        name,
        description: content.split("\n")[0] || "",
        deployed: false,
        custom: true,
      };
      // Optimistic update regardless of backend result
      setAgents([...agents, newAgent]);
      setShowAddForm(false);
      try {
        await invoke("save_agent", { name, content });
      } catch (error) {
        console.error("Failed to create agent:", error);
      }
    },
    [agents, setAgents],
  );

  // -- Edit agent --
  const handleEdit = useCallback(
    async (name: string, content: string) => {
      try {
        await invoke("save_agent", { name, content });
      } catch (error) {
        console.error("Failed to update agent:", error);
      }
      // Optimistic update
      setAgents(
        agents.map((a) =>
          a.name === name
            ? { ...a, description: content.split("\n")[0] || "" }
            : a,
        ),
      );
    },
    [agents, setAgents],
  );

  // -- Delete agent --
  const handleDelete = useCallback(
    async (name: string) => {
      try {
        await invoke("delete_agent", { name });
      } catch (error) {
        console.error("Failed to delete agent:", error);
      }
      // Optimistic update
      setAgents(agents.filter((a) => a.name !== name));
    },
    [agents, setAgents],
  );

  return (
    <div className="space-y-3 text-sm">
      {/* ---- Header row ---- */}
      <div className="flex items-center gap-2">
        <label className="text-zinc-400 text-xs uppercase tracking-wide flex-1">
          Agents
        </label>
        <button
          onClick={handleDeploy}
          disabled={deploying}
          className="px-2.5 py-1 rounded text-xs font-medium bg-green-600 text-white hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {deploying ? "Deploying..." : "Deploy"}
        </button>
        <button
          onClick={() => setShowAddForm((prev) => !prev)}
          className="px-2.5 py-1 rounded text-xs font-medium bg-zinc-700 text-zinc-200 hover:bg-zinc-600 transition-colors"
        >
          {showAddForm ? "Cancel" : "+ Add"}
        </button>
      </div>

      {/* ---- Add form (inline) ---- */}
      {showAddForm && (
        <AddAgentForm
          onSubmit={handleCreate}
          onCancel={() => setShowAddForm(false)}
        />
      )}

      {/* ---- Agent list ---- */}
      {agents.length === 0 ? (
        <p className="text-zinc-500 text-xs italic">
          No agents configured. Deploy or add one.
        </p>
      ) : (
        <div className="space-y-1">
          {agents.map((agent) => (
            <AgentCard
              key={agent.name}
              agent={agent}
              onEdit={handleEdit}
              onDelete={handleDelete}
            />
          ))}
        </div>
      )}
    </div>
  );
}
