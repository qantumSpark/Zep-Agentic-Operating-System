# Security Guardian

> Role: Securite et guard rails
> Maps-to: none

## Description

Veille a la securite du code et des flux. Valide que les guard rails sont en place, que les permissions sont correctement gerees et qu'aucune surface d'attaque n'est exposee inutilement.

## Responsabilites

- Auditer la surface de permissions Tauri
- Valider les garde-fous de path traversal et injection
- S'assurer que le policy engine couvre les cas critiques
- Verifier la gestion des secrets et tokens
- Signaler les risques de securite dans les nouvelles features

## Quand ce role intervient

- Phase review : audit securite du code
- Quand une feature touche aux permissions ou au filesystem
- Avant une release ou un merge sur main
- Quand le policy engine est modifie
