#!/bin/bash
# =============================================================================
# Hook: UserPromptSubmit — Injection de contexte workflow
# =============================================================================
# Lit state.json et current-epic.md, puis injecte un rappel de phase,
# de rôle et de règles critiques dans le contexte de Claude à CHAQUE prompt.
# C'est le mécanisme principal qui garde les instructions "fraîches".
# =============================================================================

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"
STATE_FILE="$PROJECT_DIR/.workflow/state.json"
EPIC_FILE="$PROJECT_DIR/.memory/current-epic.md"
STATE_MD="$PROJECT_DIR/.memory/state.md"

# --- Lecture de l'état workflow ---
if [ -f "$STATE_FILE" ]; then
  PHASE=$(jq -r '.phase // "idle"' "$STATE_FILE" 2>/dev/null)
  EPIC=$(jq -r '.epic // "aucune"' "$STATE_FILE" 2>/dev/null)
  TASK=$(jq -r '.task // "aucune"' "$STATE_FILE" 2>/dev/null)
  MODE=$(jq -r '.mode // "free"' "$STATE_FILE" 2>/dev/null)
  GATE=$(jq -r '.gate_validated // false' "$STATE_FILE" 2>/dev/null)
else
  PHASE="idle"
  EPIC="aucune"
  TASK="aucune"
  MODE="free"
  GATE="false"
fi

# --- Construction du contexte ---
CONTEXT="
========== RAPPEL WORKFLOW (auto-injecté) ==========
Phase : $PHASE | Epic : $EPIC | Task : $TASK | Mode : $MODE | Gate validé : $GATE
"

# --- Rappels spécifiques par phase ---
case "$PHASE" in
  "idle")
    CONTEXT+="
Tu es l'Orchestrateur. Aucun travail en cours.
Attends la demande de l'utilisateur ou propose de reprendre un travail existant."
    ;;
  "comprehension")
    CONTEXT+="
RÔLE : Orchestrateur — Phase Compréhension
ACTION : Écoute, clarifie, reformule la demande en une phrase claire.
INTERDIT : Ne PAS coder. Ne PAS planifier en détail.
GATE REQUIS : L'utilisateur doit valider ta reformulation avant de passer en Spécification."
    ;;
  "specification")
    CONTEXT+="
RÔLE : Orchestrateur + Architecte — Phase Spécification
ACTION : Rédige une mini-spec. Déploie le Researcher si besoin d'infos.
INTERDIT : Ne PAS coder. Ne PAS découper en tasks encore.
GATE REQUIS : L'utilisateur doit valider la spec avant de passer en Architecture."
    ;;
  "architecture")
    CONTEXT+="
RÔLE : Délègue à l'Architecte (via subagent Task) — Phase Architecture
ACTION : Plan technique + découpage en tasks dans current-epic.md.
INTERDIT : Ne PAS coder. Ne PAS modifier de fichiers projet.
GATE REQUIS : L'utilisateur doit valider le plan ET les tasks."
    ;;
  "implementation")
    CONTEXT+="
RÔLE : Orchestrateur — Phase Implémentation
⚠️  TU NE CODES PAS TOI-MÊME. DÉLÈGUE au Codeur via subagent Task.
Le Codeur prend les tasks une par une dans current-epic.md.
Reviens vers l'utilisateur entre chaque task pour validation.
GATE : Le code compile/tourne sans erreur pour chaque task."
    ;;
  "review")
    CONTEXT+="
RÔLE : Délègue au Reviewer (via subagent Task) — Phase Review
ACTION : Le Reviewer relit tout le code modifié et produit des remarques.
Si points BLOQUANTS → le Codeur corrige, puis re-review.
GATE : Aucun point BLOQUANT restant."
    ;;
  "test")
    CONTEXT+="
RÔLE : Délègue au Testeur (via subagent Task) — Phase Tests
ACTION : Scénarios de test, écriture, exécution.
GATE : Tous les tests passent."
    ;;
  "closure")
    CONTEXT+="
RÔLE : Orchestrateur — Phase Clôture
ACTION : Mettre à jour state.md, current-epic.md, écrire le session log.
GATE REQUIS : L'utilisateur confirme la clôture.
Utilise: .workflow/wfctl.sh reset  quand la clôture est confirmée."
    ;;
esac

# --- Rappel epic en cours (si existe) ---
if [ -f "$EPIC_FILE" ] && [ "$PHASE" != "idle" ]; then
  EPIC_SUMMARY=$(head -20 "$EPIC_FILE" 2>/dev/null)
  CONTEXT+="

--- Tasks de l'epic en cours ---
$EPIC_SUMMARY"
fi

# --- Règles non-négociables (toujours injectées) ---
CONTEXT+="

========== RÈGLES NON-NÉGOCIABLES ==========
1. JAMAIS coder sans plan validé dans current-epic.md
2. JAMAIS sauter un gate sans l'accord EXPLICITE de l'utilisateur
3. DÉLÉGUER via subagent Task — ne PAS tout faire dans le contexte principal
4. JAMAIS inventer une API, classe, méthode ou signal — vérifier d'abord
5. Signaler immédiatement tout écart par rapport au plan
6. Pour changer de phase : utiliser .workflow/wfctl.sh phase <nom>
====================================================
"

# --- Sortie JSON pour UserPromptSubmit ---
jq -n --arg ctx "$CONTEXT" '{"additionalContext": $ctx}'
