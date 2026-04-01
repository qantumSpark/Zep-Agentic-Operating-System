# ZAOS — Roadmap & Suivi d'Implementation

> Derniere mise a jour : 2026-04-01
> Statut global : **Phase 11 TERMINE** — V1.5 Stabilisation (3 sprints audit : securite, runtime, events)

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
- [x] AgentsSection — remplace par UnifiedAgentsSection (live delegations + mapping)
- [x] ScreenshotGallery — reecrit avec grid live, zoom, capture, delete
- [x] ActionsFeed — live avec icones, animation running, error detection, result preview
- [x] PipelineSection — pipelineProgress derive depuis workflow-change events
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
- [x] tauri-plugin-dialog (Rust + JS + capability)
- [x] projectStore.ts
- [x] ProjectPicker component + integration StatusBar
- [x] Listener project-changed + reset stores
- [x] Commande start_epic (phase→comprehension, write current-epic.md)
- [x] UI "Start Epic" dans WorkflowSection
- [x] CTA guide dans StartupDashboard

---

## Phase 8 — Workflow Integration Bugs

### 8.1 Hooks Display Fix

- [x] Brancher hooks_active du backend dans le frontend (App.tsx, useTauriEvents.ts)
- [x] Deriver le statut active par hook depuis hooks_active (workflowKitStore.ts)

### 8.2 Format Memoire

- [x] Ajouter specs format exact dans CLAUDE.md (template current-epic.md + state.md)
- [x] Fixer templates par defaut dans init.rs (headings compatibles parser)
- [x] Ajouter rappel format dans inject-context hook (zaos_hooks.rs)

### 8.3 Gate Workflow

- [x] Reset gate_validated dans next_phase() (engine.rs)
- [x] Envoyer message a Claude quand gate valide (commands.rs, auto-inject prompt)
- [x] Feedback visuel apres validate_gate (WorkflowSection.tsx)

---

## Phase 9 — Field-Tested Corrections

> 17 findings du premier test run reel sur un projet vierge (2026-04-01).
> Findings detailles dans `.memory/test-findings.md`.

### 9.1 Pipeline Mecanique (CRITIQUE)

- [x] Epic 9.1.1 — Phase-aware hook : block-code bloque le code hors implementation/test en mode pipeline
- [x] Epic 9.1.2 — Brainstorming → Pipeline : exemple parcours complet dans CLAUDE.md + instructions idle enrichies
- [x] Epic 9.1.3 — Gate enforcement mecanique : enforce-gate hook (Bash) + block-code gate check + inject-context/on-compact gate warning
- [x] Epic 9.1.4 — Inject-context directif : DO/DON'T par phase + rappel memoire apres chaque task

### 9.2 Agents Unifies (HAUTE)

- [x] Epic 9.2.1 — UnifiedAgentsSection : merge agents + delegations, dedup fix, mapAgentName (regex+keywords), pastille active, tri running-first

### 9.3 Dashboard Temps Reel (HAUTE)

- [x] Epic 9.3.1 — Compteur progression epic (barre verte X/Y tasks), StatusBadge verifie, watcher OK

### 9.4 Permissions UX (HAUTE)

- [x] Epic 9.4.1 — Toggle "Accept Edits" (Write/Edit/WebSearch auto-approve), auto-approval interceptor Rust, indicateur visuel, persist state.json

### 9.5 Coordination Agents (HAUTE)

- [x] Epic 9.5.1 — Serialiser npm installs : regle #6 dans CLAUDE.md + instruction inject-context implementation phase

---

## Phase 10 — Test Run #2 Fixes

> 8 findings du test run #2 (2026-04-01, post Phase 9).
> Findings detailles dans `.memory/test-findings-run2.md`.

### 10.1 Hooks Phase-Aware (CRITIQUE)

- [x] CODE_ALLOWED_PHASES += "review" (Write/Edit autorise en review)
- [x] GATE_ENFORCED_PHASES = ["implementation"] (enforce-gate restreint)
- [x] has_active_tasks() retourne true quand 0 data rows + parse par colonne Statut
- [x] Warning ATTENTE GATE restreint a phase implementation (inject-context + on-compact)

### 10.2 Gate Quality Checkpoint (CRITIQUE)

