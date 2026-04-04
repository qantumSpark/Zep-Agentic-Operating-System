# Backlog — Findings non bloquants Blocs 2-4

> Sources : revue post-bloc 2 (2026-04-04), review Sprint 3B (2026-04-04), review Sprint 4A (2026-04-04)
> Statut : tous non bloquants, a traiter quand pertinent

| ID | Severite | Categorie | Fichier(s) | Description | Sprint suggere |
|---|---|---|---|---|---|
| F3 | MODERE | dette | `src/types/workflow.ts` | Interface `WorkflowState` orpheline — ne correspond pas au backend DTO. Seul `BackendWorkflowPayload` est utilise. Code mort trompeur. | Nettoyage opportuniste |
| F4 | MODERE | vision | `DashboardPanel.tsx` | Product contract et workflow projection deconnectes visuellement. Pas de lien acceptance checks / gate, ni session-insights / phase produit. | Sprint 4A (Dashboard Decision-First) |
| F5 | MINEUR | vision | `models.rs`, `productContract.ts` | session-insights est singulier — un seul snapshot, pas d'historique. Limite l'utilite de la phase Learn. | Sprint 4B (Personas + session insights) |
| F6 | MODERE | dette | `src-tauri/src/memory/reader.rs` | Parsers produit silencieux sur contenu maldeforme. Heading different → struct vide sans avertissement. Impossible de distinguer "fichier vide" de "parsing rate". | Nettoyage opportuniste |
| F7 | MINEUR | dette | `src-tauri/src/workflow/state.rs` | DTO construit un WorkflowState temporaire synthetique pour deriver next_product_phase. Fonctionnel mais indirection contournee. | Pas urgent |
| F9 | MINEUR | dette | `src/stores/workflowStore.ts` | Fallback silencieux si phase inconnue — pas de log quand le backend envoie une valeur non reconnue par le frontend. | Pas urgent |
| S2-3B | MODERE | vision | `src-tauri/src/policy.rs` | `derive_action_type` : Read/Glob/Grep tombent dans ToolCall (Medium). Outils lecture seule meritent un risk Low. Envisager variant ReadOnly ou mapping explicite. | Sprint ulterieur |
| S3-3B | MINEUR | robustesse | `src-tauri/src/policy.rs` | `is_destructive_command` ne detecte pas `sudo rm`, `git push --force`, `git push -f`. Ameliorer heuristique + split par `&&`/`;`/`\|`. | Sprint ulterieur |
| S5-3B | MINEUR | coherence | `src/stores/permissionStore.ts`, `src/types/zaosEvents.ts` | `PolicyLogEntry.verdict` inclut "ask" mais ce chemin n'est jamais emprunte via policy_decision. Restreindre a "allow"/"deny" ou documenter. | Nettoyage opportuniste |
| S6-3B | MINEUR | UX | `src/components/chat/PermissionRequestBlock.tsx` | matchedRules affiches en labels techniques bruts (profile:guided-build, verdict:ask). Mapping vers labels humains a prevoir. | Sprint 4A (Dashboard) |
| S3-4A | MINEUR | perf | `PhaseIndicator.tsx` | Double indexOf inutile dans le map (idx en 2e arg de map, currentIdx recalcule 7 fois). | Nettoyage opportuniste |
| S4-4A | MINEUR | lisibilite | `ProgressGroup.tsx` | IIFE de 40 lignes dans le JSX pour tasks/progress. Extraire en sous-composant. | Nettoyage opportuniste |
| S5-4A | MINEUR | DRY | `ProgressGroup.tsx`, `ActionsFeed.tsx` | StatusIndicator dupliquee entre les deux fichiers. Extraire dans common/. | Nettoyage opportuniste |
| S6-4A | MINEUR | coherence | `MissionGroup.tsx` | Layout Brief inline (span) vs ProductSection (p + mt-0.5). Divergence intentionnelle mais non documentee. | Pas urgent |
| S7-4A | MINEUR | perf | `EvidenceGroup.tsx` | hasValidation pas memoize alors que recentScreenshots l'est. Incoherence de pattern. | Pas urgent |
