# Memoire Codex

## But de ZAOS

ZAOS ne doit pas devenir un simple dashboard pour Claude Code.
La vision cible est un **cockpit conversationnel de vibe coding avec guard rails forts** :

- l'utilisateur discute, arbitre, valide
- ZAOS transforme l'intention en produit
- l'utilisateur ne devrait plus avoir a lire le code dans le flux normal
- le systeme doit compenser les limites des LLM :
  - derive
  - oubli
  - confiance excessive
  - dette cachee
  - manque de discipline de delegation

Cible prioritaire :

- d'abord usage personnel
- ensuite potentiellement dev solo / indie

## Decision strategique cle

Ne pas essayer de rendre ZAOS multi-runtime tout de suite.

Strategie retenue :

1. construire une **baseline stable Claude-first**
2. developper V1.5 sur cette base
3. garder l'architecture **agnostic-ready**
4. ajouter Codex plus tard via un adapter/runtime propre

Formule retenue :

**Claude-first, architecture-agnostic-ready**

## Etat valide a ce jour

Branche courante :

- `baseline/stabilize-current`

Tag existant :

- `zaos-baseline`

Baseline validee :

- `npm run build` passe
- `cargo build` passe
- `cargo test` passe
- baseline compilee en release
- cette baseline peut servir de cockpit stable pour continuer

## Ce qui a deja ete fait

### Sprint 1A

- stabilisation du socle
- build frontend corrige
- cleanup baseline
- garde-fous ameliores partiellement

### Sprint 1B

- abstraction runtime introduite
- `SessionManager` wrappe un `AgentRuntime`
- `ClaudeRuntime` existe

Important :

- l'abstraction runtime existe mais n'est pas encore totalement neutre provider
- le backend fuit encore `CliEvent` dans la couture runtime

### Sprint 1C

- normalisation d'evenements introduite
- backend emet des `ZaosEvent`
- frontend consomme des `ZaosEvent`

Important :

- c'est suffisant pour la baseline
- mais pas encore le niveau final d'agnosticisme runtime

## Test run baseline

Le fichier source de reference est :

- [test_run_baseline.md](C:\Users\apeze\Documents\Claude\Projects\Workflow\Zep Agentic Operating System\zaos\test_run_baseline.md)

Conclusion du test :

- ZAOS a du potentiel reel
- le workflow peut aller au bout
- mais le cockpit de pilotage reste incomplet/casse sur plusieurs points critiques

Resume :

- 2 bloquants
- 7 majeurs
- 3 mineurs

## Findings les plus importants du test run

### Priorite immediate

1. bug Windows sur `switch_project`
2. aucun bouton UI pour valider le gate
3. pipeline/dashboard pas synchronises
4. `state.json` contourne au lieu de suivre le workflow normal
5. tasks du plan invisibles tant qu'elles ne sont pas `DONE`
6. mauvais agent parfois lance (`general-purpose` au lieu de `coder`)
7. corrections post-review faites par l'orchestrateur au lieu d'etre deleguees

### Frictions importantes mais secondaires

1. approbations MCP trop nombreuses
2. actions live qui n'arrivent pas vraiment en live
3. historique agent parfois mal mappe

## Diagnostic important sur le bug project switching

Symptome observe :

- la selection de dossier depuis l'app ne change pas de projet
- si on deplace l'exe dans le dossier du projet test, ZAOS fonctionne sur ce projet

Lecture retenue :

- ZAOS prend correctement le projet depuis son contexte de lancement
- mais le changement de projet est bugge/incomplet

Cause probable identifiee et deja analysee :

