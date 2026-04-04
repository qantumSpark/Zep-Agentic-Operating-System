# Backlog — Findings non bloquants Bloc 2

> Source : revue post-bloc 2 (2026-04-04)
> Statut : tous non bloquants, a traiter quand pertinent

| ID | Severite | Categorie | Fichier(s) | Description | Sprint suggere |
|---|---|---|---|---|---|
| F3 | MODERE | dette | `src/types/workflow.ts` | Interface `WorkflowState` orpheline — ne correspond pas au backend DTO. Seul `BackendWorkflowPayload` est utilise. Code mort trompeur. | Nettoyage opportuniste |
| F4 | MODERE | vision | `DashboardPanel.tsx` | Product contract et workflow projection deconnectes visuellement. Pas de lien acceptance checks / gate, ni session-insights / phase produit. | Sprint 4A (Dashboard Decision-First) |
| F5 | MINEUR | vision | `models.rs`, `productContract.ts` | session-insights est singulier — un seul snapshot, pas d'historique. Limite l'utilite de la phase Learn. | Sprint 4B (Personas + session insights) |
| F6 | MODERE | dette | `src-tauri/src/memory/reader.rs` | Parsers produit silencieux sur contenu maldeforme. Heading different → struct vide sans avertissement. Impossible de distinguer "fichier vide" de "parsing rate". | Nettoyage opportuniste |
| F7 | MINEUR | dette | `src-tauri/src/workflow/state.rs` | DTO construit un WorkflowState temporaire synthetique pour deriver next_product_phase. Fonctionnel mais indirection contournee. | Pas urgent |
| F9 | MINEUR | dette | `src/stores/workflowStore.ts` | Fallback silencieux si phase inconnue — pas de log quand le backend envoie une valeur non reconnue par le frontend. | Pas urgent |
