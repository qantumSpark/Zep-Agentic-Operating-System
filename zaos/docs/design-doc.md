# ZAOS — Zep Agentic OS

## Design Document v1.0

**Date :** 29 mars 2026
**Auteur :** Antoine + Claude
**Statut :** Draft — en attente de validation

---

## 1. Vision

ZAOS est une application desktop qui fournit une interface visuelle pour piloter un workflow de développement de jeux Godot assisté par IA. Elle remplace l'expérience purement terminal de Claude Code par un environnement split-panel : un chat agentique à gauche, un dashboard de suivi en temps réel à droite.

ZAOS n'est **pas** un nouveau framework agentique. C'est une **couche de visualisation et de contrôle** au-dessus de Claude Code / Claude Agent SDK, qui rend visible et manipulable ce qui se passe aujourd'hui dans le terminal.

### 1.1 Objectifs

- Rendre le workflow agentique **observable** : voir en temps réel le plan, les étapes, les actions, les screenshots
- Rendre le workflow **contrôlable** : pause, reprise, validation de gates, override d'actions
- **Conserver l'écosystème existant** : GoPeak (MCP Godot), système de workflow (.workflow/), mémoire (.memory/), agents (.claude/agents/)
- **Consommer le plan Max 5x** d'Anthropic, pas des crédits API à la consommation

### 1.2 Non-objectifs (v1)

