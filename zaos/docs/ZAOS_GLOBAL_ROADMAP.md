# ZAOS - Roadmap Globale Vers La Version Finale

> Statut: roadmap strategique
> Objet: organiser l'evolution du code existant vers la vision finale de ZAOS
> Contrainte directrice: rendre ZAOS utilisable pour developper ZAOS lui-meme le plus tot possible

---

## 1. Role de ce document

Ce document ne remplace pas la roadmap historique d'implementation du repo.

Il la complete avec une **roadmap de transformation produit/systeme**:

- a partir du code actuel
- alignee sur la vision finale de ZAOS
- ordonnee pour maximiser au plus vite la capacite de self-development

Roadmap historique de reference:

- [`ROADMAP.md`](../ROADMAP.md)

Documents de vision qui servent de cible:

- [Product Vision](./PRODUCT_VISION_FINAL.md)
- [Ultimate Cockpit](./ZAOS_ULTIMATE_COCKPIT.md)
- [Ultimate Workflow](./ZAOS_ULTIMATE_WORKFLOW.md)
- [Ultimate Memory](./ZAOS_ULTIMATE_MEMORY.md)
- [Ultimate Validation](./ZAOS_ULTIMATE_VALIDATION.md)
- [Ultimate Security](./ZAOS_ULTIMATE_SECURITY.md)
- [Ultimate Policy System](./ZAOS_ULTIMATE_POLICY_SYSTEM.md)
- [Ultimate Artifacts](./ZAOS_ULTIMATE_ARTIFACTS.md)
- [Ultimate Resumption](./ZAOS_ULTIMATE_RESUMPTION.md)
- [Ultimate Runtime](./ZAOS_ULTIMATE_RUNTIME.md)
- [Ultimate Release / Maintenance](./ZAOS_ULTIMATE_RELEASE_MAINTENANCE.md)

---

## 2. Principe directeur

L'ordre de travail ne doit pas viser la vision finale complete d'un seul bloc.

L'ordre optimal est:

1. stabiliser le noyau
2. rendre ZAOS utile pour developper ZAOS lui-meme
3. renforcer la preuve, la reprise et la gouvernance
4. etendre progressivement vers la vision finale complete

Le but n'est donc pas:

- d'ajouter le plus de features possible le plus vite possible

Le but est:

- d'obtenir le plus vite possible un **ZAOS Bootstrap Self-Dev**
- suffisamment fiable pour piloter son propre chantier
- puis de le faire monter vers un cockpit professionnel complet

Cette logique est directement alignee avec:

- [Product Vision](./PRODUCT_VISION_FINAL.md)
- [Ultimate Workflow](./ZAOS_ULTIMATE_WORKFLOW.md)
- [Ultimate Runtime](./ZAOS_ULTIMATE_RUNTIME.md)
- [Ultimate Validation](./ZAOS_ULTIMATE_VALIDATION.md)

---

## 3. Etat de depart

Le code existant fournit deja une base serieuse:

- shell desktop Tauri + React
- backend Rust structure
- workflow engine
- memory reader
- policy engine
- session manager / runtime adapter Claude
- screenshots
- MCP embarque
- stores frontend et dashboard deja larges

Mais l'existant n'est pas encore au niveau de la vision finale sur plusieurs axes structurants:

- verite live coherentement synchronisee
- multi-projet robuste
- artefacts de session / task / gate suffisamment solides
- validation et preuve encore trop faibles pour une confiance maximale
- separation encore incomplete entre cockpit, control plane, memoire et runtime

Cette roadmap part donc d'un constat simple:

- **on ne repart pas de zero**
- **on fiabilise, puis on recentre, puis on etend**

---

## 4. Ordre strategique recommande

Ordre recommande des vagues:

1. Vague 1 - Stabilisation du socle
2. Vague 2 - Boucle self-dev minimale
3. Vague 3 - Preuve et validation reelles
4. Vague 4 - Cockpit decisionnel V1
5. Vague 5 - Memoire gouvernee V1
6. Vague 6 - Policy, hooks et approvals systeme
7. Vague 7 - Runtime / control plane V1
8. Vague 8 - Securite, release, maintenance, supportabilite

