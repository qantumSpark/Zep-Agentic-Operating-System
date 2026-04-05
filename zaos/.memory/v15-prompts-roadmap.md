# ZAOS V1.5 — Roadmap des Prompts

> Derniere mise a jour : 2026-04-04
> Les sprints 1A-1C, 2A-2B sont termines.

## Regle d'usage

- Lancer dans l'ordre, un par un.
- Attendre la fin de chaque prompt avant de lancer le suivant.

## Ordre recommande

| # | Prompt | Statut |
|---|--------|--------|
| 1 | Cadrage global | DONE |
| 2 | Sprint 1A — Stabilisation du socle | DONE |
| 3 | Sprint 1B — Couture runtime minimale | DONE |
| 4 | Sprint 1C — Normalisation des evenements | DONE |
| 5 | Revue post-bloc 1 | DONE |
| 6 | Sprint 2A — Product Contract Layer | DONE |
| 7 | Sprint 2B — Projection workflow produit | DONE |
| 8 | Revue post-bloc 2 | DONE |
| 9 | Sprint 3A — Policy Engine contextuel | DONE |
| 10 | Sprint 3B — Hooks enrichis + UX permissions | DONE |
| 11 | Revue post-bloc 3 | DONE |
| 12 | Sprint 4A — Dashboard Decision-First | DONE |
| 13 | Sprint 4B — Personas ZAOS + session insights | DONE |
| 14 | Revue post-bloc 4 | A FAIRE |
| 15 | Post-V1.5 — Preparation Codex | A FAIRE |

---

## 1. Prompt de cadrage global

Nous allons construire ZAOS V1.5 a partir du code actuel, sans reecriture globale, en gardant Claude comme runtime actif pendant cette phase.

**Objectif produit:**
ZAOS doit evoluer d'un dashboard de dev oriente Claude Code vers un cockpit conversationnel de vibe coding avec guard rails forts, oriente idee -> design -> build -> verify -> release, ou l'utilisateur pilote surtout par intention, arbitrage, validation et preuves, sans avoir besoin de lire le code la plupart du temps.

**Objectif architecture:**
Construire V1.5 de maniere Claude-first mais agnostic-ready.
Ne pas essayer d'ajouter un second runtime maintenant.
En revanche, eviter de renforcer le couplage a Claude.
Toute nouvelle couche metier doit tendre vers une source de verite ZAOS, pas une source de verite .claude.

**Contraintes:**
- partir strictement du code actuel
- inspecter le repo avant toute modification
- ne pas faire de big-bang rewrite
- proposer un plan court avant les changements
- implementer de maniere incrementale
- verifier build/tests/checks apres les changements
- ne pas casser les flux existants si ce n'est pas necessaire
- si une decision structurelle a un cout non evident, la signaler clairement avant d'avancer

**Livrables attendus a chaque etape:**
- resume du contexte decouvert
- plan d'implementation concret
- code modifie
- verifications executees
- points de vigilance restants

**Important:**
Le but n'est pas de rendre ZAOS multi-runtime aujourd'hui.
Le but est de livrer une vraie V1.5 utile maintenant, tout en preparant proprement l'arrivee future d'un runtime Codex.

---

## 2. Sprint 1A — Stabilisation du socle

**Mission:**
Stabiliser le socle technique actuel avant toute abstraction plus profonde.

**Objectifs:**
- corriger les problemes bloquants de build frontend
- corriger les problemes critiques de securite deja presents dans le code
- fiabiliser les transitions de workflow et les garde-fous existants
- conserver le comportement global de l'application autant que possible

**Points d'attention connus a reevaluer dans le code:**
- build TypeScript frontend
- path traversal ou acces fichiers non bornes cote backend
- suppression de fichiers arbitraires via les screenshots
- surface de permissions Tauri trop large
- garde de workflow/gate insuffisante
- hook block-code incomplet
- refresh agents / watchers / stores incoherents

**Consignes:**
- commence par auditer l'etat actuel du code concerne
- propose un mini-plan d'attaque
- implemente uniquement les corrections de stabilite/securite necessaires a cette etape
- n'introduis pas encore de systeme runtime agnostic
- garde les changements cibles
- execute les verifications pertinentes a la fin

**Livrables:**
- liste des corrections effectuees
- fichiers touches
- verifications realisees
- dette restante a traiter plus tard

---

## 3. Sprint 1B — Couture runtime minimale

**Mission:**
Extraire une couture runtime minimale pour commencer a decoupler ZAOS de Claude, sans changer le comportement utilisateur et sans ajouter de second runtime.

**Objectifs:**
- introduire une abstraction interne de runtime agentique cote backend
- faire de Claude une implementation derriere cette abstraction
- conserver le fonctionnement actuel des sessions, messages, permissions et interruptions
- eviter toute reecriture globale

**Scope technique prioritaire:**
- session manager
- commands backend
- parsing d'evenements backend
- types backend lies au runtime
- bootstrap backend si necessaire

**Architecture cible:**
- une interface runtime claire
- une implementation ClaudeRuntime
- le reste du backend ne doit plus dependre directement de details Claude quand ce n'est pas necessaire

