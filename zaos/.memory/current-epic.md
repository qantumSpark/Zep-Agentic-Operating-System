# Epic active : Chat Complet

> Milestone : 1 — Chat fonctionnel avec Claude Code CLI
> Date de debut : 2026-03-29
> Statut : En cours

## Objectif

Rendre le chat pleinement fonctionnel : afficher les tool_use blocks (Read, Write, Bash...), le thinking, nourrir le feed d'actions, et permettre l'interruption reelle du CLI.

## Spec

**Comportement attendu** : Quand Claude utilise un outil (Read, Write, Bash, Glob...), l'utilisateur voit dans le chat un bloc compact montrant le nom de l'outil, un resume de l'input, et le statut. Quand Claude reflechit (thinking), un indicateur anime s'affiche. Le feed d'actions du dashboard se remplit en temps reel. L'utilisateur peut interrompre le CLI avec un bouton.

**Criteres de done** :
- Les tool_use blocks s'affichent dans le chat avec nom + input + statut
- Les tool_result mettent a jour le statut de l'action (success/error)
- Le ThinkingIndicator s'affiche pendant la reflexion
- L'ActionsFeed du dashboard montre les actions en temps reel
- Le bouton interrupt tue reellement le process CLI
- Le flag `--include-partial-messages` est actif pour un streaming optimal

## Plan technique

- Etendre le type `Message` pour porter des donnees tool_use et thinking
- Modifier `useStreaming` pour parser tous les types de content blocks
- Creer un composant `ToolUseBlock` pour le rendu visuel
- Mettre a jour `MessageBubble` pour rendre les nouveaux types
- Brancher `ThinkingIndicator` dans `ChatPanel`
- Nourrir `actionsStore` depuis les tool_use/tool_result events
- Stocker le child process handle et l'utiliser pour interrupt

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|---|---|---|---|
| 1 | Ajouter `--include-partial-messages` au spawn CLI | `src-tauri/src/session/manager.rs` | todo | 1 ligne a ajouter apres `--verbose` |
| 2 | Etendre le type Message avec champs toolUse et thinking | `src/types/events.ts` | todo | Champs optionnels sur Message |
| 3 | Parser tool_use + tool_result + thinking dans useStreaming | `src/hooks/useStreaming.ts` | todo | Creer messages pour chaque type, nourrir actionsStore |
| 4 | Creer le composant ToolUseBlock | `src/components/chat/ToolUseBlock.tsx` | todo | Nouveau fichier — nom outil, input, statut |
| 5 | Mettre a jour MessageBubble pour rendre tool_use et thinking | `src/components/chat/MessageBubble.tsx` | todo | Conditionnel sur message.toolUse |
| 6 | Brancher ThinkingIndicator dans ChatPanel | `src/components/chat/ChatPanel.tsx` | todo | Import + rendu conditionnel sur isThinking |
| 7 | Implementer interrupt reel du process CLI | `src-tauri/src/session/manager.rs`, `src-tauri/src/commands.rs` | todo | Stocker child handle, kill on interrupt |

## Dependances

- Task 2 doit etre faite avant Task 3 (types necessaires)
- Task 3 doit etre faite avant Tasks 4 et 5 (donnees dans le store)
- Task 4 doit etre faite avant Task 5 (composant importe)
- Tasks 1, 6, 7 sont independantes

## Prochaine action

Commencer par Task 1 (triviale) puis Task 2, puis enchainer dans l'ordre.