- Remplacer l'éditeur Godot
- Remplacer un IDE pour l'édition de code (pas de LSP, pas d'autocomplétion)
- Gérer plusieurs projets simultanément
- Être un produit distribué à d'autres utilisateurs
- Intégrer Computer Use (beta macOS uniquement — l'interaction Godot passe par GoPeak via MCP)

---

## 2. Architecture

### 2.1 Vue d'ensemble

```
┌──────────────────────────────────────────────────────┐
│                     ZAOS (Tauri)                      │
│                                                       │
│  ┌──────────────────┬───────────────────────┐        │
│  │                  │                        │        │
│  │   Chat Panel     │   Dashboard Panel      │        │
│  │   (React)        │   (React)              │        │
│  │                  │                        │        │
│  └────────┬─────────┴──────────┬────────────┘        │
│           │    WebView (Frontend)│                     │
│  ─────────┼─────────────────────┼─────────────────── │
│           │    Tauri Core (Rust) │                     │
│  ┌────────┴─────────────────────┴────────────────┐   │
│  │            ZAOS Backend (Rust)                 │   │
│  │                                                │   │
│  │  ┌──────────────┐  ┌───────────────────────┐  │   │
│  │  │ Session      │  │ Event Bus             │  │   │
│  │  │ Manager      │  │ (parse JSON stream    │  │   │
│  │  │              │  │  → broadcast to UI)   │  │   │
│  │  └──────┬───────┘  └───────────────────────┘  │   │
│  │         │                                      │   │
│  │  ┌──────┴───────┐  ┌───────────────────────┐  │   │
│  │  │ Workflow     │  │ zaos-ide MCP Server   │  │   │
│  │  │ Engine       │  │ (127.0.0.1:random)    │  │   │
│  │  └──────────────┘  │ diffs, notifs, state  │  │   │
│  │                     └──────────┬────────────┘  │   │
│  │  ┌──────────────┐             │               │   │
│  │  │ Screenshot   │             │               │   │
│  │  │ Manager      │             │               │   │
│  │  └──────────────┘             │               │   │
│  └─────────┬─────────────────────┘               │   │
│            │                                      │   │
└────────────┼──────────────────────────────────────┘   │
             │                                          │
             │  stdin/stdout (stream-json)               │
             │                                          │
┌────────────┴──────────────────────────┐              │
│  Claude Code CLI                      │              │
│  (processus local, auth OAuth Max 5x) │              │
│                                       │              │
│  Connecté aux MCP servers :           │              │
│  ┌─────────────────┐                  │              │
│  │ GoPeak          │──────────────────┼──── Godot Engine
│  │ (95+ tools)     │                  │
│  └─────────────────┘                  │
│  ┌─────────────────┐                  │
│  │ zaos-ide        │◄─── (déclaré dans .mcp.json)
│  │ (diffs, notifs) │                  │
│  └─────────────────┘                  │
│                                       │
│  Accès filesystem :                   │
│  .workflow/ .memory/ GAME/            │
└───────────────────────────────────────┘
```

### 2.2 Couches

| Couche | Technologie | Responsabilité |
|--------|------------|----------------|
| **Frontend** | React + TypeScript + Tailwind | UI split-panel, chat, dashboard, screenshots |
| **Bridge** | Tauri IPC (invoke/events) | Communication bidirectionnelle frontend ↔ backend |
| **Backend** | Rust (Tauri core) | Gestion du processus Claude Code CLI, parsing d'événements, workflow engine, file watching |
| **Agent Engine** | Claude Code CLI (processus local) | Exécution des prompts, tool use, streaming — authentifié via abonnement Max |
| **MCP local** | Serveur MCP "zaos-ide" (inspiré du MCP "ide" de VS Code) | Permet au CLI d'interagir avec le dashboard (diffs, screenshots, notifications) |
| **MCP Godot** | GoPeak (npx gopeak) | Interface avec Godot Engine (screenshots, input, scenes, debug) |

### 2.3 Décision : Wrapper CLI (architecture VS Code)

ZAOS adopte **exactement la même architecture que l'extension VS Code officielle** d'Anthropic : un wrapper graphique autour du processus Claude Code CLI.

#### Pourquoi ce choix

L'extension VS Code ne passe pas par le Claude Agent SDK ni par l'API directe. Elle spawn un processus Claude Code CLI local et communique avec lui. Ce CLI s'authentifie via l'OAuth de l'abonnement (Pro/Max/Team/Enterprise), ce qui est officiellement supporté et garanti par Anthropic. L'usage est facturé sur le plan Max de l'utilisateur, pas à la consommation API.

Le Claude Agent SDK a une position ambiguë sur ce point : la doc officielle interdit aux développeurs tiers d'utiliser l'authentification claude.ai, mais un employé d'Anthropic a clarifié en mars 2026 que rien ne change pour l'usage personnel avec Max. Plutôt que de dépendre d'une zone grise, ZAOS utilise la méthode éprouvée du CLI wrapper.

#### Mécanisme

```
ZAOS Backend (Rust)
    │
    ├── Spawn: `claude --output-format stream-json [options]`
    │   Le CLI utilise l'authentification OAuth Max déjà configurée
    │   sur la machine (~/.claude/credentials)
    │
    ├── stdin  → envoi des prompts et commandes
    ├── stdout → réception du flux d'événements JSON (streaming)
    └── stderr → logs et erreurs
```

Le CLI est lancé en mode non-interactif avec `--output-format stream-json` qui produit un flux d'événements JSON structurés (un objet par ligne). Chaque événement contient un type (text, tool_use, tool_result, error, etc.) et les données associées.

#### Serveur MCP local "zaos-ide"

En plus du CLI, ZAOS fait tourner un petit serveur MCP local (sur 127.0.0.1, port aléatoire, token d'auth éphémère) — exactement comme le fait l'extension VS Code avec son MCP "ide". Ce serveur permet au CLI Claude Code d'interagir avec l'interface ZAOS :

- Ouvrir un diff visuel dans le dashboard quand Claude modifie un fichier
- Afficher un screenshot capturé par GoPeak directement dans le chat
- Déclencher une notification système quand un gate attend validation
- Fournir au CLI des informations sur l'état de l'UI (quel fichier est visible, quelle section du dashboard est ouverte)

Ce MCP est déclaré dans la configuration du projet (`.mcp.json`) aux côtés de GoPeak.

#### Fallback : Claude Agent SDK

Si à l'avenir le SDK supporte officiellement le billing Max (issue #559 sur le repo), ZAOS pourra basculer sur le SDK pour une intégration plus propre. L'architecture est prévue pour que le Session Manager abstrait la source (CLI ou SDK) derrière une interface commune.

---

## 3. Interface utilisateur

### 3.1 Layout principal

```
┌──────────────────────────────────────────────────────────────┐
│  ZAOS — Expanse 4X                          [⚙] [🔄] [—][×] │
├────────────────────────┬─────────────────────────────────────┤
│                        │  ┌─ Workflow ──────────────────────┐│
│  Chat                  │  │ Epic: Système de combat         ││
│                        │  │ Phase: implementation ● (3/7)   ││
│  ┌──────────────────┐  │  │ Mode: pipeline                  ││
│  │ 🤖 J'ai analysé  │  │  │ Task: HitBox component          ││
│  │ la scene Player  │  │  └─────────────────────────────────┘│
│  │ et je propose... │  │                                     │
│  └──────────────────┘  │  ┌─ Pipeline ──────────────────────┐│
│                        │  │                                  ││
│  ┌──────────────────┐  │  │ ✅ comprehension                ││
│  │ 👤 Ok, lance     │  │  │ ✅ specification                ││
│  │ l'implémentation │  │  │ ✅ architecture                 ││
│  └──────────────────┘  │  │ 🔄 implementation  ← en cours   ││
│                        │  │ ⬜ review                        ││
│  ┌──────────────────┐  │  │ ⬜ test                          ││
│  │ 🤖 [Coder] Je    │  │  │ ⬜ closure                      ││
│  │ crée le fichier  │  │  │                                  ││
│  │ hitbox.gd...     │  │  │ [Valider Gate ▶]                ││
│  │                  │  │  └─────────────────────────────────┘│
│  │ ```gdscript      │  │                                     │
│  │ extends Area2D   │  │  ┌─ Actions (live) ────────────────┐│
│  │ ...              │  │  │                                  ││
│  │ ```              │  │  │ 14:32:01 📂 Write hitbox.gd     ││
│  └──────────────────┘  │  │ 14:32:03 🖥️ shell: godot --run  ││
│                        │  │ 14:32:08 📸 screenshot captured  ││
│  ┌──────────────────┐  │  │ 14:32:09 ✅ validation OK       ││
│  │ 🤖 [Playtester]  │  │  │                                  ││
│  │ Screenshot du    │  │  └─────────────────────────────────┘│
│  │ résultat :       │  │                                     │
│  │ [📸 image]       │  │  ┌─ Agents ────────────────────────┐│
│  └──────────────────┘  │  │ 🟢 Orchestrateur (actif)        ││
│                        │  │ 🟢 Coder (délégation en cours)  ││
│                        │  │ ⚪ Reviewer (en attente)         ││
│                        │  │ ⚪ Tester (en attente)           ││
│ ┌────────────────────┐ │  │ 🟢 Playtester (actif)           ││
│ │ > Message...   [⏎] │ │  └─────────────────────────────────┘│
│ └────────────────────┘ │                                     │
├────────────────────────┴─────────────────────────────────────┤
│  📊 Tokens: 42.3k/200k  │  ⏱ Session: 47min  │  🔗 GoPeak ✅ │
└──────────────────────────────────────────────────────────────┘
```

### 3.2 Panneau gauche — Chat

Le chat reproduit l'expérience Claude Code mais en mode graphique.

**Éléments :**
- Bulles de messages (utilisateur / assistant / système)
- Streaming en temps réel du texte
- Blocs de code avec coloration syntaxique (GDScript, Python, Shell)
- Images inline (screenshots GoPeak)
- Indicateur d'agent actif (badge `[Coder]`, `[Reviewer]`, etc.)
- Indicateur de "thinking" (quand Claude réfléchit avant de répondre)
- Boutons d'action contextuels : « Approuver », « Rejeter », « Modifier » sur les propositions de plan ou de code

**Input :**
- Zone de texte multiline avec Shift+Enter pour nouvelle ligne, Enter pour envoyer
- Bouton pause/stop pour interrompre une exécution longue
- Raccourcis : Ctrl+Enter pour envoyer, Escape pour annuler

### 3.3 Panneau droit — Dashboard

Le dashboard est divisé en sections collapsibles, redimensionnables.

#### 3.3.1 Section Workflow

Affiche l'état courant lu depuis `.workflow/state.json` :
- **Epic** en cours (nom + description courte)
- **Phase** active (avec indicateur de progression dans le pipeline)
- **Mode** (free / pipeline) avec toggle pour changer
- **Task** en cours (description)
- **Bouton Gate** : valider le gate de la phase en cours (déclenche `wfctl.sh gate validate`)

#### 3.3.2 Section Pipeline

Visualisation verticale du pipeline de phases :
- `idle → comprehension → specification → architecture → implementation → review → test → closure`
- Chaque phase a un état : ✅ terminée, 🔄 en cours, ⬜ à venir
- Cliquer sur une phase terminée montre son résumé (output de l'agent qui l'a traitée)
- Le raccourci de pipeline (bug fix = comprehension → implementation → review → closure) est visuellement affiché quand applicable

#### 3.3.3 Section Actions (live feed)

Log chronologique en temps réel de chaque action exécutée :
- Type d'action : 📂 Write, ✏️ Edit, 🖥️ Shell, 📸 Screenshot, 🔍 Read, 🌐 Web, 💬 Delegation
- Timestamp
- Résumé court (nom du fichier, commande exécutée, etc.)
- Statut : ✅ succès, ❌ erreur, 🔄 en cours
- Cliquer sur une action ouvre le détail (contenu du fichier, output de commande, image du screenshot)

#### 3.3.4 Section Agents

Affiche les agents du système et leur état :
- Nom de l'agent + fichier source (`.claude/agents/coder.md`)
- État : 🟢 actif, ⚪ inactif, 🔴 erreur
- Historique des délégations récentes

#### 3.3.5 Section Screenshots (optionnelle, collapsible)

Galerie des screenshots capturés par GoPeak durant la session :
- Miniatures chronologiques
- Cliquer pour agrandir
- Comparaison avant/après possible

### 3.4 Barre de statut

Barre fixe en bas avec :
- **Compteur de tokens** : consommation de la session en cours (input + output)
- **Durée de session**
- **État des connexions** : Claude Code (✅/❌), GoPeak (✅/❌), Godot (✅/❌)
- **Indicateur de mode** : free / pipeline

### 3.5 Interactions entre panneaux

- Quand une action apparaît dans le feed (panneau droit), le message correspondant dans le chat (panneau gauche) est mis en surbrillance si visible
- Quand un gate est validé via le bouton du dashboard, un message système apparaît dans le chat
- Quand un screenshot apparaît dans le feed, il est aussi inséré inline dans le chat à côté du message de l'agent qui l'a demandé
- Le changement de mode (free/pipeline) via le dashboard met à jour state.json et injecte une notification dans le chat

---

## 4. Backend — Composants Rust

### 4.1 Session Manager

Responsable du cycle de vie du processus Claude Code CLI. Inspiré directement de l'architecture de l'extension VS Code officielle.

```
SessionManager
├── spawn_cli(project_path, config) → ChildProcess
│   Exécute: `claude --output-format stream-json --project-dir <path>`
│   Le CLI utilise l'auth OAuth Max déjà configurée sur la machine
├── send_prompt(text) → Stream<Event>
│   Écrit le prompt sur stdin du processus
├── send_resume(session_id, text) → Stream<Event>
│   Reprend une session existante (--resume <id>)
├── interrupt() → Result
│   Envoie SIGINT au processus CLI
├── get_session_info() → SessionInfo { tokens_used, duration, model }
├── list_sessions() → Vec<SessionSummary>
│   Lit ~/.claude/projects/<encoded-cwd>/*.jsonl
└── on_event(callback: Fn(Event))
```

**Cycle de vie :**
1. Au lancement de ZAOS, vérifier que `claude` est dans le PATH et authentifié (`claude --version`)
2. Au premier message, spawn le processus CLI avec les flags appropriés
3. Chaque ligne JSON sur stdout est parsée en `AgentEvent` et broadcastée au frontend
4. En cas de timeout ou crash du CLI, proposer un re-spawn automatique
5. Les sessions sont persistées par le CLI lui-même dans `~/.claude/projects/`

**Compatibilité :** L'utilisateur peut continuer à utiliser Claude Code en terminal ou dans VS Code en parallèle. Les sessions sont partagées via le filesystem (`~/.claude/projects/`).

### 4.2 Event Parser

Parse le flux d'événements de Claude Code et les classifie.

```rust
enum AgentEvent {
    // Chat
    TextDelta { content: String, agent: Option<String> },
    ThinkingStart,
    ThinkingEnd,
    
    // Actions
    ToolUseStart { tool: String, input: Value },
    ToolUseEnd { tool: String, output: Value, success: bool },
    FileWrite { path: String, content: String },
    FileEdit { path: String, diff: String },
    ShellCommand { command: String, output: String, exit_code: i32 },
    Screenshot { path: String, base64: Option<String> },
    
    // Workflow
    PhaseChange { from: String, to: String },
    GateValidation { phase: String, approved: bool },
    AgentDelegation { from: String, to: String, task: String },
    
    // Système
    TokenUsage { input: u64, output: u64, cache_hit: f32 },
    Error { message: String, recoverable: bool },
    SessionEnd { reason: String },
}
```

### 4.3 Workflow Engine

Lit et modifie l'état du workflow. Remplace progressivement `wfctl.sh` par une implémentation native Rust.

```
WorkflowEngine
├── load_state(project_path) → WorkflowState
├── get_phase() → Phase
├── set_phase(phase) → Result
├── validate_gate() → Result<bool>
├── next_phase() → Result<Phase>
├── set_mode(mode: Free | Pipeline) → Result
├── set_epic(name, description) → Result
├── set_task(description) → Result
├── get_pipeline_for_task_type(type) → Vec<Phase>
└── watch_state_file() → Stream<WorkflowState>  // file watcher
```

**Amélioration par rapport au système actuel :**
- `wfctl.sh` reste disponible comme fallback CLI
- Le Workflow Engine Rust lit/écrit le même `state.json` (compatibilité bidirectionnelle)
- Ajout d'un **historique des transitions** (state.json actuel ne garde que l'état courant)
- Ajout de **timestamps** sur chaque transition de phase
- Ajout d'un **estimation de progression** par phase (basée sur l'historique des sessions passées)

