# Phase 5 : Review

## Objectif
Valider la qualité du code avant de considérer l'epic comme terminée.

## Checklist de review

### Conformité
- [ ] Code conforme à `conventions.md`
- [ ] Naming cohérent avec le projet
- [ ] Patterns utilisés = patterns du projet

### Qualité
- [ ] Pas de duplication
- [ ] Edge cases gérés ou documentés en TODO
- [ ] Pas de code mort
- [ ] Commentaires utiles (pas évidents)

### Cohérence architecturale
- [ ] Respect des ADRs dans `decisions/`
- [ ] Structure fichiers cohérente avec `architecture.md`
- [ ] Pas de couplage inattendu

### Anti-hallucination
- [ ] APIs/méthodes existent réellement
- [ ] Imports référencent des fichiers existants
- [ ] Chemins et noms de ressources corrects

## Gravité des remarques
- **BLOQUANT** : à corriger avant de continuer
- **SUGGESTION** : recommandé mais non bloquant
- **NITPICK** : cosmétique

## Après la review
- Points BLOQUANTS → le Codeur corrige → re-review
- Points SUGGESTION → corriger si le temps le permet
- Quand 0 BLOQUANT restant → gate passé
