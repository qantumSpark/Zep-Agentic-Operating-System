# ZAOS — Roadmap & Suivi d'Implementation

> Derniere mise a jour : 2026-03-30
> Statut global : **Phase 1 TERMINEE** — Chat fonctionnel complet, prêt pour Phase 2 (Dashboard temps réel)

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

- [ ] Detecter l'agent actif depuis le contexte des events (system prompts, delegation)
- [ ] Remplacer la liste hardcodee par des donnees live
- [ ] Afficher l'historique de delegation

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

- [ ] watch_directory() — surveiller un dossier de screenshots
- [ ] capture_on_demand() — appeler GoPeak via MCP pour capturer
- [ ] get_gallery() — lister les screenshots avec metadata
- [ ] cleanup_old() — nettoyer les vieux screenshots
- [ ] compare() — diff visuel before/after

### 3.2 Frontend Screenshots

- [ ] ScreenshotGallery — afficher les captures avec zoom
- [ ] Insertion inline de screenshots dans le chat
- [ ] Before/after comparison view

---

## Phase 4 — MCP Server zaos-ide

- [ ] Serveur TCP local sur 127.0.0.1:port_aleatoire
- [ ] Token d'auth ephemere
- [ ] Tool: show_diff — afficher un diff dans l'UI
- [ ] Tool: notify — notification OS depuis Claude
- [ ] Tool: get_ui_state — lire l'etat du dashboard
- [ ] Tool: capture_screenshot — declencher une capture
- [ ] Enregistrement dans .mcp.json du projet

---

## Phase 5 — UX Polish

- [ ] Historique de sessions (navigation, recherche)
- [ ] Metriques par session (tokens/phase, temps/agent)
- [ ] Auto-generation des session logs dans `.memory/sessions/`
- [ ] Dashboard au startup (resume projet, etat workflow)
- [ ] Notifications OS pour gates et taches longues
- [ ] Raccourcis clavier configurables
- [ ] Theme system

---

## Compteur de progression

| Phase | Total | Done | Stub | TODO | % |
|-------|-------|------|------|------|---|
| 1.1 Backend Rust | 15 | 14 | 0 | 0 | 93% |
| 1.2 Frontend React | 24 | 21 | 3 | 0 | 88% |
| 1.3 Integration | 8 | 8 | 0 | 0 | 100% |
| **Phase 1 Total** | **47** | **43** | **3** | **0** | **91%** |
| Phase 2 | 13 | 12 | 0 | 1 | 92% |
| Phase 3 | 8 | 0 | 0 | 8 | 0% |
| Phase 4 | 7 | 0 | 0 | 7 | 0% |
| Phase 5 | 7 | 0 | 0 | 7 | 0% |
| **Total** | **82** | **55** | **3** | **24** | **67%** |