### 4.4 Screenshot Manager

Gère les screenshots capturés via GoPeak.

```
ScreenshotManager
├── watch_directory(path) → Stream<Screenshot>
├── capture_on_demand() → Screenshot  // demande via GoPeak MCP
├── compare(before, after) → DiffResult
├── get_gallery(session_id) → Vec<Screenshot>
└── cleanup_old(max_age: Duration)
```

### 4.5 Memory Reader

Lit les fichiers `.memory/` pour afficher le contexte dans le dashboard.

```
MemoryReader
├── read_index() → MemoryIndex
├── read_state() → ProjectState
├── read_current_epic() → Option<Epic>
├── read_session_log(date) → Option<SessionLog>
└── watch_changes() → Stream<MemoryChange>
```

---

## 5. Flux de données

### 5.1 Envoi d'un message

```
[Utilisateur tape un message]
    │
    ▼
[Frontend] ──invoke("send_prompt", text)──▶ [Tauri Backend]
    │                                              │
    │                                    [Session Manager]
    │                                         │
    │                                    écrit le prompt sur stdin
    │                                    du processus `claude` CLI
    │                                         │
    │                                    [Claude Code CLI]
    │                                    (auth OAuth Max)
    │                                    exécute, appelle GoPeak,
    │                                    filesystem, subagents...
    │                                         │
    │                                    stdout: flux JSON
    │                                    (1 objet JSON par ligne)
    │                                         │
    │                                    [Event Parser]
    │                                    parse chaque ligne JSON
    │                                    → classifie en AgentEvent
    │                                         │
    │                               ┌─────────┼─────────┐
    │                               │         │         │
    │                          TextDelta  ToolUse  PhaseChange
    │                               │         │         │
    ◀──────── tauri::emit ──────────┴─────────┴─────────┘
    │
[Frontend]
    ├── Chat: affiche le texte en streaming
    ├── Actions feed: ajoute l'action
    ├── Pipeline: met à jour la phase si besoin
    └── Agents: met à jour l'état de l'agent actif
```