Pourquoi cet ordre:

- V1 rend ZAOS fiable
- V2 le rend utile pour son propre developpement
- V3 le rend plus trustworthy
- V4 augmente la vitesse de supervision
- V5 reduit la derive longue
- V6 et V7 installent la gouvernance et le control plane
- V8 amene le niveau professionnel complet

---

## 5. Jalon cle a viser en premier

Le premier grand jalon cible n'est pas "version finale".

Le premier grand jalon cible est:

## ZAOS Bootstrap Self-Dev

Definition:

- un projet actif fiable
- un workflow task/gate vraiment exploitable
- session snapshot + handoff utiles
- evidence minimale obligatoire
- review/test visibles
- cockpit assez clair pour piloter le chantier ZAOS
- policies simples mais effectives

Ce jalon est la priorite produit numero 1.

Il est aligne surtout avec:

- [Product Vision](./PRODUCT_VISION_FINAL.md)
- [Ultimate Workflow](./ZAOS_ULTIMATE_WORKFLOW.md)
- [Ultimate Validation](./ZAOS_ULTIMATE_VALIDATION.md)
- [Ultimate Resumption](./ZAOS_ULTIMATE_RESUMPTION.md)

---

## 6. Vague 1 - Stabilisation Du Socle

### 6.1 Objectif

Obtenir un ZAOS qui ne ment pas sur son etat et reste coherent quand on l'utilise tous les jours.

### 6.2 Livrables

- correction du cycle de vie du projet actif
- `switch_project` fiable avec rechargement reel du workflow du projet cible
- synchronisation correcte workflow/runtime/permission mode
- MCP server aligne avec le projet actif
- correction des watchers et des read models frontend defectueux
- clarification du check runtime/auth
- hygiene repo de base: ignore des artefacts generes, separation source/etat
- remise au propre du socle Rust: `fmt`, `clippy`, warnings a zero sur le noyau

### 6.3 Sous-systemes

- runtime
- session manager
- workflow state
- watchers
- MCP embarque
- chargement projet frontend
- conventions de stockage

### 6.4 Documents de vision les plus lies

- [Ultimate Runtime](./ZAOS_ULTIMATE_RUNTIME.md)
- [Ultimate Resumption](./ZAOS_ULTIMATE_RESUMPTION.md)
- [Ultimate Policy System](./ZAOS_ULTIMATE_POLICY_SYSTEM.md)

### 6.5 Ce qu'on differe volontairement

- graph memory
- multi-runtime complet
- cockpit final complet
- release/ops avance

### 6.6 Critere de sortie

- changement de projet fiable
- etat cockpit coherent apres boot et apres switch
- MCP et runtime suivent correctement le projet courant
- socle backend propre en build/test/lint/format
- disparition des divergences majeures entre verite backend et verite UI

### 6.7 Valeur self-dev

Sans cette vague, ZAOS reste instable comme outil de travail pour se construire lui-meme.

---

## 7. Vague 2 - Boucle Self-Dev Minimale

### 7.1 Objectif

Creer le premier vrai **ZAOS Bootstrap Self-Dev** exploitable.

### 7.2 Livrables

- tache active clairement identifiee
- task card minimale mais stable
- session snapshot utile
- handoff artifact simple
- boucle de travail courte:
  comprehension -> action -> review -> test -> gate -> closure
- liaison explicite entre tache, session, gate et preuves
- cockpit recentre sur les decisions utiles pour le chantier ZAOS

### 7.3 Sous-systemes

- workflow
- memory canonique legere
- session logging
- dashboard decision-first minimum
- actions / agents / evidence feeds

### 7.4 Documents de vision les plus lies

