# Backlog — Findings non bloquants Blocs 2-4 + Post-V1.5

> Sources : revue post-bloc 2 (2026-04-04), review Sprint 3B (2026-04-04), review Sprint 4A (2026-04-04), review Post-V1.5 (2026-04-05)
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
| m4-4B | MINEUR | fragilite | `reader.rs` | `read_personas` navigue via `memory_dir.parent()` au lieu d'un champ `project_dir` explicite. Fragile si `MemoryReader` change. | Nettoyage opportuniste |
| S8-4B | SUGGESTION | clarte | `useTauriEvents.ts` | `editorial` toujours vide dans l'appel `save_session_insights`. Ajouter un TODO expliquant d'ou viendront les editoriaux. | Pas urgent |
| S9-4B | SUGGESTION | robustesse | `session/logger.rs` | `is_meaningful_insights` utilise OR (metadata OU item). Pourrait generer des archives quasi-vides. Envisager AND. | Pas urgent |
| M1-B4 | MODERE | fonctionnel | `useTauriEvents.ts`, `SessionInsightsBlock.tsx` | Editorial toujours vide dans save_session_insights — le bloc insights ne montre que des metadonnees. Besoin d'un mecanisme pour peupler decisions/risks/validations. | Post-V1.5 prioritaire |
| M2-B4 | MODERE | timing | `useTauriEvents.ts`, `watchers/service.rs` | ProductContract non force-refreshe apres write_session_insights. Depend du debounce watcher (~300ms). | Nettoyage opportuniste |
| M3-B4 | MODERE | coherence | `DecisionsGroup.tsx`, `ProgressGroup.tsx`, `MemorySection.tsx` | Listes de statuts "done" dupliquees et divergentes (7 vs 4 valeurs, case differ). Extraire helper isDoneStatus(). | Nettoyage opportuniste |
| M1-PV | MINEUR | cosmétique | `commands.rs` | Step numbering dans switch_project corrige partiellement par B1. Revoir la numerotation globale. | Pas urgent |
| M3-PV | MINEUR | extensibilite | `commands.rs` | runtime_kind immutable (pas RwLock). OK tant qu'un seul runtime. A adapter si switch runtime dynamique. | Pas urgent |
| S1-PV | SUGGESTION | test | `runtime/paths.rs` | Test unitaire pour RuntimePaths::for_kind() — verifier sous-chemins construits. | Nettoyage opportuniste |
| S2-PV | SUGGESTION | test | `runtime/paths.rs` | Test serialisation RuntimeKind → "claude" lowercase. Contrat frontend/backend. | Nettoyage opportuniste |
| S3-PV | SUGGESTION | frontend | `runtimeStore.ts` | Helper getRuntimeKind() si d'autres stores ont besoin de conditionner par runtime. | Sprint ulterieur |
| S4-PV | SUGGESTION | coherence | `deployer/mod.rs` | deploy() recalcule RuntimePaths au lieu de recevoir depuis AppState. Risque si runtime_kind != default. | Nettoyage opportuniste |