### 5.2 Validation d'un gate

```
[Utilisateur clique "Valider Gate"]
    │
    ▼
[Frontend] ──invoke("validate_gate")──▶ [Workflow Engine]
    │                                          │
    │                                  met à jour state.json
    │                                  passe en phase suivante
    │                                          │
    │                                  [Event: GateValidation]
    │                                  [Event: PhaseChange]
    │                                          │
    ◀────────── tauri::emit ───────────────────┘
    │
[Frontend]
    ├── Pipeline: phase suivante activée
    ├── Chat: message système "Gate validé, passage en phase X"
    └── Workflow section: mis à jour
```

---

## 6. Stack technique

### 6.1 Détail

| Composant | Choix | Justification |
|-----------|-------|---------------|
| **Shell** | Tauri v2 | Léger (~10MB), backend Rust natif, WebView système |
| **Frontend** | React 19 + TypeScript | Écosystème riche, composants disponibles |
| **Styling** | Tailwind CSS | Rapide, cohérent, pas de CSS custom à maintenir |
| **State management** | Zustand | Léger, simple, pas de boilerplate Redux |
| **Code highlighting** | Shiki | Support GDScript, rendu rapide |
| **Markdown rendering** | react-markdown + remark-gfm | Pour le rendu des messages Claude |
| **Backend** | Rust (Tauri core) | Performance, gestion de processus, file watching |
| **Sérialisation** | serde + serde_json | Standard Rust pour JSON |
| **File watching** | notify (crate Rust) | Surveiller state.json, .memory/, screenshots |
| **Process management** | tokio::process | Async spawn/communication avec Claude Code |
| **IPC** | Tauri commands + events | Frontend ↔ Backend bidirectionnel |