**Contraintes:**
- ne pas brancher Codex maintenant
- ne pas modifier l'UX principale a ce stade
- garder les commandes IPC existantes compatibles autant que possible
- si un renommage ou un deplacement est utile, le faire proprement et minimalement

**Livrables:**
- mini plan d'architecture
- implementation de la couture runtime
- liste des zones encore Claude-specifiques apres cette etape
- verifications de non-regression

---

## 4. Sprint 1C — Normalisation des evenements

**Mission:**
Faire en sorte que le frontend consomme un modele d'evenements ZAOS normalise, et non plus directement la forme native des evenements Claude.

**Objectifs:**
- definir un event model interne provider-neutral
- mapper les evenements Claude existants vers ce modele
- adapter les hooks/stores/frontend pour consommer ce modele
- conserver le comportement visible actuel autant que possible

**Types d'evenements internes attendus, a ajuster si necessaire:**
- message_delta
- tool_call_started
- tool_call_finished
- approval_requested
- run_state_changed
- agent_spawned
- agent_finished
- artifact_updated
- proof_added
- session_summary_ready

**Zones prioritaires:**
- types d'evenements frontend
- useStreaming
- useTauriEvents
- toute logique frontend qui suppose directement des evenements Claude

**Contraintes:**
- eviter une refonte UI a ce stade
- se concentrer sur la couche de transport / adaptation
- garder les stores coherents
- documenter clairement le mapping Claude -> ZAOS event model

**Livrables:**
- event model ZAOS mis en place
- adaptations frontend realisees
- comportement preserve
- liste des points restant encore couples au runtime Claude

---

## 5. Sprint 2A — Product Contract Layer

**Mission:**
Faire evoluer ZAOS d'une memoire centree taches techniques vers une memoire centree artefacts produit.

**Objectifs:**
- ajouter des artefacts produit de premier rang dans .memory
- etendre le backend memoire pour les lire/exposer
- etendre les stores frontend pour les consommer
- preparer l'UI a afficher un vrai contrat produit

**Artefacts a introduire:**
- product-brief
- experience-goals
- acceptance-checks
- release-readiness
- session-insights

**Intentions produit:**
- l'utilisateur doit pouvoir comprendre ce qu'on construit, pourquoi, comment on validera et ce qui reste risque sans lire le code
- current-epic ne doit plus etre la seule verite visible

**Zones prioritaires:**
- memory reader backend
- init / templates memoire
- commands backend lies a la memoire
- memoryStore
- workflowStore si necessaire

**Contraintes:**
- partir de la structure actuelle du repo
- creer des formats simples, lisibles et durables
- garder la compatibilite avec l'existant si possible
- preparer l'integration dashboard sans tout refaire maintenant

**Livrables:**
- nouveaux artefacts memoire
- backend et stores mis a jour
- explication du contrat de donnees
- verifications realisees

---

## 6. Sprint 2B — Projection workflow produit

**Mission:**
Superposer au workflow technique actuel une projection workflow produit plus naturelle pour un pilotage conversationnel.

**Objectifs:**
- conserver le pipeline technique interne existant
- exposer un workflow produit lisible:
  Imagine / Shape / Design / Build / Verify / Release / Learn
- faire apparaitre cette projection dans l'etat et dans les stores
- ameliorer la lisibilite du workflow pour un utilisateur qui ne lit pas le code

**Zones prioritaires:**
- workflow state backend
- workflow engine backend
- workflowStore frontend
- WorkflowSection
- StartupDashboard si necessaire

**Contraintes:**
- ne pas casser les transitions techniques existantes si elles servent deja
- la projection produit peut etre derivee au lieu de remplacer le pipeline brut
- toute logique de gate doit rester coherente avec cette projection

**Livrables:**
- mapping workflow technique -> workflow produit
- exposition frontend lisible
- explication des choix
- verifications

---

## 7. Sprint 3A — Policy Engine contextuel

**Mission:**
Remplacer la logique permission/garde-fous actuelle trop binaire par un Policy Engine contextuel oriente risque, intention et preuve attendue.

**Objectifs:**
- sortir de strict / accept-edits
- introduire des profils de politique
- evaluer les demandes d'action selon le contexte
- rendre les decisions explicables

**Profils suggeres:**
- observe
- guided-build
- autopilot-safe
- release-guarded

**Dimensions d'evaluation suggerees:**
- intention de l'action
- type d'action
- portee sur le repo
- phase produit
- risque securite
- risque destruction
- preuve attendue
- possibilite de rollback

**Zones prioritaires:**
- workflow state
- workflow engine
- commands backend
- nouveau module policy si pertinent

**Contraintes:**
- partir des mecanismes actuels
- garder le comportement comprehensible
- ne pas sur-complexifier inutilement
- preparer le branchement futur de plusieurs runtimes sans dependre de Claude pour la logique metier

**Livrables:**
- policy engine minimal mais reel
- nouveaux profils de policy
- integration backend
- explication de la matrice de decision
- verifications

---

## 8. Sprint 3B — Hooks enrichis + UX permissions

