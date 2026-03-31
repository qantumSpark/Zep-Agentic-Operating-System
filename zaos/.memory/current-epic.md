# Epic active : Phase 5 — UX Polish

> Milestone : 5 — UX Polish
> Date de debut : 2026-03-31
> Statut : EN COURS

## Objectif

Polir l'experience utilisateur de ZAOS : historique de sessions, metriques, notifications natives, raccourcis clavier, theme clair/sombre, et retour visuel d'activite en temps reel.

## Architecture

4 streams paralleles, 12 taches :

- **Stream A (Sessions & Metriques)** : T1 → T2 → T3
- **Stream B (Dashboard & Notifications)** : T4 → T5
- **Stream C (Input & Theme)** : T6 → T7
- **Stream D (Activity Feedback)** : T8 → T9 → T10 → T11 → T12

### Corrections post-verification

- Notifications : `tauri-plugin-notification` (pas browser Notification API — ne marche pas dans WebView2/Windows)
- Glow streaming : pseudo-element + `opacity` animation (GPU-accelerated, pas `box-shadow`)
- Theme persistence : Zustand `persist` middleware (pas localStorage manuel)

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| T1 | Historique de sessions (navigation, recherche) | `SessionHistory.tsx`, `sessionStore.ts`, `DashboardPanel.tsx` | TODO | Utilise list_sessions IPC existant |
| T2 | Metriques par session (tokens/phase, temps/agent) | `SessionMetrics.tsx`, `sessionStore.ts`, `useTauriEvents.ts`, `useStreaming.ts`, `DashboardPanel.tsx` | TODO | Record phase tokens + agent timings |
| T3 | Auto-generation session logs `.memory/sessions/` | `session/logger.rs`, `session/mod.rs`, `commands.rs`, `main.rs`, `useTauriEvents.ts` | TODO | IPC save_session_log, tokio::fs |
| T4 | Dashboard au startup (resume projet) | `StartupDashboard.tsx`, `ChatPanel.tsx` | TODO | Remplace "No messages" par resume |
| T5 | Notifications natives pour gates et taches longues | `useTauriEvents.ts`, `App.tsx`, `tauri.conf.json` | TODO | tauri-plugin-notification |
| T6 | Raccourcis clavier | `useKeyboardShortcuts.ts`, `App.tsx`, `InputBar.tsx` | TODO | Ctrl+N focus, Ctrl+K clear, Ctrl+G gate |
| T7 | Theme system (dark/light) | `themeStore.ts`, `app.css`, `App.tsx`, `StatusBar.tsx` | TODO | CSS variables + Zustand persist |
| T8 | Heartbeat visuel StatusBar | `StatusBar.tsx` | TODO | Pulsing dot quand CLI actif |
| T9 | "Last seen Xs ago" sur agents | `agentsStore.ts`, `AgentsSection.tsx`, `useStreaming.ts` | TODO | lastSeenAt + ticking display |
| T10 | Stream preview dans StatusBar | `chatStore.ts`, `StatusBar.tsx` | TODO | Derniere ligne tronquee |
| T11 | Glow sur ChatPanel pendant streaming | `ChatPanel.tsx`, `app.css` | TODO | pseudo-element + opacity animation |
| T12 | Sub-agent tool_use dans ActionsFeed | `actionsStore.ts`, `useStreaming.ts`, `ActionsFeed.tsx` | TODO | parentId + indentation CSS |

## Dependencies nouvelles

- `tauri-plugin-notification` (T5)
- Aucune dep npm nouvelle