### 6.2 Structure du projet

```
zaos/
├── src-tauri/                    # Backend Rust
│   ├── src/
│   │   ├── main.rs               # Entry point Tauri
│   │   ├── session/
│   │   │   ├── mod.rs
│   │   │   ├── manager.rs        # Session Manager (spawn CLI, gère stdin/stdout)
│   │   │   ├── cli_wrapper.rs    # Wrapper Claude Code CLI (stream-json parsing)
│   │   │   └── auth_check.rs     # Vérification auth OAuth Max au démarrage
│   │   ├── events/
│   │   │   ├── mod.rs
│   │   │   ├── parser.rs         # Event Parser (JSON stream → AgentEvent)
│   │   │   └── types.rs          # AgentEvent enum
│   │   ├── mcp_server/
│   │   │   ├── mod.rs
│   │   │   ├── server.rs         # zaos-ide MCP server (127.0.0.1:random)
│   │   │   ├── tools.rs          # Tools exposés au CLI (show_diff, notify, etc.)
│   │   │   └── auth.rs           # Token éphémère pour auth locale
│   │   ├── workflow/
│   │   │   ├── mod.rs
│   │   │   ├── engine.rs         # Workflow Engine
│   │   │   ├── state.rs          # State types
│   │   │   └── phases.rs         # Phase definitions
│   │   ├── screenshots/
│   │   │   ├── mod.rs
│   │   │   └── manager.rs        # Screenshot Manager
│   │   ├── memory/
│   │   │   ├── mod.rs
│   │   │   └── reader.rs         # Memory Reader
│   │   └── commands.rs           # Tauri IPC commands
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                          # Frontend React
│   ├── App.tsx
│   ├── components/
│   │   ├── chat/
│   │   │   ├── ChatPanel.tsx
│   │   │   ├── MessageBubble.tsx
│   │   │   ├── CodeBlock.tsx
│   │   │   ├── ScreenshotInline.tsx
│   │   │   └── InputBar.tsx
│   │   ├── dashboard/
│   │   │   ├── DashboardPanel.tsx
│   │   │   ├── WorkflowSection.tsx
│   │   │   ├── PipelineSection.tsx
│   │   │   ├── ActionsFeed.tsx
│   │   │   ├── AgentsSection.tsx
│   │   │   └── ScreenshotGallery.tsx
│   │   ├── statusbar/
│   │   │   └── StatusBar.tsx
│   │   └── common/
│   │       ├── SplitPane.tsx
│   │       └── CollapsibleSection.tsx
│   ├── stores/
│   │   ├── chatStore.ts          # Messages, streaming state
│   │   ├── workflowStore.ts      # Phases, gates, mode
│   │   ├── actionsStore.ts       # Action feed
│   │   └── sessionStore.ts       # Tokens, duration, connections
│   ├── hooks/
│   │   ├── useTauriEvents.ts     # Écoute des events Tauri
│   │   └── useStreaming.ts       # Gestion du streaming texte
│   └── types/
│       ├── events.ts             # Types miroir de AgentEvent
│       └── workflow.ts           # Types workflow
├── package.json
└── README.md
```

