# Conventions ZAOS

> Standards de code pour ce projet. Lu avant chaque phase d'implementation.
> Mis a jour : 2026-03-29

## TypeScript / React

### Naming
- Composants : PascalCase (`ChatPanel.tsx`, `MessageBubble.tsx`)
- Hooks : camelCase prefixe `use` (`useStreaming.ts`, `useTauriEvents.ts`)
- Stores Zustand : camelCase suffixe `Store` (`chatStore.ts`, `sessionStore.ts`)
- Types/interfaces : PascalCase (`CliEvent`, `WorkflowState`)
- Fichiers de types : camelCase (`events.ts`, `workflow.ts`)

### Patterns adoptes
- **Zustand** pour tout le state management (pas de Context API, pas de Redux)
- **Functional components** uniquement (pas de class components)
- **Named exports** partout (pas de default exports)
- **Tailwind CSS v4** pour le styling (pas de CSS modules, pas de styled-components)
- **react-markdown + remark-gfm** pour le rendu Markdown

### Structure d'un composant
```tsx
import React from "react";
// imports externes
// imports internes (stores, hooks, types)

// types locaux si necessaire

export function ComponentName() {
  // hooks
  // state local
  // handlers
  // return JSX
}
```

### Patterns interdits
- Pas de `any` — typer explicitement
- Pas de `useEffect` pour du state management (utiliser Zustand)
- Pas de CSS inline sauf cas ponctuel justifie

## Rust / Tauri

### Naming
- Modules : snake_case (`session/manager.rs`, `events/parser.rs`)
- Structs/Enums : PascalCase (`AppState`, `CliEvent`, `WorkflowMode`)
- Functions : snake_case (`send_prompt`, `parse_event`)
- Constantes : SCREAMING_SNAKE_CASE

### Patterns adoptes
- **tokio** pour l'async runtime
- **serde** pour serialisation/deserialisation JSON
- **tracing** pour le logging (pas println!)
- **Arc<Mutex<>>** pour le state partage entre tasks
- **broadcast channel** pour la distribution d'events
- **Custom error types** par module (SessionError, WorkflowError, etc.)

### Patterns interdits
- Pas de `unwrap()` en production — utiliser `?` ou gerer l'erreur
- Pas de `println!` — utiliser `tracing::info/warn/error`

## Regles de commit
- Format : description courte en anglais, imperatif
- Ex: `Add tool_use block rendering in chat`, `Fix CLI spawn missing --include-partial-messages`

## Regles de documentation
- Commenter le "pourquoi", pas le "quoi"
- Doc comments (`///`) sur les fonctions publiques Rust
- Pas de JSDoc sur les composants React (le nom suffit)