- [Ultimate Workflow](./ZAOS_ULTIMATE_WORKFLOW.md)
- [Ultimate Cockpit](./ZAOS_ULTIMATE_COCKPIT.md)
- [Ultimate Artifacts](./ZAOS_ULTIMATE_ARTIFACTS.md)
- [Ultimate Resumption](./ZAOS_ULTIMATE_RESUMPTION.md)

### 7.5 Ce qu'on differe volontairement

- orchestration multi-persona sophistiquee
- replay avance
- anti-cheat complexe
- validation security-specifique

### 7.6 Critere de sortie

- on peut piloter une task ZAOS dans ZAOS sans se perdre
- chaque session laisse un snapshot exploitable
- chaque task a un prochain pas clair
- le gate n'est plus purement decoratif
- la reprise apres interruption devient rapide

### 7.7 Valeur self-dev

C'est la vague qui transforme ZAOS en outil reel pour accelerer son propre developpement.

---

## 8. Vague 3 - Preuve Et Validation Reelles

### 8.1 Objectif

Faire en sorte que ZAOS ne puisse pas se declarer "done" sur simple narration.

### 8.2 Livrables

- evidence pack minimal par task
- fermeture de task conditionnee a des preuves
- traces reliees aux decisions de closure
- validation deterministe prioritaire
- boucle echec -> diagnostic -> correction -> revalidation
- premiers anti-cheat checks
- escalation apres boucles repetees sans progres

### 8.3 Sous-systemes

- validation
- traces et logs
- gates
- session artifacts
- orchestration de tests

### 8.4 Documents de vision les plus lies

- [Ultimate Validation](./ZAOS_ULTIMATE_VALIDATION.md)
- [Ultimate Artifacts](./ZAOS_ULTIMATE_ARTIFACTS.md)
- [Ultimate Workflow](./ZAOS_ULTIMATE_WORKFLOW.md)

### 8.5 Ce qu'on differe volontairement

- clean-room review avancee
- eval harness complexe
- dataset export complet

### 8.6 Critere de sortie

- une task ne peut plus se fermer sans preuve
- les preuves sont visibles depuis le cockpit
- les retries sans diagnostic sont freines
- les claims de validation sont tracables

### 8.7 Valeur self-dev

Cette vague augmente fortement la fiabilite du developpement de ZAOS par ZAOS.

---

## 9. Vague 4 - Cockpit Decisionnel V1

### 9.1 Objectif

Passer d'un dashboard de suivi a un vrai poste de pilotage.

### 9.2 Livrables

- `Overview Strip`
- `Decision Queue`
- `Evidence Lane`
- `Agent Lane`
- `Risk Lane`
- hierarchie stable:
  story -> proof -> trace
- approvals, gates, risks, evidence et sessions comme objets de premier rang
- read models backend dedies au cockpit

### 9.3 Sous-systemes

- dashboard frontend
- backend read models
- aggregations sessions / evidence / risk / approvals

### 9.4 Documents de vision les plus lies

- [Ultimate Cockpit](./ZAOS_ULTIMATE_COCKPIT.md)
- [Ultimate Artifacts](./ZAOS_ULTIMATE_ARTIFACTS.md)
- [Ultimate Resumption](./ZAOS_ULTIMATE_RESUMPTION.md)
- [Product Vision](./PRODUCT_VISION_FINAL.md)

### 9.5 Ce qu'on differe volontairement

- analytics avancees
- replay multi-branches riche
- polish final global

### 9.6 Critere de sortie

- la prochaine decision utile est visible sans lire tout le transcript
- les preuves precedent la confiance
- les risques ouverts sont visibles et actionnables
- les sessions sont une vraie unite de supervision

### 9.7 Valeur self-dev

Le cout cognitif pour piloter le chantier ZAOS baisse fortement.

---

## 10. Vague 5 - Memoire Gouvernee V1

### 10.1 Objectif

Transformer la memoire actuelle en vrai systeme anti-drift.

### 10.2 Livrables

- ownership definitive:
  `.memory/` canonique, `.zaos/` operationnel
- core memory blocks injectables
- regles simples de promotion operationnel -> canonique
- retrieval ciblee par phase / task / persona / risk
- memory health utile:
  stale, contradiction, mismatch, manque de preuve, drift
