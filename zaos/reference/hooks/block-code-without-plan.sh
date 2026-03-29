#!/bin/bash
# =============================================================================
# Hook: PreToolUse (Write|Edit) — Blocage dur : pas de code sans plan
# =============================================================================
# Vérifie que current-epic.md contient un plan de tasks AVANT d'autoriser
# l'écriture de fichiers de code (.gd, .dart, .tscn, .tres, .gdshader).
#
# Exit 2 = blocage mécanique. Claude ne peut PAS passer outre.
# Les fichiers non-code (.md, .json, .cfg, etc.) ne sont PAS bloqués.
# =============================================================================

INPUT=$(cat)
PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"
EPIC_FILE="$PROJECT_DIR/.memory/current-epic.md"
STATE_FILE="$PROJECT_DIR/.workflow/state.json"

# --- Extraire le chemin du fichier ciblé ---
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // .tool_input.file // ""' 2>/dev/null)

# Si pas de chemin, laisser passer (cas edge)
if [ -z "$FILE_PATH" ]; then
  exit 0
fi

# --- Vérifier si c'est un fichier de code ---
# Extensions Godot : .gd, .tscn, .tres, .gdshader, .gdextension
# Extensions Flutter : .dart
# On ne bloque PAS : .md, .json, .cfg, .txt, .mdc, .sh, etc.
case "$FILE_PATH" in
  *.gd|*.dart|*.tscn|*.tres|*.gdshader|*.gdextension)
    # C'est un fichier de code — on vérifie le plan
    ;;
  *)
    # Pas un fichier de code — on laisse passer
    exit 0
    ;;
esac

# --- Vérifier le mode workflow ---
# En mode "free", pas de blocage (brainstorm, prototypage rapide)
if [ -f "$STATE_FILE" ]; then
  MODE=$(jq -r '.mode // "free"' "$STATE_FILE" 2>/dev/null)
  if [ "$MODE" = "free" ]; then
    exit 0
  fi
fi

# --- Vérifier que current-epic.md existe ---
if [ ! -f "$EPIC_FILE" ]; then
  cat >&2 << 'EOF'
⛔ BLOQUÉ — Pas de plan d'epic

Aucun fichier .memory/current-epic.md trouvé.
Tu ne peux pas écrire de code sans un plan de tasks validé.

Actions possibles :
  1. Déployer l'Architecte pour créer le plan
  2. Passer en mode libre : .workflow/wfctl.sh mode free
  3. Demander à l'utilisateur de valider un plan
EOF
  exit 2
fi

# --- Vérifier que le plan contient des tasks ---
# Cherche des marqueurs de tasks : todo, in_progress, done, ou checkboxes markdown
if ! grep -qiE '(todo|in_progress|done|\- \[[ x]\])' "$EPIC_FILE" 2>/dev/null; then
  cat >&2 << 'EOF'
⛔ BLOQUÉ — Plan sans tasks

.memory/current-epic.md existe mais ne contient pas de tasks.
Le plan doit contenir des tasks avec statut (todo/in_progress/done)
avant que le code puisse être écrit.

Actions possibles :
  1. Compléter le plan avec des tasks précises
  2. Faire valider le plan par l'utilisateur
EOF
  exit 2
fi

# --- Vérifier qu'il y a au moins une task in_progress ou todo ---
if ! grep -qiE '(todo|in_progress)' "$EPIC_FILE" 2>/dev/null; then
  cat >&2 << 'EOF'
⛔ BLOQUÉ — Toutes les tasks sont terminées

Toutes les tasks dans current-epic.md sont marquées done.
Passe en phase Review ou Clôture, ou ajoute de nouvelles tasks.
EOF
  exit 2
fi

# --- Tout est OK, le code peut être écrit ---
exit 0
