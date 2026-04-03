# Vision Memoire ZAOS

## Statut

Idee de long terme.
Ne pas lancer ce chantier maintenant.

Le chantier prioritaire reste :

- correction des findings du `test_run_baseline.md`
- puis reprise du plan `V1.5`

## Position retenue

La proposition "Memory V2" est une **bonne direction long terme**, mais **trop ambitieuse comme chantier immediat**.

Verdict honnete :

- bonne architecture cible
- mauvaise premiere etape si on essaie de tout construire d'un bloc

## Idees a conserver absolument

### 1. Separation canonique / operationnel

Conserver cette distinction comme fondation :

- `.memory/` = memoire canonique, humaine, officielle
- `.zaos/memory/` = memoire operationnelle, journaux, index, extraction, audit

C'est probablement l'idee la plus forte du document.

### 2. Markdown comme source de verite humaine

Ne pas remplacer la memoire officielle par une DB opaque.

Le canonique doit rester :

- lisible
- editable
- diffable
- stable

### 3. Retrieval avec confiance et provenance

ZAOS ne doit pas seulement "retrouver du texte".
Il doit savoir :

- si l'information est canonique ou historique
- si elle est fraiche ou stale
- d'ou elle vient
- si elle est confirmee ou contradictoire

### 4. Audit / drift

Bonne direction pour ZAOS :

- derive doc <-> code
- derive workflow <-> memoire
- contradictions
- staleness

Mais a introduire progressivement.

## Ce qu'il ne faut pas faire maintenant

Ne pas lancer tout de suite :

- semantic search complet
- promotion queue complexe
- memory health qui bloque des gates
- router de recall tres sophistique
- systeme trop vaste de types memoire
- gros chantier "plateforme memoire"

Raison :

- ZAOS n'a pas encore fini de fiabiliser son cockpit
- le plan V1.5 reste prioritaire
- trop de complexite maintenant ralentirait le produit

## Forme recommandee

Transformer l'idee "Memory V2" en deux niveaux :

### Niveau 1

**Vision long terme**

Garder le document original comme North Star.

### Niveau 2

**Memory V2 Lite**

Version realiste a faire plus tard, incrementalement.

## Memory V2 Lite — version recommandee

Si on veut preparer le terrain sans deriver, la bonne version initiale serait :

1. `MemoryService`
2. `MemorySnapshot`
3. journal brut minimal
4. recherche lexicale + metadata
5. audit leger
6. memory health en signal, pas en blocage

## MVP memoire recommande plus tard

### 1. MemoryService

Faire de `MemoryReader` un parser bas niveau, et non plus le centre logique.

Nouvelle facade cible :

- `MemoryService`

### 2. Canonical snapshot

Construire un snapshot compact de la memoire officielle pour :

- dashboard
- bootstrap de session
- injection de contexte

### 3. Raw journal

Commencer a ecrire un journal brut sous :

- `.zaos/memory/events/`

Il s'agit d'une base pour plus tard, pas encore d'un moteur complet.

### 4. Lexical + metadata retrieval

Commencer simple :

- recherche texte
- filtres
- types
- provenance
- date
- phase

Pas besoin d'embeddings au debut.

### 5. Audit leger

Au debut, seulement quelques checks utiles :

- derive workflow <-> memoire
- fraicheur du canonique
- references mortes / chemins invalides

### 6. Memory health comme signal

Montrer :

- badge
- warning
- score simple

Mais **ne pas bloquer** les gates au debut.

## Integrations futures a preparer pendant V1.5

Sans lancer Memory V2 maintenant, il faudra se souvenir de preparer le terrain dans certains sprints.

### Sprint 2A

A preparer :

- facade `MemoryService`
- `MemorySnapshot`
- distinction conceptuelle canonique / operationnel

### Sprint 2B

A preparer :

- place pour un signal leger de fraicheur / sante memoire

### Sprint 3B

A preparer :

- journal brut minimal de certains evenements/session sous `.zaos/memory/events/`

### Sprint 4B

A preparer :

- `session-insights` structures proprement pour pouvoir plus tard alimenter :
  - extraction
  - recall
  - audit

## Principe a ne pas oublier

La memoire ultime n'est pas juste une meilleure recherche.

Le vrai but pour ZAOS est :

- distinguer la verite du projet de l'historique
- fournir du contexte avec confiance
- garder des preuves
- detecter la derive
- aider le workflow sans le bureaucratiser

## Ordre recommande

Ordre a conserver :

1. finir le hardening courant
2. finir la trajectoire V1.5
3. ensuite seulement lancer "Memory V2 Lite"
4. garder "Memory V2 complet" comme horizon plus lointain

## Rappel tres important

Si le contexte est compacte plus tard :

- ne pas lancer Memory V2 complet maintenant
- garder cette vision comme reference
- injecter seulement les fondations dans certains sprints V1.5
- ne pas laisser la memoire devenir prioritaire avant que le cockpit soit fiable