---

## 7. Workflow amélioré

### 7.1 Compatibilité

ZAOS lit et écrit le même format `state.json` que `wfctl.sh`. Les deux outils peuvent coexister :
- Utiliser ZAOS quand on veut l'interface visuelle
- Utiliser `wfctl.sh` dans le terminal quand on veut aller vite
- L'état est toujours synchronisé via le fichier

### 7.2 Améliorations prévues

| Fonctionnalité actuelle | Amélioration ZAOS |
|------------------------|-------------------|
| Phase courante uniquement dans state.json | + Historique horodaté de toutes les transitions |
| Pas de durée par phase | + Tracking du temps passé par phase |
| Pas de métriques | + Tokens consommés par phase, par agent |
| Gates validés en CLI | + Bouton visuel avec confirmation |
| Mode free/pipeline en CLI | + Toggle visuel avec feedback immédiat |
| Logs de session manuels (.memory/sessions/) | + Génération automatique du log en fin de session |
| Pas de résumé inter-sessions | + Dashboard d'accueil avec état du projet au lancement |

### 7.3 Format state.json étendu

```json
{
  "phase": "implementation",
  "mode": "pipeline",
  "epic": "Système de combat",
  "task": "HitBox component",
  "gate_validated": false,
  
  "// Nouveaux champs ZAOS": "",
  "history": [
    {
      "phase": "comprehension",
      "started_at": "2026-03-29T14:00:00Z",
      "ended_at": "2026-03-29T14:12:00Z",
      "agent": "orchestrator",
      "tokens": { "input": 8200, "output": 1400 }
    },
    {
      "phase": "specification",
      "started_at": "2026-03-29T14:12:00Z",
      "ended_at": "2026-03-29T14:25:00Z",
      "agent": "architect",
      "tokens": { "input": 12000, "output": 3200 }
    }
  ],
  "session": {
    "started_at": "2026-03-29T14:00:00Z",
    "total_tokens": { "input": 42300, "output": 8900 },
    "screenshots_count": 7
  }
}
```

