#!/bin/bash
# =============================================================================
# Hook: SessionStart (matcher: startup) — Message d'accueil
# =============================================================================
# Vérifie que le workflow est correctement installé et affiche
# un diagnostic + l'état actuel au lancement de Claude Code.
# =============================================================================

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"
STATE_FILE="$PROJECT_DIR/.workflow/state.json"
EPIC_FILE="$PROJECT_DIR/.memory/current-epic.md"
WFCTL="$PROJECT_DIR/.workflow/wfctl.sh"

echo ""
echo "╔══════════════════════════════════════════════════╗"
echo "║        🚀 WORKFLOW ACTIF — {{NOM_DU_PROJET}}    ║"
echo "╚══════════════════════════════════════════════════╝"
echo ""

# --- Vérification des composants ---
ERRORS=0

check() {
  if [ -f "$1" ]; then
    echo "  ✅ $2"
  else
    echo "  ❌ $2 — MANQUANT"
    ERRORS=$((ERRORS + 1))
  fi
}

echo "── Composants ──"
check "$PROJECT_DIR/CLAUDE.md" "CLAUDE.md"
check "$STATE_FILE" "state.json"
check "$WFCTL" "wfctl.sh"
check "$PROJECT_DIR/.claude/hooks/inject-context.sh" "Hook inject-context"
check "$PROJECT_DIR/.claude/hooks/block-code-without-plan.sh" "Hook block-code"
check "$PROJECT_DIR/.claude/hooks/on-compact.sh" "Hook on-compact"
check "$PROJECT_DIR/.memory/INDEX.md" "Mémoire INDEX"
check "$PROJECT_DIR/.memory/state.md" "Mémoire state"
check "$PROJECT_DIR/.workflow/process.md" "Process workflow"

# --- Vérifier jq ---
if command -v jq &>/dev/null; then
  echo "  ✅ jq $(jq --version 2>&1)"
else
  echo "  ❌ jq — NON INSTALLÉ (les hooks ne fonctionneront pas)"
  ERRORS=$((ERRORS + 1))
fi

echo ""

# --- Diagnostic ---
if [ "$ERRORS" -gt 0 ]; then
  echo "⚠️  $ERRORS problème(s) détecté(s) — certains hooks pourraient ne pas fonctionner"
  echo ""
else
  echo "✅ Tous les composants OK — workflow opérationnel"
  echo ""
fi

# --- État du workflow ---
if [ -f "$STATE_FILE" ]; then
  PHASE=$(jq -r '.phase // "idle"' "$STATE_FILE" 2>/dev/null)
  EPIC=$(jq -r '.epic // "—"' "$STATE_FILE" 2>/dev/null)
  TASK=$(jq -r '.task // "—"' "$STATE_FILE" 2>/dev/null)
  MODE=$(jq -r '.mode // "free"' "$STATE_FILE" 2>/dev/null)

  echo "── État actuel ──"
  echo "  Phase : $PHASE"
  echo "  Epic  : $EPIC"
  echo "  Task  : $TASK"
  echo "  Mode  : $MODE"

  if [ "$MODE" = "pipeline" ]; then
    echo ""
    echo "  🔒 Mode PIPELINE — blocage code actif"
  else
    echo ""
    echo "  🔓 Mode FREE — brainstorm libre, pas de blocage"
  fi
fi

echo ""
echo "──────────────────────────────────────────────────"
echo "  Commandes : bash .workflow/wfctl.sh help"
echo "──────────────────────────────────────────────────"
echo ""