- separation nette live state vs snapshot vs archive

### 10.3 Sous-systemes

- memoire
- workflow
- session artifacts
- indexes et retrieval operationnels

### 10.4 Documents de vision les plus lies

- [Ultimate Memory](./ZAOS_ULTIMATE_MEMORY.md)
- [Ultimate Resumption](./ZAOS_ULTIMATE_RESUMPTION.md)
- [Ultimate Artifacts](./ZAOS_ULTIMATE_ARTIFACTS.md)
- [Product Vision](./PRODUCT_VISION_FINAL.md)

### 10.5 Ce qu'on differe volontairement

- graph memoire riche
- reflection automatique avancee
- temporal graph complet

### 10.6 Critere de sortie

- la memoire reduit reellement l'amnesie
- la reprise n'exige plus d'archeologie transcript
- les contradictions importantes remontent
- les documents canoniques ont une gouvernance claire

### 10.7 Valeur self-dev

ZAOS garde mieux le cap sur sa vision et ses decisions quand il se developpe lui-meme sur la duree.

---

## 11. Vague 6 - Policy, Hooks Et Approvals Systeme

### 11.1 Objectif

Faire appliquer les regles au runtime au lieu de seulement les suggerer.

### 11.2 Livrables

- stack policy claire:
  guidance, scoped rules, hooks, policy-as-code, approvals
- permissions par phase
- permissions par persona
- chemins et fichiers proteges
- safe-output boundary pour mutations sensibles
- gouvernance MCP par projet et niveau de risque
- audit des decisions allow / ask / deny

### 11.3 Sous-systemes

- policy engine
- hooks
- approvals
- runtime permissions
- gouvernance MCP

### 11.4 Documents de vision les plus lies

- [Ultimate Policy System](./ZAOS_ULTIMATE_POLICY_SYSTEM.md)
- [Ultimate Security](./ZAOS_ULTIMATE_SECURITY.md)
- [Ultimate Runtime](./ZAOS_ULTIMATE_RUNTIME.md)
- [Ultimate Workflow](./ZAOS_ULTIMATE_WORKFLOW.md)

### 11.5 Ce qu'on differe volontairement

- policy-as-code trop ambitieuse trop tot si elle ralentit le bootstrap
- securite metier profonde avant que les bases soient stabilisees

### 11.6 Critere de sortie

- les contraintes critiques sont reellement executees
- les approvals sont durables et auditables
- les personas ont de vraies frontieres
- MCP n'est plus un angle mort

### 11.7 Valeur self-dev

ZAOS peut contribuer a son propre code avec moins de risque de derive ou d'action dangereuse.

---

## 12. Vague 7 - Runtime / Control Plane V1

### 12.1 Objectif

Faire evoluer le backend d'un runtime integre vers un vrai control plane agentique.

### 12.2 Livrables

- distinction claire:
  orchestrator, adapters, session manager, event log, policy layer, safe writer, observers
- identites stables:
  project, workflow, epic, task, session, run, subagent, checkpoint, approval, trace
- append-only event log exploitable
- checkpoints machine-resumables
- replay distinct du resume
- observers read-only pour alimenter le cockpit
- Claude-first solide mais architecture non verrouillee

### 12.3 Sous-systemes

- runtime
- sessions
- eventing
- MCP
- observers et read models

### 12.4 Documents de vision les plus lies

- [Ultimate Runtime](./ZAOS_ULTIMATE_RUNTIME.md)
- [Ultimate Resumption](./ZAOS_ULTIMATE_RESUMPTION.md)
- [Ultimate Artifacts](./ZAOS_ULTIMATE_ARTIFACTS.md)
- [Ultimate Workflow](./ZAOS_ULTIMATE_WORKFLOW.md)

### 12.5 Ce qu'on differe volontairement

- multi-provider complet
- infra distribuee lourde
- branching/fork avance si non necessaire a court terme

### 12.6 Critere de sortie