- [x] gate_ready: bool dans WorkflowState (serde default, backward compat)
- [x] Reset gate_ready dans validate_gate, next_phase, start_epic, set_phase + set_gate_ready()
- [x] gate_ready dans WorkflowStateResponse + get_workflow_state
- [x] Detecter fin de turn reussie → gate_ready = true dans event forwarder
- [x] Reset gate_ready = false au debut de send_prompt
- [x] gate_ready dans struct hooks standalone (compat deserialization)
- [x] gateReady dans workflowStore + BackendWorkflowPayload
- [x] Bouton gate disabled/enabled + anti-double-clic + styles conditionnels

### 10.3 Dashboard Empty State (MOYENNE)

- [x] "Plan en attente..." quand 0 tasks + strip "(X/Y taches)" du activeEpic

### 10.4 Delegation Tracking Fix (BASSE)

- [x] Fallback result event : marquer delegations RUNNING comme completed
- [x] Style "stale" pour delegations RUNNING > 10 min (opacity + badge)

---

## Phase 11 — V1.5 Stabilisation (Audit Sprints)

> 3 sprints issus d'un audit de code. Objectif : stabiliser la base technique,
> decoupler le runtime, normaliser le modele d'evenements. "Claude-first but agnostic-ready."

### 11.1 Sprint 1 — Securite & Stabilite

- [x] Path traversal prevention dans agent CRUD (`commands.rs` — `validate_safe_name()`)
- [x] Screenshot path validation (`commands.rs` + `orchestrator.rs` — canonicalize + starts_with)
- [x] CSP headers (`tauri.conf.json` — default-src 'self', script-src, style-src, img-src, connect-src)
- [x] Asset protocol scope restriction (`tauri.conf.json` — `.screenshots/**` + `$APPDATA/**` au lieu de `**`)
- [x] Retrait `shell:allow-execute` (`capabilities/default.json`)
- [x] Extensions bloquees elargies dans hooks (`zaos_hooks.rs` — +.js, .jsx, .css)
- [x] Remplacement des `as any` par types corrects (`useTauriEvents.ts`)
- [x] Gate bypass prevention — `validate_gate` verifie `gate_ready` (`engine.rs`)

### 11.2 Sprint 2 — Couche d'Abstraction Runtime

- [x] Trait `AgentRuntime` (Send + Sync, async-trait) (`runtime/mod.rs`)
- [x] `ClaudeRuntime` — implementation complete extraite de SessionManager (`runtime/claude.rs`)
- [x] `SessionManager` — thin wrapper delegant a `Box<dyn AgentRuntime>` (`session/manager.rs`)

### 11.3 Sprint 3 — Modele d'Evenements Normalise

- [x] `ZaosEvent` — 15 types provider-neutral (discriminated union) (`types/zaosEvents.ts`)
- [x] `mapClaudeEvent()` — mapper Claude → ZAOS (`adapters/claudeMapper.ts`)
- [x] Refactoring `useStreaming.ts` — for/switch sur ZaosEvent au lieu de if/else sur CliEvent
- [x] Refactoring `useTauriEvents.ts` — session/tokens/result via events normalises
- [x] `permissionStore.ts` — `ControlRequest` → `ApprovalRequestedEvent`
- [x] `PermissionRequestBlock.tsx` — acces champs normalises (plus de nested `.request.`)
- [x] `Message.permissionRequest` → `ApprovalRequestedEvent` (`events.ts`)

---

## Compteur de progression

| Phase | Total | Done | Stub | TODO | % |
|-------|-------|------|------|------|---|
| 1.1 Backend Rust | 15 | 15 | 0 | 0 | 100% |
| 1.2 Frontend React | 24 | 24 | 0 | 0 | 100% |
| 1.3 Integration | 8 | 8 | 0 | 0 | 100% |
| **Phase 1 Total** | **47** | **47** | **0** | **0** | **100%** |
| Phase 2 | 13 | 13 | 0 | 0 | 100% |
| Phase 3 | 13 | 13 | 0 | 0 | 100% |
| Phase 4 | 7 | 7 | 0 | 0 | 100% |
| Phase 5 | 12 | 12 | 0 | 0 | 100% |
| Phase 6 | 17 | 17 | 0 | 0 | 100% |
| Phase 7 | 15 | 15 | 0 | 0 | 100% |
| Phase 8 | 8 | 8 | 0 | 0 | 100% |
| Phase 9 | 28 | 28 | 0 | 0 | 100% |
| Phase 10 | 15 | 15 | 0 | 0 | 100% |
| Phase 11 | 18 | 18 | 0 | 0 | 100% |
| **Total** | **193** | **193** | **0** | **0** | **100%** |