Champs existants préservés, nouveaux champs ajoutés. `wfctl.sh` ignore les champs qu'il ne connaît pas → rétrocompatible.

---

## 8. Phases MVP

### Phase 1 — Chat fonctionnel (semaines 1-3)

**Objectif :** Pouvoir envoyer des messages à Claude Code et recevoir les réponses en streaming dans l'interface ZAOS.

**Livrables :**
- **Semaine 1 — Investigation CLI :** Capturer et documenter le format stream-json du CLI (`claude --output-format stream-json`). Identifier tous les types d'événements JSON, leur structure, et les edge cases. Étudier le code source de l'extension VS Code comme référence.
- **Semaine 1-2 — App Tauri minimale :** Shell Tauri avec split-panel (chat à gauche, placeholder à droite). Backend Rust qui spawn le CLI, lit stdout ligne par ligne, parse le JSON.
- **Semaine 2-3 — Chat complet :** Streaming texte en temps réel, coloration syntaxique des blocs de code, zone d'input avec envoi (Enter) + interruption (Ctrl+C → SIGINT), indicateur de "thinking", resume de sessions existantes.

**Critère de succès :** L'expérience de chat est au moins aussi bonne que le terminal Claude Code pour les conversations texte simples.

### Phase 2 — Dashboard temps réel (semaines 4-6)

**Objectif :** Visualiser les actions et le workflow en temps réel.

