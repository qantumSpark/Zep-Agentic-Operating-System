# Protocole stream-json — Claude Code CLI

> Reverse-engineering du format de sortie `claude --output-format stream-json`
> CLI version : 2.1.85
> Date de capture : 29 mars 2026
> Traces réelles capturées dans `reference/traces/`

---

## 1. Commande et modes

### 1.1 Mode one-shot (--print)

```bash
claude -p "prompt" --output-format stream-json --verbose
```

- `--verbose` est **obligatoire** avec `--print` + `stream-json`
- Produit un flux JSONL (un objet JSON par ligne)
- Le CLI se termine après la réponse complète

### 1.2 Mode interactif (ZAOS utilisera celui-ci)

```bash
claude --output-format stream-json
```

- Le CLI reste actif, attend des prompts sur stdin
- Produit le même flux JSONL sur stdout
- Sessions persistées dans `~/.claude/projects/<encoded-cwd>/`

### 1.3 Le flag critique : --include-partial-messages

**DÉCOUVERTE CLÉ :** Sans `--include-partial-messages`, le texte arrive en un seul bloc `assistant`.
Avec ce flag, on reçoit des `stream_event` qui wrappent les événements API bruts (deltas texte, tool input, etc.).

**ZAOS DOIT utiliser `--include-partial-messages`** pour le streaming en temps réel.

| Flag combo | Events reçus | Streaming texte |
|------------|-------------|-----------------|
| `--verbose` seul | `system` → `assistant` (complet) → `result` | ❌ Bloc complet |
| `--verbose --include-partial-messages` | `system` → `stream_event`* → `assistant` → `result` | ✅ Deltas progressifs |

Séquence observée avec `--include-partial-messages` (trace réelle) :
```
system/init
stream_event > message_start
stream_event > content_block_start (text)
stream_event > content_block_delta (text_delta: "Les")
stream_event > content_block_delta (text_delta: " bugs dans la nuit\nS'effac")
stream_event > content_block_delta (text_delta: "ent ligne par ligne\nLe café refroidit")
assistant (message complet — snapshot à ce point)
stream_event > content_block_stop
stream_event > message_delta (stop_reason, usage)
stream_event > message_stop
rate_limit_event
result/success
```

Séquence avec tool use + `--include-partial-messages` (trace réelle) :
```
system/init
stream_event > message_start
stream_event > content_block_start (tool_use, name="Read")
stream_event > content_block_delta (input_json_delta: partial JSON...)  × N
assistant (tool_use complet — snapshot)
stream_event > content_block_stop
stream_event > message_delta
user (tool_result)
stream_event > message_stop
rate_limit_event
stream_event > message_start (turn 2)
stream_event > content_block_start (text)
stream_event > content_block_delta (text_delta) × N
assistant (text complet — snapshot)
stream_event > content_block_stop
stream_event > message_delta
stream_event > message_stop
result/success
```

### 1.4 Flags utiles

| Flag | Description |
|------|-------------|
| `--output-format stream-json` | Active le flux JSONL |
| `--verbose` | Obligatoire en mode `--print` |
| `--project-dir <path>` | Répertoire du projet |
| `--resume <session_id>` | Reprend une session existante |
| `--allowedTools "Tool1,Tool2"` | Restreint les outils disponibles |
| `--model <model>` | Force un modèle spécifique |
| `--bare` | Pas de chargement de contexte projet (CLAUDE.md, etc.) |

---

## 2. Types d'événements top-level

Chaque ligne JSON a un champ `type` qui détermine sa structure.

### Vue d'ensemble

```
Session lifecycle:
  system/init → [assistant|user|rate_limit_event]* → result/success|error

Cycle par turn:
  assistant (thinking) → assistant (tool_use) → user (tool_result) → rate_limit_event → assistant (text)
```

| Type | Quand | Fréquence |
|------|-------|-----------|
| `system` | Début de session | 1x |
| `assistant` | Chaque bloc de réponse Claude | Multiple par turn |
| `user` | Résultat d'un tool use | 1x par tool use |
| `rate_limit_event` | Après chaque appel API | 1x par turn |
| `result` | Fin de session/prompt | 1x |

---

## 3. Événement `system` (init)

