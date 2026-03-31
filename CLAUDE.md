# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.
## Absolute Rules

You are an Orchestrator, whenever it's possible, you ALWAYS deleguate task to sub-agents

## Repository Structure

This repo contains two sub-projects:

- **`workflow-kit/`** — Reusable file kit (templates, rules, agents, memory system) that turns Claude Code into a structured dev assistant for Godot/Flutter projects. This is reference material, not an application.
- **`zaos/`** — Tauri v2 desktop app providing a visual interface for piloting AI-assisted dev workflows. This is the active codebase.

## ZAOS Architecture

Tauri v2 app with a Rust backend and React frontend communicating via IPC.

**Data flow:** User types in InputBar → `invoke("send_prompt")` → Rust spawns `claude -p "..." --output-format stream-json --verbose` → JSONL stdout parsed by EventParser → events broadcast via `app.emit("agent-event")` → React hooks (`useStreaming`, `useTauriEvents`) update Zustand stores → UI re-renders.

**Frontend** (React 19 + TypeScript + Tailwind v4 + Zustand v5):
- Split-panel layout: Chat (left) + Dashboard (right) + StatusBar (bottom)
- Hooks `useStreaming` and `useTauriEvents` are the sole bridge from Tauri events to UI state
- All state in Zustand stores: `chatStore`, `sessionStore`, `workflowStore`, `actionsStore`

**Backend** (Rust, tokio async):
- `session/manager.rs` — Spawns and manages Claude Code CLI subprocess
- `events/parser.rs` + `types.rs` — Parses stream-json JSONL protocol
- `workflow/engine.rs` + `state.rs` — Reads/writes `.workflow/state.json`, manages phase transitions
- `commands.rs` — 7 Tauri IPC commands: `send_prompt`, `interrupt_session`, `validate_gate`, `set_mode`, `get_workflow_state`, `check_cli_auth`, `list_sessions`

**Protocol reference:** `zaos/docs/stream-json-protocol.md` documents the CLI output format. Reference traces in `zaos/reference/traces/`.

## Build & Dev Commands

```bash
# Frontend dev server (from zaos/)
cd zaos && npm run dev

# Tauri dev (frontend + backend, from zaos/)
cd zaos && npm run tauri dev

# Rust build only (from zaos/src-tauri/)
cd zaos/src-tauri && cargo build

# TypeScript check (from zaos/)
cd zaos && npx tsc --noEmit

# Production build (from zaos/)
cd zaos && npm run tauri build
```

## Workflow-Driven Development

This project uses a structured workflow with persistent memory. Before coding:

1. Read `zaos/.memory/INDEX.md` → `state.md` → `current-epic.md`
2. Check `zaos/ROADMAP.md` for overall progress
3. A validated task plan must exist in `current-epic.md` before writing code
4. **After architecture/planning, before any implementation:** launch a research agent to verify the plan against real documentation, web sources, crate/package docs, and community best practices. The agent must confirm that APIs, crates, protocols, and patterns referenced in the plan actually exist and work as assumed. Update the plan if findings contradict it. Never start coding an unverified plan.
5. Follow phases: comprehension → spec → architecture → **verification** → implementation → review → closure
6. Each phase has a gate — user must validate before advancing
7. Update `.memory/state.md` and `.memory/current-epic.md` after completing work
8. **After completing any phase or milestone:** update `ROADMAP.md` (mark tasks done, update progress table and status line), `.memory/state.md` (milestone status, priorities), and `.memory/current-epic.md` (task statuses). These files must always reflect the current state of the project.

## Key Conventions

**TypeScript/React:** Functional components only, named exports, Zustand for state, Tailwind for styling. No `any`, no `useEffect` for state management.

**Rust:** `tokio` async, `serde` for JSON, `tracing` for logging (never `println!`), `Arc<Mutex<>>` for shared state, `?` operator for errors (never `unwrap()` in production).

**Tasks:** Each task touches 1 file (2-3 max if coupled), is verifiable independently, and described in one sentence with the what and the where.
