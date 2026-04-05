# Experience Goals

## Qualites cibles

| # | Qualite | Critere | Priorite |
|---|---------|---------|----------|
| 1 | Fiabilite workflow | Le pipeline end-to-end ne perd pas d'etat entre sessions ni apres crash | P0 |
| 2 | Reactivite dashboard | Le dashboard reflete l'etat reel en moins de 2 secondes | P0 |
| 3 | Guard rails efficaces | Les permissions bloquent les actions destructives sans bloquer le flow normal | P0 |
| 4 | Memoire durable | Le product contract et les session insights survivent entre sessions et projets | P1 |
| 5 | Lisibilite pilotage | L'utilisateur comprend ou il en est et quelle decision on attend de lui sans lire le code | P1 |
| 6 | Qualite vendable | Chaque feature finie est polie, robuste et testee — pas juste fonctionnelle | P1 |
| 7 | Extensibilite runtime | Ajouter un nouveau runtime (Codex, autre LLM) ne casse pas l'existant | P2 |

## Standards UX

- Pilotage par intention et validation, pas par commande technique
- Le dashboard repond a 5 questions : quoi, ou en est-on, quelle decision, quelle preuve, quel risque
- Feedback immediat sur chaque action utilisateur (pas de silence apres clic)
- Erreurs explicables en langage produit, pas en codes techniques
- Les phases produit (Imagine, Shape, Build...) sont le vocabulaire principal, pas les phases techniques

## Anti-patterns

- Terminal brut deguise en app desktop
- Dashboard telemetrie dev au lieu de cockpit de decision
- Guard rails qui bloquent le flow au lieu de le securiser
- Features decoratives sans usage reel en session
- Silence sur erreur (parsers silencieux, insights vides, gates fantomes)
- Accumuler de la dette technique non-bloquante sans jamais la traiter