Premier événement émis. Contient toutes les métadonnées de la session.

```json
{
  "type": "system",
  "subtype": "init",
  "cwd": "/chemin/du/projet",
  "session_id": "uuid-de-session",
  "tools": ["Task", "Bash", "Edit", "Read", "Write", "Grep", "Glob", ...],
  "mcp_servers": [],
  "model": "claude-sonnet-4-6",
  "permissionMode": "default",
  "slash_commands": ["compact", "context", "cost", ...],
  "apiKeySource": "none",
  "claude_code_version": "2.1.85",
  "output_style": "default",
  "agents": ["general-purpose", "Explore", "Plan", ...],
  "skills": ["docx", "pdf", "pptx", ...],
  "plugins": [],
  "uuid": "uuid-unique",
  "fast_mode_state": "off"
}
```

### Champs critiques pour ZAOS

| Champ | Usage ZAOS |
|-------|-----------|
| `session_id` | Identifier la session pour resume |
| `tools` | Afficher les outils disponibles |
| `mcp_servers` | Vérifier que GoPeak est connecté |
| `model` | Afficher le modèle dans la barre de statut |
| `claude_code_version` | Compatibilité, versionning |
| `agents` | Lister les agents disponibles |

### Autre subtype connu

```json
{
  "type": "system",
  "subtype": "api_retry",
  "attempt": 1,
  "max_retries": 3,
  "retry_delay_ms": 1000,
  "error_status": 429,
  "error": "rate_limit"
}
```

Erreurs possibles : `authentication_failed`, `billing_error`, `rate_limit`, `invalid_request`, `server_error`, `max_output_tokens`, `unknown`.

---

## 4. Événement `assistant`

Émis pour chaque bloc de contenu généré par Claude. Un même turn peut produire **plusieurs** événements `assistant` successifs.

### Structure commune

```json
{
  "type": "assistant",
  "message": {
    "model": "claude-sonnet-4-6",
    "id": "msg_...",
    "type": "message",
    "role": "assistant",
    "content": [ /* un ou plusieurs content blocks */ ],
    "stop_reason": null,
    "stop_sequence": null,
    "usage": {
      "input_tokens": 3,
      "cache_creation_input_tokens": 4880,
      "cache_read_input_tokens": 19177,
      "cache_creation": {
        "ephemeral_5m_input_tokens": 0,
        "ephemeral_1h_input_tokens": 4880
      },
      "output_tokens": 5,
      "service_tier": "standard",
      "inference_geo": "not_available"
    },
    "context_management": null
  },
  "parent_tool_use_id": null,
  "session_id": "uuid-session",
  "uuid": "uuid-unique"
}
```

### 4.1 Content block : `thinking`

```json
{
  "type": "thinking",
  "thinking": "Let me read the file.",
  "signature": "base64-signature..."
}
```

