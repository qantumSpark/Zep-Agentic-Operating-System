# Epic active : Runtime auth check clarifie

> Milestone : 25 — Vague 1 Stabilisation du socle (E4)
> Statut : TERMINE

## Objectif

Clarifier le check runtime/auth pour qu'il soit honnete : distinguer "CLI presente" de "CLI authentifiee", enrichir la reponse IPC, et afficher 3 etats visuels dans le frontend.

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| 1 | Variante NotAuthenticated dans RuntimeError | `runtime/mod.rs` | DONE | |
| 2 | Split trait : check_installed() + check_auth() | `runtime/mod.rs` | DONE | |
| 3 | Impl Claude : check_installed=version, check_auth=auth status --json | `runtime/claude.rs` | DONE | parse JSON loggedIn |
| 4 | Enrichir CheckAuthResponse (ajouter cli_found) | `commands.rs` | DONE | |
| 5 | Adapter IPC + SessionManager pour les 2 methodes | `commands.rs`, `session/manager.rs` | DONE | + re-export SessionError |
| 6 | Re-check auth apres switch_project | `commands.rs` | DONE | via loadProjectContext |
| 7 | Adapter type TS + sessionStore | `sessionStore.ts` | DONE | cliFound + 4 params |
| 8 | Adapter App.tsx pour nouveau format | `App.tsx` | DONE | cli_found dans invoke |
| 9 | Re-check dans loadProjectContext | `projectLoader.ts` | DONE | ajout dans allSettled |
| 10 | StatusBar 3 etats visuels | `StatusBar.tsx` | DONE | rouge/orange/vert |
| 11 | Verification build + tests | - | DONE | 160 tests OK |
| 12 | Review B1 — Supprimer dead code cli-auth-changed | `commands.rs` | DONE | |
| 13 | Review SG1 — Timeout check_auth (10s) | `runtime/claude.rs` | DONE | |
| 14 | Review SG2 — Timeout check_installed (5s) | `runtime/claude.rs` | DONE | |
| 15 | Review SG4 — Reset cliFound dans resetSession | `sessionStore.ts` | DONE | |
