# Test Run Baseline — zaos-baseline tag

**Date:** 2026-04-01
**Build:** release (zaos_0.1.0_x64)
**Commit:** 1773407
**Tag:** zaos-baseline
**Branche:** baseline/stabilize-current
**Testeur:** User (test manuel)
**Projet test:** Launchpad Solo — board Kanban vanilla JS (3 milestones, 5 epics)
**Duree test:** ~1h (22:04 → 23:04)

---

## Synthese

| Categorie | Compte |
|-----------|--------|
| BLOQUANT | 2 |
| MAJEUR | 7 |
| MINEUR | 3 |
| Total findings | 12 |

---

## Findings detailles

### #1 — BLOQUANT — Selection de dossier projet echoue

**Contexte:** Au lancement de l'app, l'utilisateur clique sur le ProjectPicker dans la StatusBar, le selecteur de dossier natif Windows s'ouvre, il selectionne un dossier, mais rien ne se passe. Aucun feedback UI. L'erreur n'est visible que dans la console DevTools (F12).

**Message d'erreur:** `Failed to switch project: Projects must be under user home directory`

**Cause racine identifiee:** Sur Windows, `std::fs::canonicalize()` retourne des chemins avec le prefixe UNC etendu `\\?\C:\Users\...` alors que `dirs::home_dir()` retourne `C:\Users\...`. Le check `canonical.starts_with(&home)` echoue systematiquement car les prefixes ne matchent pas.

**Fichier:** `zaos/src-tauri/src/commands.rs` lignes 759-766

**Fix:** Canonicaliser les DEUX chemins pour qu'ils aient le meme format : `let canonical_home = home.canonicalize().unwrap_or(home);`

**Probleme secondaire:** L'erreur est avalee silencieusement cote UI — le `catch` dans `ProjectPicker.tsx:23-24` log en console mais n'affiche rien a l'utilisateur.

**Statut:** Plan de fix pret.

---

### #2 — BLOQUANT — Bouton de validation de gate absent

**Contexte:** En mode Pipeline, chaque phase a un "gate" que l'utilisateur doit valider avant de passer a la suivante. Ce bouton n'apparait nulle part dans le dashboard. L'utilisateur n'a aucun moyen de valider un gate depuis l'UI.

**Comment decouvert:** L'utilisateur a demande "met a jour la phase du pipeline, j'ai un bug, le bouton pour avancer n'apparait pas" pendant la phase architecture du premier epic.

**Hypothese:** Le composant `PipelineSection` dans le dashboard n'implemente pas de bouton "Valider le gate" ni d'appel a la commande Tauri `validate_gate`. Le workflow engine backend est pret (la methode `validate_gate()` existe et fonctionne) mais le frontend n'expose pas l'action.

**Impact cascade:** Sans ce bouton, les findings #3, #4 et partiellement #10 en decoulent — le pipeline ne peut pas avancer normalement, l'orchestrateur a du contourner en ecrivant manuellement dans `state.json`.

**Fichiers a investiguer:** `zaos/src/components/dashboard/PipelineSection.tsx`, `zaos/src/stores/workflowStore.ts`

---

### #3 — MAJEUR — Affichage des phases pipeline reste sur "Idle"

**Contexte:** Le dashboard affiche la section Pipeline avec les 8 phases (Idle → Closure) mais la phase active reste sur "Idle" tout au long de la session, meme quand `state.json` est modifie manuellement ou par l'orchestrateur.

**Comment decouvert:** Screenshots #1 et #3 montrent "Idle" alors que le chat est en phase implementation/review. L'orchestrateur ecrit `"phase": "implementation"` dans `state.json` mais le dashboard ne reagit pas.

