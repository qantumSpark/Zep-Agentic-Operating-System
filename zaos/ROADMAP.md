# ZAOS — Roadmap & Suivi d'Implementation

> Derniere mise a jour : 2026-03-31
> Statut global : **Phase 7 EN COURS** — Project Portability & Workflow Init

---

## Legende

- [x] Termine et fonctionnel
- [~] Stub/placeholder (compile mais pas de vraie logique)
- [ ] A faire

---

## Phase 1 — Chat Fonctionnel

### 1.1 Backend Rust (Tauri)

- [x] Tauri v2 setup (Cargo.toml, tauri.conf.json, build.rs, capabilities)
- [x] Session Manager — spawn CLI avec `--output-format stream-json --verbose`
- [x] Session Manager — resume via `--resume` + session_id
- [x] Event Parser — parse JSONL stream depuis stdout CLI
- [x] Event Types — systeme complet (System, StreamDelta, Assistant, User, RateLimit, Result)
- [x] Broadcast channel — tokio broadcast vers toutes les listeners
- [x] Workflow Engine — lecture/ecriture `.workflow/state.json`
- [x] Workflow Engine — transitions de phase, historique, gate validation
- [x] Commandes IPC — send_prompt, validate_gate, set_mode, get_workflow_state, check_cli_auth
- [x] AppState thread-safe (Arc<Mutex<>>)
- [x] Tracing/logging configure
- [x] Ajouter `--include-partial-messages` au spawn CLI
- [x] interrupt_session — real implementation via stdin signal to long-lived CLI process
- [x] Long-lived CLI process model with stdin communication (`--permission-prompt-tool stdio`)
- [x] list_sessions — lit les sessions depuis ~/.claude/projects/ avec metadata

### 1.2 Frontend React

- [x] Layout split-panel (SplitPane drag-to-resize)
- [x] ChatPanel — affichage des messages, auto-scroll
- [x] InputBar — envoi via `invoke("send_prompt")`, gestion Enter/Shift+Enter
- [x] MessageBubble — rendu user/assistant/system avec Markdown (react-markdown + remark-gfm)
- [x] CodeBlock — rendu du code avec classes CSS
- [x] useStreaming hook — ecoute `agent-event`, parse text_delta, accumule dans chatStore
- [x] useTauriEvents hook — parse System, RateLimit, Result, workflow-change events
- [x] chatStore (Zustand) — messages, streaming buffer, dedup
- [x] sessionStore — tokens, duree, modele, connexions
- [x] workflowStore — phase, epic, task, mode, gate
- [x] actionsStore — structure complete (addAction, updateStatus)
- [x] StatusBar — tokens, duree, modele, connexions (donnees live depuis les stores)
- [x] WorkflowSection — affiche epic/phase/task, boutons validate_gate et set_mode
- [x] CollapsibleSection — expand/collapse avec animation
- [x] DashboardPanel — layout avec toutes les sections
- [x] Afficher les tool_use blocks dans le chat (Read, Write, Bash, etc.)
- [x] Brancher ThinkingIndicator dans ChatPanel
- [x] Nourrir actionsStore depuis les tool_use events
- [~] AgentsSection — liste hardcodee, pas de tracking reel
- [~] ScreenshotGallery — array toujours vide
- [x] ActionsFeed — live avec icones, animation running, error detection, result preview
- [~] PipelineSection — lit le store mais pipelineProgress vide sans events backend
- [x] Syntax highlighting reel (prism-react-renderer v2, vsDark theme, language aliases)
- [x] Bouton copy-to-clipboard sur CodeBlock

### 1.3 Integration Frontend <-> Backend

- [x] send_prompt : InputBar → Rust → CLI spawn → events stream → useStreaming → chat
- [x] validate_gate : WorkflowSection → Rust → state.json
- [x] set_mode : WorkflowSection → Rust → workflow engine
- [x] get_workflow_state : useTauriEvents → Rust → state.json
- [x] check_cli_auth : disponible (pas appele au startup)
- [x] Appeler check_cli_auth au demarrage et afficher le statut
- [x] Forwarder les tool_use events vers actionsStore
- [x] Implementer interrupt reel (signal via stdin to long-lived CLI process)

---

## Phase 2 — Dashboard Temps Reel

### 2.1 Actions Feed vivant

- [x] Parser les tool_use content blocks depuis les assistant events
- [x] Mapper tool_use → action (nom, input, status, duree)
- [x] Parser les user/tool_result events pour le status (success/error)
- [x] Afficher dans ActionsFeed avec icones par type (Read, Write, Bash, Glob, etc.)

### 2.2 Agents tracking

- [x] Detecter l'agent actif depuis le contexte des events (system prompts, delegation)
- [x] Remplacer la liste hardcodee par des donnees live
- [x] Afficher l'historique de delegation

### 2.3 Pipeline vivant

- [x] File watcher sur `.workflow/state.json` (notify crate)
- [x] Emettre `workflow-change` events depuis le backend quand le fichier change
- [x] Mettre a jour pipelineProgress dans le store en temps reel

### 2.4 Memory Reader (Rust)

- [x] Parser `.memory/INDEX.md` (extraire la liste des fichiers memoire)
- [x] Parser `.memory/state.md` (milestones, epic en cours, blocages)
- [x] Parser `.memory/current-epic.md` (tasks et statuts)
- [x] Exposer via commande IPC `get_memory_state`
- [x] Afficher dans le dashboard (section memoire)

### 2.5 File Watchers

- [x] Watcher sur `.workflow/state.json` → emit workflow-change
- [x] Watcher sur `.memory/` → emit memory-change
- [x] Debounce pour eviter le spam d'events