- sous Windows, `canonicalize()` peut produire un chemin avec prefixe `\\?\`
- `home_dir()` reste dans un format differnt
- le `starts_with()` echoue

Fix propose par Claude et valide conceptuellement :

- canonicaliser **les deux** chemins avant comparaison

Important :

- il faut aussi afficher un vrai feedback UI si `switch_project` echoue
- aujourd'hui l'erreur est surtout loggee en console

## Recommandation de sequence avant V1.5

Ne pas partir tout de suite sur Sprint 2.

Faire d'abord un **Sprint 1D de hardening base sur le test run baseline**.

Ordre recommande :

1. corriger le bug Windows `switch_project`
2. ajouter le bouton UI de validation de gate
3. corriger la synchro pipeline / dashboard / `state.json`
4. corriger l'affichage des tasks non `DONE`
5. renforcer les regles de delegation agent dans `CLAUDE.md`
6. ensuite seulement reprendre le plan V1.5 normal

## Plan V1.5 retenu

Apres le hardening 1D, reprendre :

1. `Sprint 2A` : Product Contract Layer
2. `Sprint 2B` : projection workflow produit
3. `Sprint 3A` : Policy Engine contextuel
4. `Sprint 3B` : hooks enrichis + UX permissions
5. `Sprint 4A` : dashboard decision-first
6. `Sprint 4B` : personas ZAOS + session insights
7. ensuite preparation runtime agnostic / Codex

## Point architectural a ne pas oublier

Le plus important pour l'avenir :

- ne pas recentrer ZAOS sur `.claude`
- `.claude` doit devenir une sortie / couche compatible
- la verite metier doit progressivement vivre dans `.zaos`

Structure cible a garder en tete :

- `.zaos/personas/`
- `.zaos/policies/`
- `.zaos/workflows/`
- `.zaos/runtime/`

## Point produit a ne jamais perdre

ZAOS doit privilegier :

1. intention
2. decisions
3. preuves
4. progression

Et non :

1. code brut
2. diffs comme interface principale
3. panneaux techniques au centre

Le cockpit doit repondre d'abord a :

- qu'est-ce qu'on construit ?
- ou en est-on ?
- qu'est-ce que je dois valider ?
- qu'est-ce qui prouve que c'est bon ?
- qu'est-ce qui reste risque ?

## Prompt important deja redige

Le prompt cle a lancer maintenant est un **Sprint 1D hardening**.

Objet :

- corriger les findings du test run baseline avant Sprint 2

Priorites de ce prompt :

1. fix Windows path mismatch dans `switch_project`
2. ajouter une vraie erreur UI si `switch_project` echoue
3. ajouter un bouton UI pour valider le gate
4. corriger la synchro workflow reel <-> dashboard
5. afficher les tasks du plan meme si elles ne sont pas `DONE`
6. renforcer les instructions de delegation
7. optionnel ensuite : reduire les approbations MCP

## Rappel important sur la maniere de travailler

Strategie retenue :

- utiliser **Claude Code CLI** pour travailler sur le code source
- utiliser la **baseline compilee** comme cockpit stable
- ne pas travailler "dans" le build courant
- modifier le repo, puis recompiler periodiquement

## Quand creer la branche V1.5

Pas encore.

Recommandation retenue :

- faire `Sprint 1D` sur `baseline/stabilize-current`
- retester
- si le cockpit est suffisamment fiabilise
  - soit merger vers `main`
  - soit tagger une nouvelle baseline
- puis seulement creer `v1.5`

## Points de vigilance si contexte compacté

Si je perds le contexte, retenir absolument :

1. la vision cible est **vibe coding sous controle**, pas dashboard de dev
2. la baseline `zaos-baseline` existe et est validee
3. la branche active de travail est `baseline/stabilize-current`
4. il faut faire **Sprint 1D** avant Sprint 2
5. les findings du test run baseline sont la verite terrain prioritaire
6. le prochain gros chantier V1.5 commence seulement apres ce hardening
7. l'agnosticisme runtime reste un objectif futur, pas le chantier immediat

## Fichiers de reference a relire en priorite si besoin

- [test_run_baseline.md](C:\Users\apeze\Documents\Claude\Projects\Workflow\Zep Agentic Operating System\zaos\test_run_baseline.md)
- [CLAUDE.md](C:\Users\apeze\Documents\Claude\Projects\Workflow\Zep Agentic Operating System\zaos\CLAUDE.md)
- [src-tauri/src/commands.rs](C:\Users\apeze\Documents\Claude\Projects\Workflow\Zep Agentic Operating System\zaos\src-tauri\src\commands.rs)
- [src/components/common/ProjectPicker.tsx](C:\Users\apeze\Documents\Claude\Projects\Workflow\Zep Agentic Operating System\zaos\src\components\common\ProjectPicker.tsx)
- [src/components/dashboard/PipelineSection.tsx](C:\Users\apeze\Documents\Claude\Projects\Workflow\Zep Agentic Operating System\zaos\src\components\dashboard\PipelineSection.tsx)
- [src/stores/workflowStore.ts](C:\Users\apeze\Documents\Claude\Projects\Workflow\Zep Agentic Operating System\zaos\src\stores\workflowStore.ts)
- [src-tauri/src/workflow/engine.rs](C:\Users\apeze\Documents\Claude\Projects\Workflow\Zep Agentic Operating System\zaos\src-tauri\src\workflow\engine.rs)
- [src-tauri/src/memory/reader.rs](C:\Users\apeze\Documents\Claude\Projects\Workflow\Zep Agentic Operating System\zaos\src-tauri\src\memory\reader.rs)
