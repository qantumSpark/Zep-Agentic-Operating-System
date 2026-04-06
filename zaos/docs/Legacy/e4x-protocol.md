# Protocole E4X — Formats et structures que ZAOS doit comprendre

> Document de référence technique pour l'intégration ZAOS ↔ Workflow-kit
> Date : 29 mars 2026
> Source : workflow-kit (snapshot local)

---

## 1. state.json — État du workflow

**Chemin :** `.workflow/state.json` (relatif à la racine du projet)

### 1.1 Schéma actuel

```json
{
  "phase": "idle",
  "epic": null,
  "task": null,
  "mode": "free",
  "gate_validated": false,
  "last_updated": "2026-03-28T20:20:57Z"
}
```

### 1.2 Champs

| Champ | Type | Valeurs possibles | Description |
|-------|------|-------------------|-------------|
| `phase` | string | `idle`, `comprehension`, `specification`, `architecture`, `implementation`, `review`, `test`, `closure` | Phase active du pipeline |
| `epic` | string \| null | Nom libre ou null | Epic en cours |
| `task` | string \| null | Description libre ou null | Tâche courante |
| `mode` | string | `free`, `pipeline` | Mode de travail. `pipeline` active le blocage mécanique de code sans plan |
| `gate_validated` | boolean | true/false | Gate de la phase courante validé ou non. Reset à false à chaque changement de phase |
| `last_updated` | string (ISO 8601) | Timestamp | Dernière modification |

### 1.3 Extensions ZAOS prévues

ZAOS ajoutera des champs au même fichier (rétrocompatible — `wfctl.sh` ignore les champs inconnus) :

```json
{
  "phase": "implementation",
  "epic": "Système de combat",
  "task": "HitBox component",
  "mode": "pipeline",
  "gate_validated": false,
  "last_updated": "2026-03-29T14:32:00Z",

  "history": [
    {
      "phase": "comprehension",
      "started_at": "2026-03-29T14:00:00Z",
      "ended_at": "2026-03-29T14:12:00Z",
      "agent": "orchestrator",
      "tokens": { "input": 8200, "output": 1400 }
    }
  ],
  "session": {
    "started_at": "2026-03-29T14:00:00Z",
    "total_tokens": { "input": 42300, "output": 8900 },
    "screenshots_count": 7
  }
}
```

### 1.4 Cycle de vie des phases

```
idle → comprehension → specification → architecture → implementation → review → test → closure → idle
```

Chaque transition :
1. Remet `gate_validated` à `false`
2. Met à jour `last_updated`
3. (ZAOS) Ajoute une entrée dans `history[]`

### 1.5 Raccourcis par type de tâche

Le pipeline complet n'est pas toujours nécessaire. Raccourcis définis dans `process.md` :

| Type | Phases |
|------|--------|
| Feature complète | comprehension → specification → architecture → implementation → review → test → closure |
| Bug fix | comprehension → implementation → review → closure |
| Refactoring | architecture → implementation → review → test → closure |
| Recherche | comprehension → closure |
| Continuation (reprise) | Reprend à la phase en cours |

---

## 2. wfctl.sh — Utilitaire CLI

**Chemin :** `.workflow/wfctl.sh`

### 2.1 Commandes

| Commande | Action | Effet sur state.json |
|----------|--------|---------------------|
| `status` | Affiche l'état courant | Lecture seule |
| `phase <nom>` | Change la phase | `phase` = nom, `gate_validated` = false |
| `epic <nom>` | Définit l'epic active | `epic` = nom |
| `task <description>` | Définit la tâche courante | `task` = description |
| `mode <free\|pipeline>` | Change le mode | `mode` = valeur |
| `gate validate` | Valide le gate | `gate_validated` = true |
| `next` | Passe à la phase suivante | `phase` = suivante dans l'ordre, `gate_validated` = false |
| `reset` | Retour à l'état initial | `phase` = idle, `epic` = null, `task` = null, `mode` = free, `gate_validated` = false |
| `help` | Affiche l'aide | Aucun |

