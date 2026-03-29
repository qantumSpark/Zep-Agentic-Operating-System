# Plan Directeur — Workflow LLM-Assisted Dev

> Document de synthèse du brainstorming du 2026-03-27
> Auteur : Antoine + Claude
> Statut : Plan validé, prêt pour implémentation

---

## 1. Vision

Créer un ensemble de fichiers (.md, .mdc, templates) à déposer à la racine de tout nouveau projet Godot ou Flutter. Ces fichiers transforment Claude Code en assistant de développement structuré, avec mémoire persistante, rôles spécialisés, et garde-fous anti-hallucination.

### Objectifs
- **Zéro ré-explication** : Claude comprend le projet dès le lancement
- **Anti-hallucination** : vérifications obligatoires avant de coder
- **Anti-bloat** : mémoire propre, archivage automatique, pas de duplication
- **Continuité** : mémoire long terme entre les sessions
- **Spec-driven** : on spécifie d'abord, on code ensuite
- **Découpage granulaire** : chaque tâche de code est petite, précise et vérifiable

### Cibles
- Jeux vidéo (Godot 4 / GDScript)
- Apps mobiles (Flutter / Dart)
- Architecture : base commune + overlays par techno

---

## 2. Structure des fichiers

```
projet/
├── CLAUDE.md                              ← Orchestrateur (point d'entrée)
│
├── .claude/
│   ├── rules/
│   │   ├── 01-always.mdc                 ← Règles universelles
│   │   ├── 02-anti-hallucination.mdc     ← Vérifications obligatoires
│   │   ├── 03-code-review.mdc            ← Checklist review auto
│   │   └── 04-memory-hygiene.mdc         ← Maintenance contexte
│   │
│   └── agents/
│       ├── architect.md                   ← Design et décisions
│       ├── coder.md                       ← Implémentation
│       ├── reviewer.md                    ← Validation qualité
│       ├── tester.md                      ← Tests
│       ├── researcher.md                  ← Recherche et collecte
│       └── playtester.md                  ← QA visuel du jeu (Godot)
│
├── .workflow/
│   ├── process.md                         ← Pipeline de dev
│   ├── phases/
│   │   ├── 01-spec.md
│   │   ├── 02-architecture.md
│   │   ├── 03-implementation.md
│   │   ├── 04-review.md
│   │   └── 05-test-deploy.md
│   └── templates/
│       ├── feature-spec.md
│       ├── epic-plan.md                   ← Template plan d'epic avec tasks
│       ├── adr-template.md
│       └── session-log.md
│
├── .memory/
│   ├── INDEX.md                           ← Table des matières mémoire
│   ├── architecture.md                    ← Stack, structure, dépendances
│   ├── conventions.md                     ← Style, naming, patterns
│   ├── state.md                           ← État courant (milestones + epics)
│   ├── current-epic.md                    ← Epic active : tasks et statuts
│   ├── decisions/                         ← ADRs
│   │   └── (numérotés 001-xxx.md)
│   ├── research/                          ← Résultats de recherches
│   │   └── (numérotés 001-xxx.md)
│   └── sessions/
│       └── (datés YYYY-MM-DD.md)
│
└── .overlays/
    ├── godot/
    │   ├── conventions.md
    │   ├── patterns.md
    │   └── rules/
    │       └── godot-specific.mdc
    └── flutter/
        ├── conventions.md
        ├── patterns.md
        └── rules/
            └── flutter-specific.mdc
```

---

## 3. L'Orchestrateur (CLAUDE.md)

Rôle par défaut de Claude à chaque lancement. Fichier court (~40 lignes).

### Contenu
- Identité : "Tu es l'orchestrateur de ce projet"
- Première action à chaque session : lire INDEX.md + state.md
- Table de délégation vers les agents
- Quand agir directement vs déléguer
- Pointer vers .workflow/process.md pour le pipeline
- Fin de session : mise à jour mémoire
- Références vers tous les sous-systèmes

### Règles de délégation

| Situation | Agent |
|---|---|
| Nouvelle feature, choix d'archi | → Architect |
| Implémentation d'un plan validé | → Coder |
| Code terminé, besoin de validation | → Reviewer |
| Écriture / vérification de tests | → Tester |
| Besoin d'infos, vérification doc, collecte refs | → Researcher |
| Question simple, debug rapide, mise à jour mémoire | → Fait lui-même |
| Tâche ambiguë | → Demande des précisions |

