# Phase 3 : Architecture & Découpage

## Objectif
Produire un plan technique clair et un découpage en tasks exécutables par le Codeur.

## Checklist

### Architecture
- [ ] `architecture.md` a été lu (structure actuelle du projet)
- [ ] `conventions.md` a été lu (standards en place)
- [ ] Les ADRs pertinents ont été consultés
- [ ] Le plan technique est cohérent avec l'existant
- [ ] Si nouvelle décision d'archi → ADR créé avec le template
- [ ] L'utilisateur a validé le plan technique

### Découpage en tasks
- [ ] L'epic est décrite avec son objectif dans `current-epic.md`
- [ ] Chaque task touche 1 fichier (2-3 max si couplés)
- [ ] Chaque task est décrite en 1 phrase (quoi + où)
- [ ] Chaque task est vérifiable (compile, testable)
- [ ] L'ordre d'implémentation est logique (dépendances respectées)
- [ ] Aucune task n'est estimée à plus de 60 min
- [ ] L'utilisateur a validé le découpage

## Signaux d'alerte
- Une task touche 5+ fichiers → c'est une epic déguisée, re-découper
- Dépendance circulaire entre tasks → revoir l'ordre
- Aucun ADR existant pour un domaine qu'on modifie → en créer un
