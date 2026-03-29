# Pipeline de développement

> Ce fichier définit le process que l'Orchestrateur suit pour chaque demande.
> Chaque phase a un **gate** — un checkpoint de validation avant de passer à la suite.

## Phase 0 : Initialisation (automatique, chaque session)

**Qui** : Orchestrateur
**Actions** :
1. Lire `.memory/INDEX.md`
2. Lire `.memory/state.md`
3. Lire `.memory/current-epic.md` si existant
4. Si travail en cours → résumer et demander confirmation

**Gate** : aucun (automatique)

---

## Phase 1 : Compréhension

**Qui** : Orchestrateur
**Actions** :
1. Écouter la demande de l'utilisateur
2. Poser des questions pour clarifier les zones floues
3. Reformuler la demande en une phrase claire
4. Identifier le type de tâche (feature, bug fix, refactoring, recherche, question)

**Gate** : l'utilisateur valide la reformulation

---

## Phase 2 : Spécification

**Qui** : Orchestrateur + Architecte
**Actions** :
1. Rédiger une mini-spec avec le template `.workflow/templates/feature-spec.md`
2. Si besoin d'infos → déployer le Researcher
3. Définir les critères de "done"

**Gate** : l'utilisateur valide la spec

---

## Phase 3 : Architecture & Découpage

**Qui** : Architecte
**Actions** :
1. Lire la mémoire projet et les ADRs pertinents
2. Proposer un plan technique (quels fichiers, quels patterns, quelle structure)
3. Découper en tasks précises → écrire dans `.memory/current-epic.md`
4. Si nouvelle décision d'archi → créer un ADR
5. Si besoin de vérification → déployer le Researcher

**Gate** : l'utilisateur valide le plan et les tasks

---

## Phase 4 : Implémentation

**Qui** : Codeur
**Actions** :
1. Prendre la prochaine task `todo` dans `current-epic.md`
2. La marquer `in_progress`
3. Implémenter en suivant le plan et les conventions
4. Vérifier : imports, chemins, noms, APIs
5. Marquer `done` quand le code est vérifié
6. Passer à la task suivante

**Gate** : le code compile/tourne sans erreur pour chaque task

---

## Phase 5 : Review

**Qui** : Reviewer
**Actions** :
1. Relire tout le code modifié depuis le début de l'epic
2. Appliquer la checklist de review (voir `reviewer.md`)
3. Produire la liste de remarques (BLOQUANT / SUGGESTION / NITPICK)
4. Si points bloquants → le Codeur corrige, puis re-review

**Gate** : aucun point BLOQUANT restant

---

## Phase 6 : Tests

**Qui** : Testeur
**Actions** :
1. Identifier les scénarios à couvrir à partir de la spec
2. Écrire les tests
3. Exécuter tous les tests (nouveaux + existants)
4. Signaler les zones non couvertes

**Gate** : tous les tests passent

---

## Phase 7 : Clôture

**Qui** : Orchestrateur
**Actions** :
1. Mettre à jour `.memory/state.md` (avancement milestone/epic)
2. Mettre à jour `.memory/current-epic.md` (toutes tasks done → archiver)
3. Écrire le session log `.memory/sessions/YYYY-MM-DD.md`
4. Résumer ce qui a été fait et ce qui reste
5. L'utilisateur valide

**Gate** : l'utilisateur confirme la clôture

---

## Raccourcis par type de tâche

Pas besoin de passer par les 7 phases à chaque fois. L'Orchestrateur adapte :

| Type de tâche | Phases à suivre |
|---|---|
| Feature complète | 0 → 1 → 2 → 3 → 4 → 5 → 6 → 7 |
| Bug fix simple | 1 → 4 → 5 → 7 |
| Refactoring | 1 → 3 → 4 → 5 → 7 |
| Question / explication | Orchestrateur répond directement |
| Recherche pure | 1 → Researcher → 7 |
| Continuation d'epic | 0 → 4 → (5 → 6 si fin d'epic) → 7 |

## Règle d'or

**Ne jamais sauter un gate sans l'accord explicite de l'utilisateur.**
En cas de doute sur la phase à suivre → demander à l'utilisateur.