---

## 4. Hiérarchie de découpage des projets

### Principe

Plus une tâche donnée au LLM est petite et précise, moins il hallucine. Mais un découpage trop fin perd la cohérence d'ensemble. Le bon équilibre : 4 niveaux, chacun avec son rôle.

### Les 4 niveaux

**Milestone** (la destination)
- Objectif utilisateur, pas technique : "Le joueur peut ramasser, stocker et utiliser des objets"
- 3 à 8 par projet
- On ne code jamais un milestone directement — on le décompose en epics
- Défini par : l'utilisateur + l'Orchestrateur
- Stocké dans : `.memory/state.md` (vue d'ensemble)

**Epic** (le chantier)
- Bloc fonctionnel cohérent : "Système d'inventaire", "UI d'inventaire", "Système de loot"
- 2 à 5 par milestone
- C'est le niveau où l'Architecte intervient pour planifier
- Défini par : l'Architecte
- Stocké dans : `.memory/state.md` (liste) + `.memory/current-epic.md` (détail de l'epic active)

**Task** (l'unité de travail — NIVEAU CRITIQUE)
- Un incrément de code précis et vérifiable : "Créer la classe Item avec name, icon_path, stack_size dans items/item.gd"
- 3 à 10 par epic
- C'est ce que le Codeur exécute. Une task = un fichier modifié (2-3 max si couplés)
- Défini par : l'Architecte (lors du plan de l'epic)
- Stocké dans : `.memory/current-epic.md` (avec statut todo/in_progress/done)

**Step** (le micro-pas)
- Décomposition interne d'une task par le Codeur au moment de l'implémentation
- Jamais planifié à l'avance — émerge pendant le travail
- Non stocké en mémoire

### Calibrage d'une bonne Task
- Touche 1 fichier (2-3 max si fortement couplés)
- Vérifiable immédiatement : ça compile, on peut tester
- Décrite en une phrase avec le quoi ET le où
- ~15-60 min de travail LLM — si plus, re-découper
- Si une task touche 5 fichiers et 3 systèmes → c'est une epic déguisée

### Flow de découpage
1. Utilisateur donne un objectif → Orchestrateur identifie le milestone
2. Architecte découpe le milestone en epics, propose l'ordre
3. Utilisateur valide → on attaque la première epic
4. Architecte découpe l'epic en tasks précises → current-epic.md
5. Codeur prend les tasks une par une, dans l'ordre
6. Reviewer valide après chaque task (ou groupe de tasks liées)
7. Epic terminée → archiver, passer à la suivante

---

## 5. Système de Mémoire (.memory/)

### Principes
- Mémoire en entonnoir : INDEX (léger, toujours lu) → fichiers de référence (à la demande) → fichiers profonds (rarement)
- Ne jamais surcharger le contexte : pointer plutôt que copier
- Semi-automatique : Claude propose les mises à jour, l'utilisateur valide

### Fichiers de référence (Niveau 2)

**INDEX.md** (~30 lignes max)
- Liste chaque fichier mémoire avec une description d'une ligne
- Sert de carte pour savoir où chercher

**architecture.md**
- Stack technique, structure des dossiers, dépendances principales
- Mis à jour rarement (changements d'archi uniquement)

**conventions.md**
- Naming, style de code, patterns à utiliser/éviter
- Structure d'un fichier type
- Lu avant chaque phase de code

**state.md** (max 50 lignes, écrasé à chaque mise à jour)
- Vue d'ensemble : milestones et leur avancement
- Epic en cours identifiée
- Bugs connus, blocages, prochaines priorités
- Mis à jour à chaque fin de session

**current-epic.md** (max 40 lignes, écrasé à chaque changement d'epic)
- Nom et objectif de l'epic active
- Liste des tasks avec statut (todo / in_progress / done)
- Dépendances entre tasks si applicable
- Le Codeur lit ce fichier pour savoir quoi faire ensuite
- Archivé quand l'epic est terminée

### Fichiers profonds (Niveau 3)

**decisions/NNN-titre.md** (ADRs)
- Format : contexte → décision → alternatives rejetées → conséquences
- Numérotés séquentiellement
- Marqués [SUPERSEDED] puis archivés quand remplacés

**research/NNN-titre.md**
- Résultats de recherches du Researcher
- Format : question → findings (avec sources) → recommandation → fiabilité
- Sauvegardé uniquement si valeur long terme

**sessions/YYYY-MM-DD.md** (max 20 lignes)
- Ce qui a été fait, décisions prises, problèmes ouverts, prochaines étapes
- Au-delà de 10 sessions : archiver les plus anciennes dans archive.md condensé

---

## 6. Rules (.claude/rules/)

### 01-always.mdc — Universelles (toujours actives)
- Lire INDEX.md + state.md en début de session
- Vérifier architecture.md avant de créer un fichier
- Lire conventions.md + overlay techno avant de coder
- Ne jamais assumer — demander si info manquante
- Proposer mise à jour mémoire en fin de session

### 02-anti-hallucination.mdc — Vérifications
- Ne jamais inventer une API, classe, méthode ou signal
- Exprimer le doute explicitement si incertain
- Vérifier imports, chemins, noms avant de référencer
- Citer la source des patterns utilisés
- Pas de code placeholder silencieux — TODO: explicites

### 03-code-review.mdc — Review auto (activé sur modification de code)
- Conformité aux conventions
- Pas de duplication avec code existant
- Cohérence de naming avec le projet
- Edge cases gérés ou documentés
- Cohérence avec les ADRs existants

### 04-memory-hygiene.mdc — Anti-bloat
- state.md ≤ 50 lignes, écrasé pas accumulé
- Session logs ≤ 20 lignes
- Archiver sessions au-delà de 10
- ADRs [SUPERSEDED] archivés après 30 jours
- Jamais dupliquer une info — référencer plutôt

---

## 7. Agents (.claude/agents/)

### architect.md — L'Architecte
- **Invoqué** : nouvelle feature, restructuration, choix technique
- **Lit** : architecture.md, conventions.md, ADRs pertinents
- **Produit** : plan technique, ADR si nouvelle décision
- **Interdit** : écrire du code, modifier des fichiers, décider seul

### coder.md — Le Codeur
- **Invoqué** : implémentation d'un plan validé
- **Lit** : conventions.md, overlay techno, plan de l'architecte
- **Produit** : code incrémental, TODO: pour l'incomplet
- **Interdit** : changer l'archi, créer des fichiers hors structure, inventer des APIs

### reviewer.md — Le Reviewer
- **Invoqué** : après le code, avant clôture
- **Lit** : code modifié, conventions, ADRs
- **Produit** : liste de remarques (bloquant / suggestion / nitpick)
- **Interdit** : corriger le code lui-même, approuver sans relecture complète

### tester.md — Le Testeur
- **Invoqué** : écrire ou vérifier les tests
- **Lit** : spec de la feature, code implémenté
- **Produit** : tests ciblés, rapport de couverture
- **Interdit** : modifier le code de production, mocks inutiles

### researcher.md — Le Chercheur
- **Invoqué** : besoin d'infos, vérification, collecte de références
- **Lit** : demande de l'orchestrateur ou de l'architecte
- **Produit** : mini-rapport sourcé (question → findings → recommandation → fiabilité)
- **Sauvegarde** : propose stockage dans .memory/research/ si valeur long terme
- **Interdit** : prendre des décisions d'archi, coder, présenter des infos non sourcées comme des faits

### playtester.md — Le QA Playtester (Godot uniquement)
- **Invoqué** : test visuel du jeu, vérification UI/gameplay du point de vue joueur
- **Prérequis** : gopeak MCP + runtime addon actif dans le projet Godot
- **Outils** : capture_screenshot, inject_action/key/mouse, inspect_runtime_tree, get_runtime_metrics, get_debug_output
- **Produit** : rapport QA structuré avec screenshots, métriques, liste de bugs (critiques/mineurs)
- **Interdit** : modifier le code ou les scènes, ignorer une erreur console, approuver si problème critique détecté

---

## 8. Pipeline de Dev (.workflow/process.md)

### Flow complet (nouvelle feature)

```
Phase 0: Initialisation (auto, chaque session)
  → Lire INDEX.md + state.md
  → Reprendre le travail en cours si applicable
  ✓ Gate: aucune (automatique)

Phase 1: Compréhension (Orchestrateur)
  → Écouter, clarifier, reformuler
  ✓ Gate: utilisateur valide la reformulation

Phase 2: Spec (Orchestrateur + Architecte)
  → Mini-spec avec template feature-spec.md
  → Researcher si besoin d'infos
  ✓ Gate: utilisateur valide la spec

Phase 3: Architecture (Architecte)
  → Plan technique, ADR si nécessaire
  → Researcher si besoin de vérification
  ✓ Gate: utilisateur valide le plan

Phase 4: Implémentation (Codeur)
  → Code incrémental suivant le plan
  → Vérifications à chaque étape
  ✓ Gate: code compile/tourne sans erreur

Phase 5: Review (Reviewer)
  → Relecture complète, liste de remarques
  → Codeur corrige si points bloquants
  ✓ Gate: aucun point bloquant restant

Phase 6: Tests (Testeur)
  → Scénarios, écriture, exécution
  ✓ Gate: tests passent

Phase 7: Clôture (Orchestrateur)
  → Mise à jour state.md + session log
  → Résumé pour l'utilisateur
  ✓ Gate: utilisateur confirme
```

### Raccourcis par type de tâche

| Type | Phases |
|---|---|
| Bug fix simple | 1 → 4 → 5 → 7 |
| Refactoring | 1 → 3 → 4 → 5 → 7 |
| Question / explication | Orchestrateur direct |
| Recherche pure | 1 → Researcher → 7 |
| Feature complète | 0 → 1 → 2 → 3 → 4 → 5 → 6 → 7 |

---

## 9. Overlays (.overlays/)

Base commune (tous les fichiers ci-dessus) + couche spécifique par techno.

### godot/
- **conventions.md** : style GDScript, naming scènes/nodes, signals, organisation autoloads
- **patterns.md** : patterns courants (state machine, component, observer via signals)
- **godot-specific.mdc** : vérifier que nodes/signals existent, patterns autoload vs injection, glob sur *.gd, *.tscn, *.tres

### flutter/
- **conventions.md** : style Dart, structure widgets, state management choisi
- **patterns.md** : patterns courants (BLoC, Provider, Riverpod, repository pattern)
- **flutter-specific.mdc** : vérifier packages dans pubspec.yaml, glob sur *.dart

---

## 10. Plan d'implémentation

Ordre recommandé pour créer les fichiers :

### Étape 1 — Le squelette
Créer tous les dossiers et les fichiers vides/templates.

### Étape 2 — Le cœur (critique)
1. `CLAUDE.md` (orchestrateur)
2. `.claude/rules/01-always.mdc`
3. `.claude/rules/02-anti-hallucination.mdc`
4. `.memory/INDEX.md` (template)

### Étape 3 — Les agents
5. `.claude/agents/architect.md`
6. `.claude/agents/coder.md`
7. `.claude/agents/reviewer.md`
8. `.claude/agents/tester.md`
9. `.claude/agents/researcher.md`

### Étape 4 — Le workflow
10. `.workflow/process.md`
11. `.workflow/phases/` (les 5 fichiers de phase)
12. `.workflow/templates/` (les 3 templates)

### Étape 5 — Les rules complémentaires
13. `.claude/rules/03-code-review.mdc`
14. `.claude/rules/04-memory-hygiene.mdc`

### Étape 6 — Les overlays
15. `.overlays/godot/` (conventions, patterns, rules)
16. `.overlays/flutter/` (conventions, patterns, rules)

### Étape 7 — La mémoire (templates)
17. `.memory/architecture.md` (template à remplir par projet)
18. `.memory/conventions.md` (template à remplir par projet)
19. `.memory/state.md` (template vide)

### Étape 8 — Test et itération
20. Tester le workflow sur un mini-projet Godot
21. Ajuster les rules et agents selon les résultats
22. Tester sur un mini-projet Flutter
23. Affiner les overlays

---

## 11. Critères de succès

Le workflow est réussi si :
- [ ] Claude comprend le projet sans ré-explication à chaque session
- [ ] Les hallucinations de code sont significativement réduites
- [ ] La mémoire reste propre après 10+ sessions
- [ ] Le pipeline spec → code → review → test est suivi naturellement
- [ ] Les overlays Godot et Flutter fonctionnent correctement
- [ ] Le contexte n'est jamais surchargé (pas de fichiers > 50 lignes chargés inutilement)