**Hypothese:** Deux causes possibles :
1. Le `FileWatcherService` backend ne surveille pas `state.json` ou ne detecte pas les changements manuels (Write via Claude Code ne passe pas par le workflow engine, donc pas d'event broadcast)
2. Le frontend `workflowStore` n'est alimente que par l'event `"workflow-change"` emis par le backend, mais cet event n'est emis que quand le `WorkflowEngine` persiste via `persist_and_notify()` — pas quand le fichier est edite directement

**Fichiers a investiguer:** `zaos/src-tauri/src/watchers/`, `zaos/src-tauri/src/workflow/engine.rs`, `zaos/src/stores/workflowStore.ts`

---

### #4 — MAJEUR — state.json pas mis a jour normalement

**Contexte:** Le fichier `.workflow/state.json` reste sur `"phase": "idle"` meme apres plusieurs transitions de phase dans le chat. L'orchestrateur a du le mettre a jour manuellement via Write a chaque validation de gate.

**Comment decouvert:** L'orchestrateur a lu `state.json` et constate `"phase": "idle"` alors qu'on etait en phase "architecture validee".

**Hypothese:** Directement lie a #2 — le bouton gate n'existant pas, la commande Tauri `validate_gate` + `next_phase` n'est jamais appelee, donc le `WorkflowEngine` ne met jamais a jour le fichier via son chemin normal. Le contournement (Write direct) ne passe pas par le engine, donc pas de notification.

---

### #5 — MAJEUR — Actions apparaissent seulement a la fin

**Contexte:** Pendant qu'un sub-agent (Coder, Reviewer) travaille, le panel "Actions (live)" du dashboard reste vide. Toutes les actions (Read, Write, Edit) apparaissent d'un coup quand l'agent termine.

**Comment decouvert:** Screenshots #1 et #2 montrent le panel Actions vide pendant le travail, puis plein une fois termine. L'utilisateur note "toutes les taches sont apparue une fois fini, rien avant".

**Hypothese:** Les events de type `tool_call_started` et `tool_call_finished` emis par les sub-agents sont peut-etre bufferises ou ne sont pas forwarded en temps reel. Possible que le stream-json des sub-agents ne remonte pas les tool events individuellement — seulement le resultat final. A verifier dans `events/parser.rs` et le protocole stream-json pour les agents delegues.

**Fichiers a investiguer:** `zaos/src-tauri/src/events/parser.rs`, `zaos/src-tauri/src/events/mapper.rs`, `zaos/src/hooks/useStreaming.ts`

---

### #6 — MINEUR — Mauvais nom d'agent dans HISTORIQUE

**Contexte:** Dans le panel Agents > HISTORIQUE du dashboard, l'agent Reviewer affiche "Coder RUNNING 17s" alors que c'est bien le Reviewer qui a ete lance (l'appel Agent dans le chat montre `Agent: Review du code MVP` et lit `reviewer.md`).

**Comment decouvert:** Screenshot #2 — visible dans l'encart HISTORIQUE a droite : "Coder RUNNING 17s / Review du code MVP".

**Hypothese:** Le mapping entre le `subagent_type` ou le nom de l'agent dans l'event et le label affiche dans l'UI est incorrect. Le frontend extrait probablement le type d'agent depuis un champ qui ne correspond pas au role reel. Possible confusion entre le dernier agent "Coder" utilise et le Reviewer courant si l'UI ne reset pas correctement.

**Fichiers a investiguer:** `zaos/src/stores/agentsStore.ts`, composant Agents dans le dashboard

---

### #7 — MAJEUR — MCP demande approbation manuelle systematique

**Contexte:** Pendant la phase de test, le Testeur utilise le MCP Chrome DevTools pour naviguer, cliquer, prendre des screenshots, evaluer du JS. Chaque appel MCP declenche une popup "Permission requested" que l'utilisateur doit approuver manuellement. Sur une session de test complete, ca represente 30+ approbations.

**Comment decouvert:** L'utilisateur note "c'est tres fastidieux" apres avoir du approuver chaque action du testeur MCP.

**Hypothese:** Le systeme d'auto-approval dans `commands.rs` (logique `should_auto_approve`) ne couvre que les outils Claude Code standard (Write, Edit, Read, Bash, etc.) mais pas les appels MCP. En mode "Accept Edits", les MCP installes par l'utilisateur devraient etre consideres comme trusted et auto-approved. Seul le mode "strict" devrait demander confirmation.

**Fichiers a investiguer:** `zaos/src-tauri/src/commands.rs` (logique auto-approval), `zaos/src-tauri/src/bin/zaos_hooks.rs`

---

### #8 — MAJEUR — Tasks du plan s'affichent seulement quand DONE

**Contexte:** Dans le dashboard, la section "ACTIVE EPIC" affiche correctement le titre de l'epic et l'objectif. Mais les tasks individuelles du plan n'apparaissent pas tant qu'elles ne sont pas marquees DONE dans `current-epic.md`. Pendant le travail, le dashboard affiche "Plan en attente..." alors que le plan avec les tasks A FAIRE existe dans le fichier.

**Comment decouvert:** Screenshot #3 montre "Plan en attente..." alors que `current-epic.md` contient 3 tasks avec statut "A FAIRE". Apres completion, screenshot #4 montre les 3 tasks toutes a DONE.

**Hypothese:** Le parser Rust dans `memory/reader.rs` qui parse le tableau de tasks de `current-epic.md` filtre probablement les tasks par statut et ne retourne que celles en DONE. Ou bien le frontend filtre cote affichage. Le texte "Plan en attente..." suggere que le code verifie si des tasks existent et considere qu'il n'y en a aucune tant qu'aucune n'est DONE.

**Fichiers a investiguer:** `zaos/src-tauri/src/memory/reader.rs`, composant epic dans le dashboard

---

### #9 — MAJEUR — Agent general-purpose lance au lieu du Coder

**Contexte:** Quand l'orchestrateur delegue l'implementation au "Codeur", il utilise parfois `"subagent_type": "general-purpose"` dans le JSON de l'appel Agent, alors que le prompt dit explicitement "Tu es le Codeur du projet". Le code est execute correctement (le general-purpose agent fait le travail) mais ce n'est pas l'agent deploye `.claude/agents/coder.md` qui est utilise.

**Comment decouvert:** Screenshots #5 et #6 montrent le JSON de l'appel Agent avec `"subagent_type": "general-purpose"`. L'HISTORIQUE affiche "General-purpose RUNNING". Observe pour les epics 2 et 3 (Filtres, Polish) mais pas pour l'epic 1 (Board & CRUD) ou le Coder a ete correctement lance.

**Pattern observe:**
- Epic 1 (Board & CRUD) : Coder correct
- Epic 2 (Drag & Drop) : general-purpose pour implementation
- Epic 2 (Filtres) : general-purpose pour implementation
- Epic 3 (Polish) : general-purpose pour implementation
- Reviews : Reviewer toujours correct

**Hypothese:** Le CLAUDE.md dit "TOUJOURS deleguer" et liste les agents, mais ne donne pas le mapping explicite entre le role et le `subagent_type` a utiliser dans l'appel Agent. Le LLM ne sait pas que pour utiliser `coder.md`, il faut passer un subagent_type specifique. Il derive vers `general-purpose` par defaut. Le Reviewer fonctionne car il existe comme subagent_type natif de Claude Code.

**Fix:** Ajouter dans CLAUDE.md un mapping explicite : role → subagent_type (ou mieux, le role → le fichier agent a inclure dans le prompt).

---

### #10 — MAJEUR — Orchestrateur code les corrections post-review

**Contexte:** Apres chaque review, quand des corrections sont necessaires (bloquants ou suggestions), l'orchestrateur applique les corrections lui-meme via Edit/Write au lieu de deleguer au Coder.

**Comment decouvert:** Observe systematiquement dans la conversation — apres chaque review (Drag & Drop 22:39, Filtres 22:49, Polish 22:58), l'orchestrateur fait directement les Edit sur style.css et app.js.

**Pattern:** 3/3 reviews → corrections faites par l'orchestrateur. 0 delegation au Coder pour les corrections.

**Hypothese:** Plusieurs facteurs :
1. Les corrections post-review sont percues comme "petites" par le LLM, qui juge inutile de deleguer
2. Le CLAUDE.md ne mentionne pas explicitement que les corrections post-review doivent aussi etre deleguees
3. Sans gates fonctionnels (#2), le hook `block-code` ne bloque probablement pas l'orchestrateur car il n'y a pas de mecanisme pour detecter que c'est l'orchestrateur qui ecrit et pas un sub-agent

---

### #11 — MINEUR — Delegation architecture inconsistante

**Contexte:** L'agent Architect n'a ete utilise que pour le tout premier epic (Board & CRUD). Pour les 4 epics suivants, l'orchestrateur a fait l'architecture lui-meme avec la justification "Epic cible, pas besoin de deleguer a l'architecte".

**Comment decouvert:** Relecture de la conversation complete — seul l'appel `Agent: Architecture Board Kanban MVP` a 22:09:47 utilise l'architecte.

**Hypothese:** Le LLM optimise en evitant un aller-retour agent quand l'epic est simple. C'est pragmatique mais viole la regle "TOUJOURS deleguer". Le CLAUDE.md devrait soit renforcer la regle, soit expliciter les exceptions acceptables.

---

### #12 — MINEUR — Erreur Write sans Read prealable

**Contexte:** A 22:08:15, l'orchestrateur tente un Write sur `current-epic.md` sans l'avoir lu d'abord. Claude Code refuse avec `"File has not been read yet. Read it first before writing to it."` L'orchestrateur fait ensuite un Read puis re-Write avec succes.

**Comment decouvert:** Visible dans la conversation a 22:08:15.

**Hypothese:** C'est une contrainte de securite de Claude Code (pas un bug ZAOS). Mais les instructions d'agent pourraient rappeler ce pattern pour eviter le roundtrip inutile.

---

## Ce qui fonctionne bien

- L'app se lance correctement (exe standalone)
- Le chat fonctionne, les messages s'affichent en streaming
- La section Memory (milestones, epic active, blocages) parse et affiche correctement
- La liste des agents deployes s'affiche
- Le pipeline complet (7 phases) est bien suivi par l'orchestrateur
- Les phases de test via MCP Chrome DevTools fonctionnent (fond functionnel OK)
- La memoire projet (state.md, current-epic.md) est mise a jour correctement
- Le cadrage initial (questions de stack, scope, contraintes) est bien fait
- Le brainstorming milestones fonctionne
- L'orchestrateur demande validation a chaque gate (meme si le bouton UI manque)

## Classification des findings par zone

| Zone | Findings |
|------|----------|
| **Backend Rust (commands, engine, watchers)** | #1, #3, #4, #7 |
| **Frontend Dashboard (composants, stores)** | #2, #3, #5, #6, #8 |
| **Instructions LLM (CLAUDE.md, agents)** | #9, #10, #11, #12 |
| **UX/Feedback** | #1 (erreur silencieuse), #7 |

## Priorite de correction suggeree

1. **#1** — Fix Windows path (1 ligne, debloque la selection projet)
2. **#2** — Bouton gate dans le dashboard (debloque tout le pipeline UI)
3. **#3/#4** — Synchronisation state.json ↔ dashboard (decoule de #2)
4. **#9** — Mapping agent role → subagent_type dans CLAUDE.md
5. **#10** — Renforcer regle de delegation post-review
6. **#8** — Parser tasks avec tous les statuts
7. **#5** — Streaming actions en temps reel
8. **#7** — Auto-approve MCP en mode Accept Edits
9. **#6, #11, #12** — Mineurs, a traiter en opportunite