### 2.2 Invocation

```bash
bash .workflow/wfctl.sh <commande> [arguments]
```

Dépendance : `jq` (manipulation JSON).

### 2.3 Compatibilité ZAOS

ZAOS réimplémente ces commandes en Rust natif via le `WorkflowEngine`. Les deux coexistent :
- ZAOS lit/écrit `state.json` directement
- `wfctl.sh` reste disponible en fallback CLI
- File watcher (`notify` crate) détecte les modifications externes

---

## 3. Agents — Système de délégation

**Chemin :** `.claude/agents/*.md`

### 3.1 Agents disponibles

| Agent | Fichier | Rôle | Phases principales |
|-------|---------|------|-------------------|
| Orchestrateur | (CLAUDE.md) | Coordination, délégation, mémoire | Toutes |
| Architect | `architect.md` | Design technique, breakdown de tâches, ADRs | specification, architecture |
| Coder | `coder.md` | Implémentation fichier par fichier | implementation |
| Reviewer | `reviewer.md` | Revue qualité, conformité, anti-hallucination | review |
| Tester | `tester.md` | Scénarios de test, exécution | test |
| Playtester | `playtester.md` | QA visuel via GoPeak (screenshots, inputs) | test |
| Researcher | `researcher.md` | Vérification APIs, recherche doc, collecte info | Toutes (sur demande) |

### 3.2 Format des fichiers agents

Chaque fichier `.md` contient :
- **Rôle** : description du rôle
- **Quand invoquer** : conditions de délégation
- **Premières actions** : ce que l'agent fait en premier (lecture de fichiers mémoire)
- **Comment il travaille** : processus de travail
- **Format de sortie** : structure de son output
- **Règles strictes** : ce qu'il ne doit jamais faire

### 3.3 Mécanisme de délégation

L'orchestrateur (CLAUDE.md) délègue via les subagents `Task` de Claude Code. ZAOS doit détecter ces délégations dans le flux d'événements JSON pour :
- Afficher l'agent actif dans le panneau Agents
- Badger les messages avec le nom de l'agent (`[Coder]`, `[Reviewer]`, etc.)
- Tracker les transitions entre agents

---

## 4. Mémoire — Système .memory/

**Chemin :** `.memory/`

### 4.1 Structure

```
.memory/
├── INDEX.md            ← Point d'entrée (max 30 lignes)
├── state.md            ← État projet courant (max 50 lignes, ÉCRASÉ pas accumulé)
├── current-epic.md     ← Epic active avec plan de tâches (max 40 lignes)
├── architecture.md     ← Stack, structure projet, dépendances
├── conventions.md      ← Conventions de code spécifiques au projet
├── decisions/          ← ADRs (Architecture Decision Records)
├── research/           ← Résultats de recherche à valeur long-terme
└── sessions/           ← Logs de session (max 10 récents + archive)
```

### 4.2 Fichiers que ZAOS doit lire

#### INDEX.md
Point d'entrée du système mémoire. Liens vers les autres fichiers. Max 30 lignes.

#### state.md
État courant du projet. ZAOS peut l'afficher dans le dashboard d'accueil.

