# Phase 6 : Tests & Déploiement

## Objectif
Vérifier que le code fonctionne comme prévu et ne casse rien d'existant.

## Checklist Tests

### Identification des scénarios
- [ ] Cas nominal couvert
- [ ] Cas limites identifiés (vide, null, max, min)
- [ ] Cas d'erreur identifiés
- [ ] Scénarios priorisés par risque

### Écriture
- [ ] Un test = un scénario = un comportement
- [ ] Noms de tests descriptifs
- [ ] Pas de tests vagues
- [ ] Préférence intégration > mocks quand pertinent

### Exécution
- [ ] Nouveaux tests passent
- [ ] Tests existants passent (non-régression)
- [ ] Zones non couvertes signalées

## Checklist Déploiement (si applicable)
- [ ] Build réussit sans warning
- [ ] Pas de dépendance manquante
- [ ] Config de déploiement à jour
- [ ] Testable sur l'environnement cible

## Gate
Tous les tests passent → prêt pour clôture.
