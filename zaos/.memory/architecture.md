# Architecture ZAOS

> Mis a jour : 2026-03-29
> Modifie uniquement quand l'architecture change.

## Stack technique

- **Framework desktop** : Tauri v2 (Rust backend + WebView frontend)
- **Frontend** : React 19 + TypeScript + Tailwind CSS v4
- **State management** : Zustand v5
- **Markdown** : react-markdown + remark-gfm
- **Backend** : Rust 2021 edition, tokio async runtime
- **Build** : Vite 6 + @vitejs/plugin-react
- **Plateforme cible** : Windows (desktop)

## Structure du projet

```
zaos/
├── src/                          # Frontend React
│   ├── App.tsx                   # Layout principal (SplitPane + StatusBar)
│   ├── main.tsx                  # Point d'entree React
│   ├── components/
│   │   ├── chat/                 # ChatPanel, MessageBubble, CodeBlock, InputBar, ThinkingIndicator
│   │   ├── common/               # SplitPane, CollapsibleSection
│   │   ├── dashboard/            # DashboardPanel, WorkflowSection, AgentsSection, PipelineSection, ActionsFeed, ScreenshotGallery
│   │   └── statusbar/            # StatusBar
│   ├── hooks/                    # useStreaming, useTauriEvents
│   ├── stores/                   # chatStore, sessionStore, workflowStore, actionsStore (Zustand)
│   └── types/                    # events.ts, workflow.ts
├── src-tauri/                    # Backend Rust
│   ├── src/
│   │   ├── main.rs               # Entry point Tauri
│   │   ├── commands.rs           # 7 commandes IPC (send_prompt, validate_gate, etc.)
│   │   ├── events/               # types.rs, parser.rs — parse stream-json CLI
│   │   ├── session/              # manager.rs — spawn/manage Claude Code CLI
│   │   ├── workflow/             # engine.rs, state.rs — state machine phases
│   │   ├── screenshots/          # manager.rs — stub Phase 3
│   │   ├── memory/               # reader.rs — stub Phase 2
│   │   └── mcp_server/           # mod.rs — stub Phase 4
│   ├── Cargo.toml
│   └── tauri.conf.json
├── .memory/                      # Memoire projet (ce dossier)
└── docs/                         # Design doc, protocoles
```

## Systemes principaux

### Session Manager (Rust)
- **Fichiers** : `src-tauri/src/session/manager.rs`
- **Responsabilite** : spawn Claude Code CLI avec `--output-format stream-json`, gere stdin/stdout, resume de session
- **Interactions** : Event Parser (stdout), Commands (IPC)

### Event Parser (Rust)
- **Fichiers** : `src-tauri/src/events/parser.rs`, `types.rs`
- **Responsabilite** : parse le flux JSONL du CLI, broadcast via tokio channel
- **Interactions** : Session Manager (source), Frontend (via app.emit)

### Workflow Engine (Rust)
- **Fichiers** : `src-tauri/src/workflow/engine.rs`, `state.rs`
- **Responsabilite** : lit/ecrit `.workflow/state.json`, transitions de phase, gate validation
- **Interactions** : Commands (IPC), Frontend (via events)

### Chat System (React)
- **Fichiers** : `src/components/chat/*`, `src/hooks/useStreaming.ts`, `src/stores/chatStore.ts`
- **Responsabilite** : envoi de prompts, affichage streaming des reponses, rendu Markdown
- **Interactions** : Backend via `invoke("send_prompt")` et event listener `agent-event`

### Dashboard (React)
- **Fichiers** : `src/components/dashboard/*`, `src/stores/workflowStore.ts`, `actionsStore.ts`
- **Responsabilite** : visualisation temps reel du workflow, agents, actions, pipeline
- **Interactions** : Backend via events Tauri et `invoke()` calls

## Communication Frontend <-> Backend

- **Frontend → Backend** : `invoke("command_name", { args })` (Tauri IPC)
- **Backend → Frontend** : `app.emit("event-name", payload)` (Tauri events)
- **Protocole CLI** : stream-json (JSONL via stdout), doc dans `docs/stream-json-protocol.md`