---

## Phase 3 — Screenshots & Visuels

### 3.1 Screenshot Manager (Rust)

- [x] watch_directory — surveiller `.screenshots/` (FileWatcher + WatchCategory::Screenshot)
- [x] capture_on_demand — CaptureAdapter trait + FilesystemAdapter + CliMcpAdapter
- [x] get_gallery — ScreenshotOrchestrator.get_gallery() + IPC get_screenshots
- [x] cleanup_old — delete_screenshot IPC + orchestrator.delete_screenshot()
- [x] compare — ComparisonView.tsx (side-by-side + slider mode)

### 3.2 Frontend Screenshots

- [x] ScreenshotGallery — rewritten with live grid, zoom modal, capture, delete
- [x] Insertion inline — screenshotStore + useTauriEvents listeners (screenshot-new, iteration-update)
- [x] Before/after comparison view — ComparisonView.tsx

---

## Phase 4 — MCP Server zaos-ide

- [x] Serveur MCP Streamable HTTP embarque (rmcp 1.3 + axum, port aleatoire)
- [x] Token d'auth ephemere (bearer UUID, verifie par middleware)
- [x] Tool: show_diff — emit Tauri event, DiffViewer frontend
- [x] Tool: notify — emit Tauri event, OS notification via browser API
- [x] Tool: get_ui_state — lire l'etat workflow + project_dir
- [x] Tool: capture_screenshot — delegue a ScreenshotOrchestrator
- [x] Enregistrement .mcp.json auto-genere + cleanup au shutdown

---

## Phase 5 — UX Polish

- [x] Historique de sessions (navigation, recherche)
- [x] Metriques par session (tokens/phase, temps/agent)
- [x] Auto-generation des session logs dans `.memory/sessions/`
- [x] Dashboard au startup (resume projet, etat workflow)
- [x] Notifications OS pour gates et taches longues (tauri-plugin-notification)
- [x] Raccourcis clavier configurables (useKeyboardShortcuts hook)
- [x] Theme system (dark/light, Zustand persist, CSS variables)
- [x] Activity Feedback — heartbeat visuel dans StatusBar quand le CLI est actif
- [x] Activity Feedback — "last seen Xs ago" sur les agents running
- [x] Activity Feedback — stream preview (derniere ligne en cours) dans StatusBar
- [x] Activity Feedback — indicateur visuel (glow/bordure) sur ChatPanel pendant streaming
- [x] Activity Feedback — sub-agent tool_use dans ActionsFeed avec indentation

---

## Phase 6 — Workflow Kit Integration

- [x] Config types .zaos/config.json (deployer/config.rs)
- [x] Embed resources dans tauri.conf.json
- [x] Deployer sync engine (hash compare, write missing/updated, manifest fast-path)
- [x] Deployer module + IPC commands (deploy, get_status, update_config)
- [x] Bootstrap complet dans init.rs (.zaos/, .claude/, agents, hooks, rules)
- [x] zaos-hooks binary scaffold (Rust CLI, [[bin]] target)
- [x] zaos-hooks inject-context (role Orchestrateur + regles + phase)
- [x] zaos-hooks block-code (check extension + plan + mode)
- [x] zaos-hooks on-compact + welcome (reinjecte contexte)
- [x] workflowKitStore (agents, hooks, rules, config)
- [x] AgentsManager CRUD (liste + edit via read_agent + add + delete)
- [x] HooksManager (statut + toggle + shared Toggle component)
- [x] RulesManager (liste + toggle + shared components)
- [x] Remplacer AgentsSection par nouvelles sections
- [x] Auto-deploy dans start_session()
- [x] Watcher .claude/ → refresh frontend (filtered to agents/ changes)
- [x] IPC agents CRUD (read/write/delete .md files)

---

## Phase 7 — Project Portability & Workflow Init

- [x] Embedded content module (include_str! pour 6 agents + 4 rules)
- [x] Rewrite sync_agents/sync_rules sans source_dir (utilise embedded)
- [x] Supprimer find_reference_dir + bundle.resources
- [x] AppState.project_dir → Arc<RwLock<PathBuf>>
- [x] Mettre a jour tous les commands (read().await)
- [x] Commande switch_project (kill session, swap services, re-init)
- [x] Commande get_project_info
- [x] FileWatcherService stop/restart (CancellationToken)
- [ ] tauri-plugin-dialog (Rust + JS + capability)
- [ ] projectStore.ts
- [ ] ProjectPicker component + integration StatusBar
- [ ] Listener project-changed + reset stores
- [ ] Commande start_epic (phase→comprehension, write current-epic.md)
- [ ] UI "Start Epic" dans WorkflowSection
- [ ] CTA guide dans StartupDashboard

---

## Compteur de progression

| Phase | Total | Done | Stub | TODO | % |
|-------|-------|------|------|------|---|
| 1.1 Backend Rust | 15 | 14 | 0 | 0 | 93% |
| 1.2 Frontend React | 24 | 21 | 3 | 0 | 88% |
| 1.3 Integration | 8 | 8 | 0 | 0 | 100% |
| **Phase 1 Total** | **47** | **43** | **3** | **0** | **91%** |
| Phase 2 | 13 | 13 | 0 | 0 | 100% |
| Phase 3 | 13 | 13 | 0 | 0 | 100% |
| Phase 4 | 7 | 7 | 0 | 0 | 100% |
| Phase 5 | 12 | 12 | 0 | 0 | 100% |
| Phase 6 | 17 | 17 | 0 | 0 | 100% |
| Phase 7 | 15 | 10 | 0 | 5 | 67% |
| **Total** | **124** | **115** | **3** | **5** | **93%** |
