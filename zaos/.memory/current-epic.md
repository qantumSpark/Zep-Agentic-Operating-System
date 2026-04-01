# Epic active : Phase 9.2.1+9.3.1 — Agents Unifies + Dashboard Temps Reel

> Milestone : 9 — Field-Tested Corrections
> Statut : TERMINE

## Objectif

Fusionner Workflow Agents + Delegations en une section unifiee avec mapping de noms et pastille active. Ajouter compteur de progression epic au dashboard.

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| 1 | Fix dedup addDelegation | `agentsStore.ts` | DONE | Check ID avant push |
| 2 | Capturer prompt + mapping agent | `agentsStore.ts`, `useStreaming.ts` | DONE | Regex .claude/agents/ + keywords fallback |
| 3 | UnifiedAgentsSection | `UnifiedAgentsSection.tsx` | DONE | Merge agents + delegations, pastille verte, tri running-first |
| 4 | Remplacer dans DashboardPanel | `DashboardPanel.tsx` | DONE | 2 sections → 1, supprime fichiers orphelins |
| 5 | Mapping dans SessionMetrics | `SessionMetrics.tsx` | DONE | Agent deja mappe via mappedAgent, capitalize |
| 6 | Compteur progression epic | `MemorySection.tsx` | DONE | Barre verte + X/Y tasks + pourcentage |
| 7 | Badges statut verifies | `StatusBadge.tsx` | DONE | Deja correct (DONE=vert, EN COURS=bleu, BLOQUE=rouge) |