**Livrables :**
- Event Parser complet (tous les types d'événements)
- Section Workflow (lecture de state.json + file watching)
- Section Pipeline (phases avec états visuels)
- Section Actions feed (log chronologique live)
- Section Agents (état des agents)
- Bouton Gate avec validation
- Barre de statut (tokens, durée, connexions)

**Critère de succès :** En regardant le dashboard, on comprend instantanément ce que Claude Code est en train de faire, dans quelle phase du workflow, avec quel agent.

### Phase 3 — Screenshots (semaines 7-8)

**Objectif :** Visualiser les screenshots Godot capturés par GoPeak.

**Livrables :**
- Screenshot Manager (file watching sur le répertoire de captures)
- Galerie dans le dashboard
- Insertion inline des screenshots dans le chat
- Zoom / comparaison avant-après

**Critère de succès :** Les screenshots apparaissent automatiquement dans l'interface dès que GoPeak les capture.

### Phase 4 — Améliorations UX et workflow (semaines 9-12)

**Objectif :** Polir l'expérience et enrichir le workflow.

**Livrables :**
- Historique des sessions : navigation, recherche, resume depuis le dashboard
- Métriques de session (tokens par phase, temps par agent, graphiques simples)
- Génération automatique du log de session en fin de cycle (écriture dans `.memory/sessions/`)
- Dashboard d'accueil au lancement (résumé projet, état workflow, dernière session)
- Notifications système OS quand un gate attend validation ou qu'une tâche longue est terminée
- Raccourcis clavier configurables

**Critère de succès :** ZAOS est l'outil principal pour travailler sur le projet Godot, plus confortable que le terminal.

---

### Roadmap future (hors v1)

**Computer Use :** Actuellement en beta, disponible uniquement sur macOS. ZAOS est développé pour Windows/Linux. L'architecture est prévue pour intégrer Computer Use quand Anthropic le rendra disponible sur ces plateformes. En attendant, toute l'interaction avec Godot (screenshots, input injection, scene management, debug) passe par GoPeak via MCP, qui couvre ces besoins.

Quand Computer Use sera disponible :
- Affichage des actions Computer Use dans le feed (coordonnées du clic, touches pressées)
- Overlay visuel sur les screenshots montrant où Claude a cliqué
- Contrôles pause/reprise pour les séquences d'actions automatisées

---

## 9. Risques et questions ouvertes

### 9.1 Risques techniques

| Risque | Impact | Mitigation |
|--------|--------|------------|
| Format stream-json du CLI change entre versions | Parsing cassé, events manqués | Pinning de version Claude Code, tests de non-régression sur traces capturées, mode debug avec log brut visible |
| Performance Tauri avec streaming haute fréquence | Lag dans le chat ou le feed | Throttling des events UI (batch de 16ms), virtualisation des listes longues |
| CLI Claude Code crash ou timeout sur tâches longues | Session perdue | Détection du crash + proposition de resume automatique via `--resume <session_id>` |
| GoPeak breaking changes | Rupture de l'intégration screenshots/actions | Pinning de version, couche d'abstraction MCP |
| Taille du contexte (state.json + history) | Fichier qui grossit indéfiniment | Rotation de l'historique (garder les N dernières sessions) |
| Anthropic modifie les conditions d'utilisation du CLI avec abonnement Max | Usage personnel toujours OK, mais à surveiller | Surveiller les annonces. Fallback : Claude Agent SDK si le support Max est officialisé (issue #559) |

### 9.2 Questions ouvertes

1. **Format exact de sortie de Claude Code en mode stream-json** — Le format JSON streamé par `claude --output-format stream-json` n'est pas documenté publiquement par Anthropic. Il faut le reverse-engineer en capturant des traces réelles. L'extension VS Code officielle (open-source, sur le marketplace) fait exactement ça — son code source est une référence directe pour comprendre le protocole. Première tâche de la Phase 1 : capturer et documenter les types d'événements JSON produits par le CLI.

2. **Flags CLI exacts** — Documenter la combinaison exacte de flags pour le mode non-interactif : `--output-format stream-json`, `--print`, `--project-dir`, `--resume`, `--model`, etc. Certains flags peuvent ne pas être combinables.

3. **Stockage des sessions** — Faut-il une base de données (SQLite) pour l'historique des actions et métriques, ou les fichiers JSON suffisent pour un usage personnel ?

4. **Multi-fenêtre** — Est-ce qu'on veut pouvoir détacher le panneau screenshots ou la galerie dans une fenêtre séparée (utile avec un 2e écran) ?

5. **Thème** — Dark mode par défaut (cohérent avec les IDE), mais prévoir un système de thèmes ?

6. **Notifications système** — Quand Claude termine une tâche longue ou quand un gate attend validation, envoyer une notification OS ?

7. **Coexistence avec VS Code** — Si l'utilisateur a aussi l'extension VS Code, les deux outils pourraient spawner des processus CLI concurrents. Vérifier que ça ne pose pas de conflit (sessions, locks, rate limiting Max).

---

## 10. Inspirations

| Projet | Ce qu'on prend | Ce qu'on ne prend pas |
|--------|---------------|----------------------|
| **Claude Code extension VS Code** | **Architecture de référence** : CLI wrapper via stdin/stdout, MCP local "ide" pour communication bidirectionnelle, auth OAuth Max, inline diffs, streaming chat | Le fait d'être un plugin d'un IDE existant |
| **Manus** | Le dashboard de plan/étapes en temps réel, la visualisation du navigateur | L'architecture cloud, les VMs dédiées |
| **OpenHands** | Le split-panel chat + workspace | La complexité du setup Docker |
| **Cursor** | La fluidité du streaming et l'UX | Le focus IDE, pas dashboard |

---

## Annexe A — Commandes Tauri IPC

```typescript
// Frontend → Backend
invoke("send_prompt", { text: string })
invoke("resume_session", { session_id: string, text: string })
invoke("interrupt_session")                    // SIGINT au processus CLI
invoke("validate_gate")
invoke("set_mode", { mode: "free" | "pipeline" })
invoke("set_phase", { phase: string })
invoke("get_workflow_state") → WorkflowState
invoke("get_session_info") → SessionInfo
invoke("list_sessions") → SessionSummary[]     // lit ~/.claude/projects/
invoke("open_project", { path: string })
invoke("check_cli_auth") → AuthStatus          // vérifie que claude est installé + auth OK

// Backend → Frontend (events)
listen("agent-event", (event: AgentEvent) => void)
listen("workflow-change", (state: WorkflowState) => void)
listen("screenshot-captured", (screenshot: Screenshot) => void)
listen("connection-status", (status: ConnectionStatus) => void)
listen("cli-health", (status: { alive: bool, pid: number }) => void)
```

---

## Annexe B — Dépendances externes requises

- **Rust** ≥ 1.75 (toolchain stable)
- **Node.js** ≥ 18 (pour le frontend React + pour GoPeak)
- **Claude Code CLI** installé et authentifié (session Max active)
- **GoPeak** (`npx gopeak`) configuré avec le chemin Godot
- **Godot Engine** 4.x installé
- **OS** : Windows 10/11 (cible principale), Linux (cible secondaire). macOS non ciblé en v1.

---

*Document vivant — sera mis à jour au fil des décisions de conception.*