- le runtime ne melange plus orchestration, mutation et supervision
- les evenements deviennent la vraie source des vues
- resume et replay sont propres
- les subagents ont une identite et une portee reelles

### 12.7 Valeur self-dev

ZAOS devient plus robuste pour des chantiers longs et complexes sur lui-meme.

---

## 13. Vague 8 - Securite, Release, Maintenance, Supportabilite

### 13.1 Objectif

Atteindre le niveau "professionnel et stable" de la vision finale.

### 13.2 Livrables

- security workflow dedie
- validation renforcee pour auth / payments / webhooks / secrets / deploy
- release readiness gate complet
- watch window post-deploiement
- incident loop
- maintenance loop causal
- support handoff artifact
- maintenance memory
- ownership post-release
- feedback loop incident -> validation -> policy -> memory

### 13.3 Sous-systemes

- security
- validation avancee
- release readiness
- maintenance
- supportability artifacts

### 13.4 Documents de vision les plus lies

- [Ultimate Security](./ZAOS_ULTIMATE_SECURITY.md)
- [Ultimate Validation](./ZAOS_ULTIMATE_VALIDATION.md)
- [Ultimate Release / Maintenance](./ZAOS_ULTIMATE_RELEASE_MAINTENANCE.md)
- [Ultimate Artifacts](./ZAOS_ULTIMATE_ARTIFACTS.md)
- [Product Vision](./PRODUCT_VISION_FINAL.md)

### 13.5 Ce qu'on differe volontairement

- seulement les raffinements non bloquants une fois la capacite de livraison pro acquise

### 13.6 Critere de sortie

- ZAOS peut accompagner un projet jusqu'a la livraison et l'apres-livraison
- les operations sensibles passent par des chemins dedies
- incidents et maintenance nourrissent le systeme
- le produit est apte a une livraison professionnelle confiante

### 13.7 Valeur self-dev

ZAOS peut non seulement se developper lui-meme, mais aussi se durcir lui-meme.

---

## 14. Jalons

### Jalon 1 - Bootstrap Stable

Fin de la Vague 1.

Definition:

- ZAOS est coherent et fiable comme outil quotidien

### Jalon 2 - Self-Dev Usable

Fin de la Vague 2.

Definition:

- ZAOS peut deja piloter son propre chantier avec une boucle de travail courte

### Jalon 3 - Self-Dev Trustworthy

Fin de la Vague 3.

Definition:

- ZAOS peut s'aider lui-meme sans trop de faux positifs de validation

### Jalon 4 - Decision Cockpit

Fin de la Vague 4.

Definition:

- le chantier est pilotable proprement par decisions, preuves et risques

### Jalon 5 - Governed Platform

Fin des Vagues 5 a 7.

Definition:

- ZAOS ressemble vraiment a sa vision coeur:
  memoire gouvernee, policy executable, runtime de control plane

### Jalon 6 - Professional Delivery

Fin de la Vague 8.

Definition:

- ZAOS atteint le niveau professionnel/stable vise par les documents de vision

---

## 15. Regles de priorisation

Quand deux chantiers sont en concurrence, prioriser selon cet ordre:

1. ce qui augmente la fiabilite du noyau
2. ce qui augmente la capacite de self-development
3. ce qui augmente la qualite de preuve et de reprise
4. ce qui augmente la gouvernance
5. ce qui augmente la sophistication structurelle
6. le polish

Donc:

- la boucle task -> proof -> gate -> closure passe avant le graph memory
- la reprise session fiable passe avant le cockpit "parfait"
- les artefacts et read models passent avant les analytics
- Claude-first solide passe avant le multi-provider

---

## 16. Prochaine traduction operationnelle

La suite logique de ce document est:

- decliner la Vague 1 et la Vague 2 en epics concrets
- ordonner les dependances
- definir les criteres d'acceptation
- construire le backlog du jalon **Self-Dev Usable**

Ce jalon doit rester la priorite numero 1 tant qu'il n'est pas atteint.