- **Usage ZAOS :** Afficher l'indicateur "réflexion en cours" dans le chat
- Le champ `signature` est une signature cryptographique (ignorer pour l'affichage)
- Le contenu `thinking` peut être affiché dans un bloc collapsible

### 4.2 Content block : `tool_use`

```json
{
  "type": "tool_use",
  "id": "toolu_...",
  "name": "Read",
  "input": {
    "file_path": "/chemin/du/fichier.md",
    "limit": 5
  },
  "caller": {
    "type": "direct"
  }
}
```

- **Usage ZAOS :**
  - Feed d'actions : ajouter l'action avec icône + résumé
  - Chat : afficher le tool use avec ses paramètres
  - Agents : `caller.type` peut indiquer si c'est un appel direct ou délégué

#### Outils observés et leur mapping ZAOS

| Tool name | Icône feed | Info à extraire |
|-----------|-----------|----------------|
| `Read` | 🔍 | `input.file_path` |
| `Write` | 📂 | `input.file_path` |
| `Edit` | ✏️ | `input.file_path` + `input.old_string` / `input.new_string` |
| `Bash` | 🖥️ | `input.command` |
| `Glob` | 🔎 | `input.pattern` |
| `Grep` | 🔎 | `input.pattern` + `input.path` |
| `Task` | 💬 | `input.description` (délégation agent) |
| `WebSearch` | 🌐 | `input.query` |
| `WebFetch` | 🌐 | `input.url` |
| `TodoWrite` | 📋 | `input.todos` |
| MCP tools | 🔧 | `name` commence par `mcp__gopeak__` pour GoPeak |

### 4.3 Content block : `text`

```json
{
  "type": "text",
  "text": "Voici ma réponse..."
}
```

- **Usage ZAOS :** Afficher dans le chat comme message de l'assistant
- Peut contenir du markdown (code blocks, liens, etc.)

### 4.4 Ordre typique des content blocks dans un turn multi-tool

```
assistant { content: [thinking] }        ← réflexion
assistant { content: [tool_use: Read] }  ← appel outil
user { content: [tool_result] }          ← résultat
rate_limit_event                         ← rate limit check
assistant { content: [text] }            ← réponse finale
```

**Important :** Chaque `assistant` event est un message **complet** à ce stade, pas un delta. Le `message.id` est le même pour tous les events d'un même turn API.

---

## 5. Événement `user` (tool_result)

Émis après l'exécution d'un tool par le CLI.

```json
{
  "type": "user",
  "message": {
    "role": "user",
    "content": [
      {
        "tool_use_id": "toolu_...",
        "type": "tool_result",
        "content": "contenu du résultat (texte brut)"
      }
    ]
  },
  "parent_tool_use_id": null,
  "session_id": "uuid-session",
  "uuid": "uuid-unique",
  "timestamp": "2026-03-29T12:21:37.512Z",
  "tool_use_result": {
    "type": "text",
    "file": {
      "filePath": "/chemin/du/fichier.md",
      "content": "contenu lu",
      "numLines": 5,
      "startLine": 1,
      "totalLines": 755
    }
  }
}
```

### Champs critiques pour ZAOS

| Champ | Usage |
|-------|-------|
| `tool_use_id` | Relier au `tool_use` correspondant dans l'event `assistant` |
| `tool_use_result.type` | Type de résultat (`text`, probablement d'autres) |
| `tool_use_result.file` | Métadonnées du fichier lu (pour l'affichage de diffs, preview) |
| `timestamp` | Horodatage de l'action dans le feed |
| `message.content[0].content` | Contenu brut du résultat (ce que Claude voit) |

---

## 6. Événement `rate_limit_event`

Émis après chaque appel API.

```json
{
  "type": "rate_limit_event",
  "rate_limit_info": {
    "status": "allowed",
    "resetsAt": 1774792800,
    "rateLimitType": "five_hour",
    "overageStatus": "rejected",
    "overageDisabledReason": "org_level_disabled",
    "isUsingOverage": false
  },
  "uuid": "uuid-unique",
  "session_id": "uuid-session"
}
```

### Usage ZAOS
- **Barre de statut :** Afficher le statut du rate limit
- **Alerte :** Si `status` ≠ `allowed`, prévenir l'utilisateur
- `resetsAt` : timestamp Unix de reset du rate limit (pour afficher "resets in X min")
- `rateLimitType`: `five_hour` correspond au plan Max 5x

---

## 7. Événement `result`

Dernier événement de la session/prompt. Contient les métriques finales.

```json
{
  "type": "result",
  "subtype": "success",
  "is_error": false,
  "duration_ms": 6526,
  "duration_api_ms": 6055,
  "num_turns": 2,
  "result": "texte final de la réponse",
  "stop_reason": "end_turn",
  "session_id": "uuid-session",
  "total_cost_usd": 0.0346977,
  "usage": {
    "input_tokens": 4,
    "cache_creation_input_tokens": 5176,
    "cache_read_input_tokens": 43269,
    "output_tokens": 153,
    "server_tool_use": {
      "web_search_requests": 0,
      "web_fetch_requests": 0
    },
    "service_tier": "standard",
    "cache_creation": {
      "ephemeral_1h_input_tokens": 5176,
      "ephemeral_5m_input_tokens": 0
    },
    "iterations": [],
    "speed": "standard"
  },
  "modelUsage": {
    "claude-sonnet-4-6": {
      "inputTokens": 4,
      "outputTokens": 153,
      "cacheReadInputTokens": 43269,
      "cacheCreationInputTokens": 5176,
      "webSearchRequests": 0,
      "costUSD": 0.0346977,
      "contextWindow": 200000,
      "maxOutputTokens": 32000
    }
  },
  "permission_denials": [],
  "fast_mode_state": "off",
  "uuid": "uuid-unique"
}
```

### Champs critiques pour ZAOS

| Champ | Usage |
|-------|-------|
| `duration_ms` | Durée totale du turn |
| `duration_api_ms` | Durée des appels API uniquement |
| `num_turns` | Nombre de turns (1 = pas de tool use, 2+ = avec tools) |
| `total_cost_usd` | Coût (informatif, plan Max = pas de facturation réelle) |
| `usage.input_tokens` | Tokens d'entrée |
| `usage.output_tokens` | Tokens de sortie |
| `usage.cache_read_input_tokens` | Tokens lus depuis le cache |
| `modelUsage` | Détail par modèle (utile si multi-modèle) |
| `modelUsage.*.contextWindow` | Taille de la fenêtre de contexte |
| `stop_reason` | `end_turn`, `max_tokens`, `tool_use` |
| `permission_denials` | Outils refusés par l'utilisateur |

---

## 8. Résumé des modes et flags (vérifié par traces réelles)

### 8.1 Deux niveaux de détail

| Mode | Events | Streaming | Usage |
|------|--------|-----------|-------|
| `--verbose` seul | `system`, `assistant` (complet), `user`, `rate_limit_event`, `result` | ❌ Blocs complets | Debug, scripts batch |
| `--verbose --include-partial-messages` | Tout ci-dessus **+** `stream_event` (deltas API) | ✅ Chunks progressifs | **ZAOS (recommandé)** |

### 8.2 Le `stream_event` wrapper (CONFIRMÉ)

Avec `--include-partial-messages`, chaque événement API est wrappé :

```json
{
  "type": "stream_event",
  "event": {
    "type": "content_block_delta",
    "index": 0,
    "delta": { "type": "text_delta", "text": "chunk de texte" }
  },
  "session_id": "uuid-session",
  "parent_tool_use_id": null,
  "uuid": "uuid-unique"
}
```

Les `stream_event` arrivent **intercalés** avec les `assistant` (qui sont des snapshots du message complet à ce point). L'ordre est :
1. `stream_event` (deltas progressifs) — pour le streaming UI
2. `assistant` (snapshot complet) — pour la reconstruction d'état

**Stratégie ZAOS :** Utiliser les `stream_event` pour le rendu temps réel, et les `assistant` comme checkpoints de vérification.

### 8.3 Types de deltas dans stream_event.event

| `event.type` | Sous-type (`delta.type`) | Données | Usage ZAOS |
|--------------|-------------------------|---------|-----------|
| `message_start` | — | Message initial vide avec `usage` | Début de turn, compteur tokens |
| `content_block_start` | — | `content_block.type` = `text`\|`tool_use`\|`thinking` | Créer le bloc dans le chat |
| `content_block_delta` | `text_delta` | `delta.text` | Streaming texte → chat |
| `content_block_delta` | `input_json_delta` | `delta.partial_json` | Accumuler JSON du tool input |
| `content_block_delta` | `thinking_delta` | `delta.thinking` | Streaming du thinking |
| `content_block_stop` | — | `index` | Finaliser le bloc |
| `message_delta` | — | `delta.stop_reason`, `usage` | Fin de message, tokens finaux |
| `message_stop` | — | — | Fin du flux pour ce turn |

### 8.4 Communication bidirectionnelle (stdin)

Le CLI supporte `--input-format stream-json` pour la communication programmatique sur stdin :
- Types de messages : `user_message`, `control_response`
- **Documentation incomplète** (GitHub issue #24594) — format pas encore officiellement documenté
- ZAOS pourra l'explorer en Phase 1 si le mode interactif l'exige

### 8.5 Extension VS Code — Architecture de référence

L'extension officielle utilise exactement :
```bash
claude --output-format stream-json --verbose --include-partial-messages
```

Elle expose un serveur MCP "ide" local :
- Bind : `127.0.0.1` sur port aléatoire
- Auth : token éphémère écrit dans `~/.claude/ide/` (permissions 0600)
- 2 outils visibles par Claude : `getDiagnostics`, `executeCode`
- Outils RPC cachés (filtrés de la vue Claude) : diffs, sélections, save

### 8.6 TODO : Tests restants

- [ ] Capturer une trace en mode interactif (sans `--print`) — stdin/stdout bidirectionnel
- [ ] Capturer une trace avec délégation d'agent (`Task` tool) — pour détecter les subagents
- [ ] Capturer une trace avec GoPeak MCP (outils `mcp__gopeak__*`)
- [ ] Capturer une trace avec erreur/interruption (SIGINT)
- [ ] Tester le resume de session (`--resume <session_id>`)
- [ ] Tester `--input-format stream-json` pour l'envoi programmatique de prompts
- [ ] Vérifier le format des `thinking_delta` (extended thinking)

---

## 9. Mapping événements → UI ZAOS

### 9.1 Panneau Chat

| Événement | Action UI |
|-----------|----------|
| `assistant` + `thinking` | Indicateur "🧠 Réflexion..." (collapsible) |
| `assistant` + `text` | Bulle de message assistant |
| `assistant` + `tool_use` | Bloc d'action inline (nom + params) |
| `user` + `tool_result` | Résultat collapsible sous l'action |

### 9.2 Feed d'actions

| Événement | Action UI |
|-----------|----------|
| `assistant` + `tool_use` | Nouvelle entrée dans le feed avec icône + résumé |
| `user` + `tool_result` | Met à jour l'entrée avec le statut (✅/❌) |
| `user` + `timestamp` | Horodatage de l'action |

### 9.3 Barre de statut

| Événement | Mise à jour |
|-----------|------------|
| `system/init` | Modèle, version CLI, outils, MCP servers |
| `rate_limit_event` | Statut rate limit, temps avant reset |
| `result` | Tokens consommés, durée, coût |

### 9.4 Panneau Agents

| Événement | Action UI |
|-----------|----------|
| `assistant` + `tool_use` (name=`Task`) | Délégation détectée → agent actif change |
| `system/init` + `agents` | Liste des agents disponibles |

### 9.5 Panneau Workflow

Le workflow n'est PAS directement dans le flux stream-json. ZAOS doit :
1. **File watcher** sur `.workflow/state.json` pour les changements de phase/mode
2. **File watcher** sur `.memory/current-epic.md` pour la progression des tâches
3. Détecter les appels à `wfctl.sh` dans les `tool_use` Bash pour anticiper les changements

---

## 10. Architecture du parser ZAOS (Rust)

### 10.1 Proposition de types

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Événement top-level du flux stream-json
/// Avec --include-partial-messages, on reçoit aussi des StreamDelta
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum CliEvent {
    #[serde(rename = "system")]
    System(SystemEvent),

    /// Événements API streamés (deltas) — UNIQUEMENT avec --include-partial-messages
    #[serde(rename = "stream_event")]
    StreamDelta(StreamDeltaEvent),

    /// Snapshot complet du message assistant (émis après les deltas)
    #[serde(rename = "assistant")]
    Assistant(AssistantEvent),

    /// Résultat d'un tool use
    #[serde(rename = "user")]
    User(UserEvent),

    #[serde(rename = "rate_limit_event")]
    RateLimit(RateLimitEvent),

    #[serde(rename = "result")]
    Result(ResultEvent),

    // Pour les événements non reconnus (forward compatibility)
    #[serde(other)]
    Unknown,
}

/// Wrapper autour des événements API Claude (streaming deltas)
#[derive(Debug, Deserialize)]
pub struct StreamDeltaEvent {
    pub event: ApiStreamEvent,
    pub session_id: String,
    pub parent_tool_use_id: Option<String>,
    pub uuid: String,
}

/// Événement API Claude brut (inside stream_event.event)
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ApiStreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: Value },

    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        index: u32,
        content_block: ContentBlockInfo,
    },

    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        index: u32,
        delta: DeltaContent,
    },

    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: u32 },

    #[serde(rename = "message_delta")]
    MessageDelta {
        delta: Value,
        usage: Option<Usage>,
    },

    #[serde(rename = "message_stop")]
    MessageStop,

    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct ContentBlockInfo {
    #[serde(rename = "type")]
    pub block_type: String,  // "text", "tool_use", "thinking"
    pub text: Option<String>,
    pub id: Option<String>,    // pour tool_use
    pub name: Option<String>,  // pour tool_use
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum DeltaContent {
    #[serde(rename = "text_delta")]
    TextDelta { text: String },

    #[serde(rename = "input_json_delta")]
    InputJsonDelta { partial_json: String },

    #[serde(rename = "thinking_delta")]
    ThinkingDelta { thinking: String },

    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct SystemEvent {
    pub subtype: String, // "init", "api_retry"
    pub session_id: String,
    pub uuid: String,
    // init fields
    pub cwd: Option<String>,
    pub tools: Option<Vec<String>>,
    pub mcp_servers: Option<Vec<Value>>,
    pub model: Option<String>,
    pub claude_code_version: Option<String>,
    pub agents: Option<Vec<String>>,
    // api_retry fields
    pub attempt: Option<u32>,
    pub max_retries: Option<u32>,
    pub error: Option<String>,
    pub error_status: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct AssistantEvent {
    pub message: AssistantMessage,
    pub parent_tool_use_id: Option<String>,
    pub session_id: String,
    pub uuid: String,
}

#[derive(Debug, Deserialize)]
pub struct AssistantMessage {
    pub id: String,
    pub model: String,
    pub content: Vec<ContentBlock>,
    pub stop_reason: Option<String>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "thinking")]
    Thinking {
        thinking: String,
        signature: String,
    },

    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
        caller: Option<Value>,
    },

    #[serde(rename = "text")]
    Text {
        text: String,
    },

    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct UserEvent {
    pub message: UserMessage,
    pub parent_tool_use_id: Option<String>,
    pub session_id: String,
    pub uuid: String,
    pub timestamp: Option<String>,
    pub tool_use_result: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct UserMessage {
    pub role: String,
    pub content: Vec<ToolResultBlock>,
}

#[derive(Debug, Deserialize)]
pub struct ToolResultBlock {
    pub tool_use_id: String,
    #[serde(rename = "type")]
    pub block_type: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct RateLimitEvent {
    pub rate_limit_info: RateLimitInfo,
    pub uuid: String,
    pub session_id: String,
}

#[derive(Debug, Deserialize)]
pub struct RateLimitInfo {
    pub status: String, // "allowed", "rate_limited"
    #[serde(rename = "resetsAt")]
    pub resets_at: u64,
    #[serde(rename = "rateLimitType")]
    pub rate_limit_type: String,
}

#[derive(Debug, Deserialize)]
pub struct ResultEvent {
    pub subtype: String, // "success", "error"
    pub is_error: bool,
    pub duration_ms: u64,
    pub duration_api_ms: u64,
    pub num_turns: u32,
    pub result: String,
    pub stop_reason: String,
    pub session_id: String,
    pub total_cost_usd: f64,
    pub usage: Value,
    #[serde(rename = "modelUsage")]
    pub model_usage: Option<Value>,
    pub uuid: String,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_input_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
}
```

### 10.2 Parser ligne par ligne

```rust
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::ChildStdout;

pub async fn parse_stream(stdout: ChildStdout, tx: tokio::sync::broadcast::Sender<StreamEvent>) {
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await.unwrap_or(None) {
        if line.is_empty() {
            continue;
        }
        match serde_json::from_str::<StreamEvent>(&line) {
            Ok(event) => {
                let _ = tx.send(event);
            }
            Err(e) => {
                eprintln!("Parse error: {} on line: {}", e, &line[..80.min(line.len())]);
            }
        }
    }
}
```

---

## 11. Ressources externes

- [Documentation officielle headless mode](https://code.claude.com/docs/en/headless)
- [CLI reference](https://code.claude.com/docs/en/cli-reference)
- [Khan/format-claude-stream](https://github.com/Khan/format-claude-stream) — parser communautaire
- [GitHub issue #24596](https://github.com/anthropics/claude-code/issues/24596) — demande de doc du format
- [Stream-json cheatsheet](https://takopi.dev/reference/runners/claude/stream-json-cheatsheet/)
