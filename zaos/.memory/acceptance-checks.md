# Acceptance Checks

## Criteres

| # | Check | Statut | Notes |
|---|-------|--------|-------|
| 1 | Chat send/receive streaming avec Claude CLI | OK | Teste en test run 1 et 2 |
| 2 | Pipeline workflow 7 phases avec gates enforced | OK | Gates phase-aware depuis test run 2 |
| 3 | Dashboard temps reel (workflow, agents, actions) | OK | Decision-first layout depuis Sprint 4A |
| 4 | Policy engine avec 4 profils contextuels | OK | observe, guided-build, autopilot-safe, release-guarded |
| 5 | Permissions inline approve/deny dans le chat | OK | Remplace --dangerously-skip-permissions |
| 6 | Memory persistence entre sessions | OK | state.md, current-epic.md, product contract |
| 7 | Product contract affiche dans le dashboard | OK | Brief, goals, checks, readiness, insights |
| 8 | Runtime abstraction (RuntimeKind, RuntimePaths) | OK | Seams documentes, Claude comme implementation |
| 9 | MCP server zaos-ide operationnel | OK | notify, screenshot, get_ui_state, show_diff |
| 10 | Session insights capture automatique | PARTIEL | Structure existe, mais editorial/decisions jamais popules |
| 11 | Tests unitaires sur les paths critiques | A FAIRE | RuntimePaths, serialisation, helpers frontend manquants |
| 12 | Reprise de session robuste apres crash | A VERIFIER | Pas teste systematiquement |

## Validations manuelles

- Test run reel avec un projet complet de A a Z (pas juste ZAOS lui-meme)
- Verifier la reprise de session apres fermeture inattendue
- Verifier qu'un guard rail bloque effectivement une action destructive reelle (rm, force push)
- Verifier que le dashboard est comprehensible sans contexte technique prealable
- Verifier que le switch de projet preserve l'etat de chaque projet independamment