Structure :
- Milestones (tableau : #, nom, statut, epics)
- Epic active (nom, détails dans current-epic.md)
- Bugs connus
- Blocages
- Prochaines priorités

#### current-epic.md
Plan de l'epic active. Contient le tableau de tâches que ZAOS doit parser.

Structure :
- En-tête : nom, milestone parent, date, statut
- Objectif : 1-2 phrases
- Plan technique : approche de l'architecte
- **Tâches** (tableau) : `# | Task | File(s) | Status | Notes`
  - Statuts : `todo` → `in_progress` → `done`
- Prochaine action

**Important pour ZAOS :** Ce tableau de tâches est la source de vérité pour la progression. Le hook `block-code-without-plan.sh` vérifie que ce fichier contient des tâches avant d'autoriser l'écriture de code.

#### architecture.md
Rarement modifié. Stack technique, structure de dossiers, dépendances principales, systèmes principaux.

#### conventions.md
Conventions de nommage, patterns adoptés/interdits, structure de fichier type, règles de commit.

### 4.3 Limites de taille (règles d'hygiène)

| Fichier | Max lignes | Politique |
|---------|-----------|-----------|
| INDEX.md | 30 | Liens uniquement, pas de contenu |
| state.md | 50 | ÉCRASÉ à chaque mise à jour |
| current-epic.md | 40 | ÉCRASÉ au changement d'epic |
| Session logs | 20 | Max 10 récents, puis archivage condensé |

---

## 5. Hooks — Enforcement mécanique

**Chemin :** `.claude/hooks/` + `.claude/settings.json`

### 5.1 Configuration (settings.json)

```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "command": "bash .claude/hooks/inject-context.sh \"$PROMPT\"",
        "timeout": 5000
      }
    ],
    "PreToolUse": [
      {
        "matcher": "Write|Edit",
        "command": "bash .claude/hooks/block-code-without-plan.sh \"$TOOL_INPUT\"",
        "timeout": 5000
      }
    ],
    "SessionStart": [
      {
        "matcher": "compact",
        "command": "bash .claude/hooks/on-compact.sh",
        "timeout": 10000
      }
    ]
  }
}
```

### 5.2 Hooks détaillés

#### inject-context.sh (UserPromptSubmit)
- **Quand :** À chaque prompt utilisateur
- **Action :** Lit state.json + current-epic.md, injecte un rappel de phase/rôle/règles
- **Sortie :** JSON avec champ `additionalContext`
- **Pertinence ZAOS :** ZAOS peut reproduire cette injection côté UI ou laisser le hook agir. Le hook fonctionne indépendamment de ZAOS.

#### block-code-without-plan.sh (PreToolUse Write|Edit)
- **Quand :** Avant toute écriture/édition de fichier code (.gd, .dart, .tscn, .tres, .gdshader, .gdextension)
- **Action :** Vérifie que current-epic.md contient un plan avec des tâches todo/in_progress
- **Sortie :** Exit 0 (OK) ou Exit 2 (BLOCAGE DUR — Claude ne peut pas bypass)
- **Bypass :** Mode `free` désactive le blocage
- **Fichiers non bloqués :** .md, .json, .cfg, .txt, .mdc, .sh
- **Pertinence ZAOS :** Ce hook est critique — ZAOS doit comprendre que le mode `pipeline` impose des contraintes mécaniques.

#### on-compact.sh (SessionStart compact)
- **Quand :** Après compaction du contexte (conversation trop longue)
- **Action :** Ré-injecte l'état workflow, le plan d'epic, l'état projet, les conventions
- **Sortie :** Texte brut injecté dans le contexte Claude
- **Pertinence ZAOS :** ZAOS pourrait détecter les compactions et les afficher dans le feed d'actions.

#### welcome.sh (SessionStart startup)
- **Quand :** Au démarrage d'une session
- **Action :** Bannière de bienvenue, diagnostics des composants, état courant
- **Pertinence ZAOS :** ZAOS remplacera visuellement cette bannière par son propre dashboard d'accueil.

---

## 6. Rules — Règles Claude Code

**Chemin :** `.claude/rules/*.mdc`

### 6.1 Fichiers

| Fichier | Activation | Contenu |
|---------|-----------|---------|
| `01-always.mdc` | Toujours | Session start, before creating file, before coding, during work, end of session |
| `02-anti-hallucination.mdc` | Toujours | Vérification APIs, imports, patterns, code incomplet, niveaux de confiance |
| `03-code-review.mdc` | Sur modification fichiers code (`**/*.gd`, `**/*.dart`, `**/*.tscn`, `**/*.tres`) | Conformité, duplication, complétude, cohérence architecturale |
| `04-memory-hygiene.mdc` | Toujours | Limites de taille, gestion sessions/ADRs/research, anti-duplication |

### 6.2 Pertinence ZAOS

Les rules sont gérées par Claude Code directement. ZAOS n'a pas besoin de les interpréter mais peut :
- Les afficher dans un panneau de configuration
- Permettre d'activer/désactiver des rules (si le CLI le supporte)
- Montrer quelle rule est en train de s'appliquer dans le feed d'actions

---

## 7. Templates

**Chemin :** `.workflow/templates/`

### 7.1 Fichiers

| Template | Usage | Créé par |
|----------|-------|----------|
| `adr-template.md` | Architecture Decision Records | Architect agent |
| `epic-plan.md` | Plan d'epic avec tâches | Architect agent |
| `feature-spec.md` | Spécification de feature | Orchestrateur (phase specification) |
| `session-log.md` | Log de fin de session | Orchestrateur (phase closure) |

### 7.2 Pertinence ZAOS

ZAOS peut :
- Proposer la création d'un nouveau fichier depuis un template via le chat
- Afficher les templates disponibles dans le dashboard
- Générer automatiquement un session-log en fin de session (amélioration Phase 4)

---

## 8. Overlays — Extensions par technologie

**Chemin :** `.overlays/<techno>/`

### 8.1 Structure

```
.overlays/
├── godot/
│   ├── conventions.md      ← Conventions GDScript/Godot
│   ├── patterns.md         ← Patterns fréquents Godot
│   └── rules/
│       └── godot-specific.mdc  ← Règles activées pour fichiers Godot
└── flutter/
    ├── conventions.md      ← Conventions Dart/Flutter
    ├── patterns.md         ← Patterns fréquents Flutter
    └── rules/
        └── flutter-specific.mdc  ← Règles activées pour fichiers Flutter
```

### 8.2 Pertinence ZAOS

ZAOS doit détecter quel overlay est actif (basé sur le type de projet) pour :
- Afficher les conventions pertinentes
- Adapter la coloration syntaxique (GDScript vs Dart)
- Filtrer les actions/screenshots selon le contexte

---

## 9. Résumé des fichiers et chemins critiques pour ZAOS

```
projet/
├── .workflow/
│   ├── state.json          ★ CRITIQUE — état du workflow, file watching obligatoire
│   ├── wfctl.sh            ○ FALLBACK — ZAOS réimplémente en Rust
│   ├── process.md          ○ RÉFÉRENCE — définition du pipeline
│   ├── phases/*.md         ○ RÉFÉRENCE — checklists par phase
│   └── templates/*.md      ○ RÉFÉRENCE — templates de documents
│
├── .claude/
│   ├── agents/*.md         ★ IMPORTANT — détection des agents actifs
│   ├── hooks/*.sh          ○ INFO — gérés par le CLI, ZAOS peut monitorer
│   ├── rules/*.mdc         ○ INFO — gérés par le CLI
│   └── settings.json       ○ INFO — config des hooks
│
├── .memory/
│   ├── INDEX.md            ★ IMPORTANT — point d'entrée mémoire
│   ├── state.md            ★ IMPORTANT — état projet pour dashboard d'accueil
│   ├── current-epic.md     ★ CRITIQUE — plan de tâches, progression
│   ├── architecture.md     ○ RÉFÉRENCE — stack et structure
│   ├── conventions.md      ○ RÉFÉRENCE — conventions de code
│   ├── decisions/          ○ RÉFÉRENCE — ADRs
│   ├── research/           ○ RÉFÉRENCE — recherches
│   └── sessions/           ○ RÉFÉRENCE — logs de session
│
├── .overlays/<techno>/     ○ RÉFÉRENCE — conventions par technologie
│
└── CLAUDE.md               ○ INFO — instructions orchestrateur
```

Légende : ★ = ZAOS doit lire + watcher en temps réel | ○ = ZAOS peut lire à la demande