**Mission:**
Brancher les hooks et l'UX de permissions sur le nouveau Policy Engine afin que l'utilisateur valide une intention et un niveau de risque, pas un simple outil brut.

**Objectifs:**
- enrichir les hooks actuellement utilises
- connecter les hooks au Policy Engine
- ameliorer l'affichage des demandes de permission dans le chat
- rendre visible: intention, impact, niveau de risque, preuve attendue, recommandation

**Hooks a considerer:**
- PermissionRequest
- PostToolUse
- PostToolUseFailure
- TaskCompleted
- Stop
- SessionEnd
- FileChanged
- ReleaseGate

**Zones prioritaires:**
- zaos_hooks.rs
- .claude/settings.json
- permissionStore
- PermissionRequestBlock
- ChatPanel

**Contraintes:**
- rester compatible avec le runtime Claude actuel
- eviter les hooks decoratifs sans usage reel
- si certains hooks ne sont pas supportes tel quel, les emuler ou les compenser cote ZAOS si pertinent
- expliquer clairement les limites cote Claude actuel

**Livrables:**
- hooks enrichis
- UI de permission refondue
- journalisation plus utile des echecs et fins de session
- verifications et limites restantes

---

## 9. Sprint 4A — Dashboard Decision-First

**Mission:**
Refondre le dashboard principal pour qu'il reponde d'abord a des questions de pilotage produit et non a des questions de telemetrie developpeur.

**Objectifs UX — l'ecran principal doit repondre a:**
- qu'est-ce qu'on construit ?
- ou en est-on ?
- quelle decision attend-on de moi ?
- qu'est-ce qui prouve que c'est pret ou non ?
- qu'est-ce qui reste risque ?

**Structure cible suggeree:**
- Mission
- Decisions
- Evidence
- Progress
- Advanced

**Zones prioritaires:**
- DashboardPanel
- ChatPanel
- Memory/Product Contract area
- ActionsFeed
- ScreenshotGallery
- ComparisonView
- IterationTracker
- DiffViewer
- SessionHistory
- SessionMetrics

**Contraintes:**
- partir des composants existants autant que possible
- promouvoir les preuves visuelles et fonctionnelles
- releguer les diffs et panneaux tres techniques dans une vue avancee
- ne pas creer une UI gadget
- viser une UI reellement utile pour piloter sans lire le code

**Livrables:**
- plan rapide de reorganisation
- implementation de la nouvelle hierarchie
- preuves que l'experience est plus orientee decision
- verifications

---

## 10. Sprint 4B — Personas ZAOS + session insights

**Mission:**
Faire emerger les roles ZAOS comme concepts produit visibles et ameliorer fortement la reprise de session.

**Personas a formaliser cote ZAOS:**
- product-architect
- ux-designer
- builder
- reviewer
- tester
- playtester
- security-guardian
- release-manager

**Objectifs:**
- preparer une source de verite ZAOS pour ces roles
- limiter la dependance conceptuelle a .claude/agents
- produire de meilleurs session insights
- rendre la reprise de session orientee decisions, preuves et prochaines validations

**Zones prioritaires:**
- agents existants dans .claude/agents
- eventuelle structure .zaos/personas
- session logger
- SessionHistory
- SessionMetrics
- artefact session-insights

**Contraintes:**
- rester compatible avec Claude pour maintenant
- ne pas chercher a faire un provider compiler complet dans cette etape
- surtout preparer le terrain conceptuel et UX

**Livrables:**
- formalisation des personas ZAOS
- meilleure reprise de session
- session insights lisibles et utiles
- limites restantes avant un vrai runtime agnostic

---

## 11. Post-V1.5 — Preparation Codex

**Mission:**
Etablir et implementer la preparation minimale pour un futur second runtime, sans brancher completement Codex maintenant.

**Objectifs:**
- ajouter une notion interne de runtime registry
- preparer un selecteur de runtime ou au moins la structure necessaire
- s'assurer que la memoire, le workflow, le policy engine et l'UX principale ne dependent plus de Claude
- identifier precisement ce qui manque encore pour un futur CodexRuntime

**Contraintes:**
- ne pas essayer de supporter totalement Codex dans ce prompt
- se concentrer sur la preparation structurelle
- garder Claude pleinement fonctionnel
- documenter clairement les interfaces restantes a implementer

**Livrables:**
- runtime registry ou structure equivalente
- point precis sur ce qui est deja agnostic
- point precis sur ce qui reste provider-specific
- feuille de route concrete pour brancher Codex ensuite

---

## 12. Prompt de revue (a utiliser apres chaque gros bloc)

Fais une revue critique des changements recemment apportes a ZAOS dans le cadre de la construction de V1.5.

**Revue orientee:**
- regressions potentielles
- dette technique introduite
- incoherences avec la vision produit
- dependances encore trop fortes a Claude
- guard rails insuffisants
- trous dans la reprise de session, la memoire produit ou la preuve de qualite

**Consignes:**
- pars du code reellement present
- cite les fichiers concernes
- classe les findings par gravite
- sois concret
- si aucun probleme majeur n'est trouve, dis-le explicitement mais mentionne les risques residuels
