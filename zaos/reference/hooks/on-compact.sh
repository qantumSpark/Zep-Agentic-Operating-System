#!/bin/bash
# =============================================================================
# Hook: SessionStart (matcher: compact) — Ré-injection après compaction
# =============================================================================
# Quand le contexte de Claude est compacté (trop long), les instructions
# initiales sont résumées et perdent en précision. Ce hook ré-injecte
# les informations critiques pour que Claude ne "décroche" pas.
#
# Sortie : texte brut sur stdout → injecté dans le contexte de Claude.
# =============================================================================

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"
STATE_FILE="$PROJECT_DIR/.workflow/state.json"
EPIC_FILE="$PROJECT_DIR/.memory/current-epic.md"
STATE_MD="$PROJECT_DIR/.memory/state.md"
CONVENTIONS="$PROJECT_DIR/.memory/conventions.md"

echo "============================================================"
echo "  CONTEXTE CRITIQUE — Ré-injecté après compaction"
echo "============================================================"
echo ""

# --- État du workflow ---
if [ -f "$STATE_FILE" ]; then
  PHASE=$(jq -r '.phase // "idle"' "$STATE_FILE" 2>/dev/null)
  EPIC=$(jq -r '.epic // "aucune"' "$STATE_FILE" 2>/dev/null)
  TASK=$(jq -r '.task // "aucune"' "$STATE_FILE" 2>/dev/null)
  MODE=$(jq -r '.mode // "free"' "$STATE_FILE" 2>/dev/null)
  echo "Phase : $PHASE | Epic : $EPIC | Task : $TASK | Mode : $MODE"
else
  echo "Phase : idle (pas de state.json)"
fi

echo ""

# --- Plan de l'epic en cours ---
if [ -f "$EPIC_FILE" ]; then
  echo "--- Plan de l'epic active ---"
  cat "$EPIC_FILE"
  echo ""
fi

# --- État projet (résumé) ---
if [ -f "$STATE_MD" ]; then
  echo "--- État du projet (state.md) ---"
  cat "$STATE_MD"
  echo ""
fi

# --- Conventions critiques (premières lignes) ---
if [ -f "$CONVENTIONS" ]; then
  echo "--- Conventions (extrait) ---"
  head -30 "$CONVENTIONS"
  echo ""
fi

# --- Règles non-négociables ---
cat << 'EOF'
============================================================
  RÈGLES NON-NÉGOCIABLES (rappel post-compaction)
============================================================

1. Tu es l'ORCHESTRATEUR. Tu coordonnes. Tu ne codes PAS toi-même.
2. DÉLÈGUE aux subagents via Task :
   - Architecte pour les plans
   - Codeur pour l'implémentation
   - Reviewer pour la validation
   - Testeur pour les tests
3. JAMAIS coder sans plan validé dans .memory/current-epic.md
4. JAMAIS sauter un gate sans l'accord EXPLICITE de l'utilisateur
5. JAMAIS inventer une API, classe, méthode ou signal — vérifier d'abord
6. Pour changer de phase : bash .workflow/wfctl.sh phase <nom>
7. Pour voir l'état : bash .workflow/wfctl.sh status

============================================================
EOF
