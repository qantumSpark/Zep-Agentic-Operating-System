# Epic active : Phase 4 — MCP Server zaos-ide

> Milestone : 4 — MCP Server zaos-ide
> Date de debut : 2026-03-31
> Statut : EN COURS

## Objectif

Implementer un serveur MCP Streamable HTTP embarque dans le process Tauri, permettant a Claude Code CLI de piloter ZAOS (lire l'etat, afficher des diffs, capturer des screenshots, envoyer des notifications). Communication bidirectionnelle via le standard MCP.

## Architecture

```
Claude Code CLI
    | HTTP direct (Streamable HTTP transport)
    v
ZAOS Tauri App (rmcp StreamableHttpService sur axum, 127.0.0.1:{port}/mcp)
    | Arc<AppState> direct
    v
WorkflowEngine, ScreenshotOrchestrator, AppHandle
```

- rmcp 1.3.0 (SDK officiel MCP en Rust) avec transport-streamable-http-server
- Serveur axum demarre dans Tauri setup(), port aleatoire (bind port 0)
- Auth par token ephemere UUID, verifie sur chaque requete
- .mcp.json auto-genere au demarrage, nettoye a la fermeture
- Claude Code se connecte via `type: "http"` (pas de bridge stdio necessaire)

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| A1 | Add rmcp + axum + schemars to Cargo.toml | `Cargo.toml` | EN COURS | rmcp 1.3, axum 0.8, schemars 1.0 |
| A2 | Rewrite mcp_server/mod.rs — rmcp server + axum | `mcp_server/mod.rs` | TODO | StreamableHttpService, token auth, 4 tool stubs |
| A3 | Wire MCP server start into main.rs setup() | `main.rs` | TODO | Pass Arc<AppState>, spawn tokio task |
| B1 | Implement get_ui_state tool | `mcp_server/mod.rs` | TODO | Workflow state + session info |
| B2 | Implement notify tool | `mcp_server/mod.rs` | TODO | Emit Tauri event → frontend toast |
| B3 | Implement capture_screenshot tool | `mcp_server/mod.rs` | TODO | Delegate to ScreenshotOrchestrator |
| B4 | Implement show_diff tool | `mcp_server/mod.rs` | TODO | Emit Tauri event with diff payload |
| C1 | Auto-generate .mcp.json on server start | `mcp_server/mod.rs` | TODO | type:http, url with port |
| C2 | Cleanup on shutdown — remove .mcp.json + port file | `main.rs` | TODO | Graceful shutdown hook |
| D1 | Add mcp-show-diff event listener | `useTauriEvents.ts` | TODO | + diffStore or inline state |
| D2 | Create DiffViewer component | `DiffViewer.tsx` + `DashboardPanel.tsx` | TODO | Render diff payload |
| D3 | Add mcp-notify listener + toast display | `useTauriEvents.ts` + UI | TODO | OS notification or in-app toast |

## Streams de travail

- **Stream A (Fondation MCP):** A1 → A2 → A3 — serveur operationnel avec stubs
- **Stream B (Tool impls):** B1 + B2 + B3 + B4 (parallelisable apres A3)
- **Stream C (Integration CLI):** C1 + C2 (apres A3)
- **Stream D (Frontend):** D1 + D2 + D3 (parallelisable avec B)

## Dependencies Rust ajoutees

- `rmcp = { version = "1.3", features = ["server", "macros", "transport-streamable-http-server"] }`
- `axum = "0.8"`
- `schemars = "1.0"`
