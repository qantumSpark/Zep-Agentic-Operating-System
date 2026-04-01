# Test Run #2 — Findings (2026-04-01)

> Projet test : "Task Timer" — app web Node.js + Express + vanilla HTML/CSS/JS
> Dossier : C:\Users\apeze\Documents\Test Project
> Mode : Pipeline + Accept Edits

## Finding 1/6 — Gate desync + multi-clic (CRITIQUE)

**Symptome** : En idle, Claude fait la comprehension et demande le gate. Quand l'utilisateur valide via le bouton, la phase avance mais le pipeline visuel est decale d'un cran. L'utilisateur doit cliquer "dans le vide" pour rattraper. Claude pense etre en Review alors que l'implementation est encore en cours.

**Cause probable** : Le bouton Validate Gate est toujours actif — pas de protection contre multi-clic, pas de feedback d'etat ("gate valide, en attente..."). Possible double-advance : le bouton fait next_phase() ET Claude interprete la validation comme "passe a la suite".

**Solution proposee par l'utilisateur** : Gate as quality checkpoint :
- Bouton grise par defaut pendant que Claude travaille
- Claude finit une phase → flip un flag `gate_ready: true` 
- Le bouton s'active (devient vert cliquable)
- Click → gate valide → bouton redevient grise
- Ne debloquer que quand les fichiers memoire (current-epic.md, state.md) sont a jour
- Securite supplementaire : verifier coherence memoire avant d'activer le bouton

## Finding 2 — "0/0 taches" en comprehension/spec (MOYENNE)

**Symptome** : L'epic est creee en phase comprehension mais les tasks ne sont ecrites dans current-epic.md que par l'architecte (phase architecture). Pendant 2-3 phases, le dashboard affiche "Backend API — EN COURS (0/0 taches)" avec une barre verte a 0%.

**Cause** : Le compteur compte les tasks dans le tableau de current-epic.md. Avant l'architecture, le tableau est vide.

**Fix** : Soit masquer le compteur quand 0 tasks, soit afficher "Plan en attente..." au lieu de "0/0 taches".

## Finding 3 — Memory update tardif (MOYENNE)

**Symptome** : Les tasks n'apparaissent dans le dashboard qu'apres avoir ete completees, pas quand elles sont creees par l'architecte.

**Cause probable** : Le watcher .memory/ ne detecte pas les changements assez vite, ou le parser rate les nouvelles tasks. Potentiellement un probleme de debounce trop long ou de parsing incremental.

## Finding 4/5/9 — Hooks bloquent review/test (CRITIQUE)

**Symptome** : En phase review, le Reviewer identifie 2 bugs et veut les fixer. Il delegue au Coder → Edit bloque par block-code. Il essaie Bash → bloque par enforce-gate. Il essaie d'editer lui-meme → block-code bloque encore. Claude est completement paralyse.

Meme probleme en phase test : le Tester a besoin de Bash pour `node server.js` → bloque par enforce-gate.

**Cause** : Les hooks block-code et enforce-gate ne distinguent pas les phases. Ils bloquent des que "toutes les tasks sont DONE + gate pas valide", peu importe qu'on soit en implementation (legitime) ou en review/test (pas legitime).

**Fix** : block-code et enforce-gate doivent autoriser les operations en phase review et test, pas seulement en implementation. Condition de blocage : seulement en phase implementation quand toutes tasks DONE.

## Finding 5 — enforce-gate faux positif sur 0 tasks (CRITIQUE)

**Symptome** : L'architecte en phase architecture essaie de faire `ls -la` pour explorer le projet → enforce-gate le bloque. L'epic "Frontend UI" a 0 tasks, donc `has_active_tasks()` retourne false → interprete comme "toutes tasks terminees" → bloque.

**Cause** : `has_active_tasks()` retourne false quand il n'y a aucune ligne de task dans le tableau. Pas de tasks = pas de gate a enforcer.

**Fix** : `has_active_tasks()` doit retourner true (ou bypass) quand il n'y a aucune task dans le tableau. 0 tasks ≠ toutes tasks done.

## Finding 7/10 — Compteur 0/0 persistant (MOYENNE)

**Symptome** : "Frontend UI — EN COURS (0/0 taches)" reste affiche meme quand des tasks sont visibles en dessous dans la liste. Le compteur ne se met pas a jour.

**Cause probable** : Le parser du compteur (MemorySection.tsx) ne re-parse pas quand current-epic.md est modifie, ou le watcher .memory/ n'emet pas l'event assez frequemment.

## Finding 8 — Delegation RUNNING fantome (BASSE)

**Symptome** : "Coder: Task 2 - style.css" affiche RUNNING depuis 7 min alors que Task 3 (app.js) est deja COMPLETED a 6 min. Le statut n'est jamais flippe a COMPLETED.

**Cause probable** : Le tool_result de retour de l'Agent n'est pas capte correctement pour cette delegation. L'ID de la delegation ne matche pas entre le tool_use et le tool_result dans useStreaming.ts.

## Priorites

1. CRITIQUE : Hooks trop restrictifs (Finding 4/5/9) — rend le pipeline inutilisable apres implementation
2. CRITIQUE : Gate UX (Finding 1/6) — desync + pas de controle
3. CRITIQUE : has_active_tasks faux positif sur 0 tasks (Finding 5)
4. MOYENNE : Compteur/memory updates (Finding 2/3/7/10)
5. BASSE : Delegation fantome (Finding 8)
