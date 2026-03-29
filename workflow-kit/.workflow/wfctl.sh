#!/bin/bash
# =============================================================================
# wfctl.sh — Contrôleur de workflow
# =============================================================================
# Utilitaire pour gérer state.json. Utilisé par Claude (via Bash) et par
# les hooks pour connaître/modifier l'état du workflow.
#
# Usage : bash .workflow/wfctl.sh <commande> [args]
# =============================================================================

# Résoudre le chemin du projet (parent de .workflow/)
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
STATE_FILE="$PROJECT_DIR/.workflow/state.json"

# --- Timestamp ISO ---
now() {
  date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date -Iseconds 2>/dev/null || date
}

# --- Phases valides ---
VALID_PHASES="idle comprehension specification architecture implementation review test closure"

# --- Aide ---
usage() {
  cat << 'EOF'
Usage: wfctl.sh <commande> [arguments]

Commandes :
  status                   Afficher l'état actuel du workflow
  phase <nom>              Changer la phase (reset le gate)
  epic <nom>               Définir l'epic active
  task <description>       Définir la task en cours
  mode <free|pipeline>     Changer le mode
  gate validate            Marquer le gate actuel comme validé
  next                     Passer à la phase suivante (avec gate auto-reset)
  reset                    Remettre à zéro (idle, mode free)
  help                     Afficher cette aide

Phases valides :
  idle → comprehension → specification → architecture →
  implementation → review → test → closure → idle

Modes :
  free      Pas de blocage code (brainstorm, proto)
  pipeline  Blocage actif (workflow formel)

Exemples :
  wfctl.sh phase comprehension
  wfctl.sh epic "Système de brouillard de guerre"
  wfctl.sh task "Créer fog_manager.gd avec la logique de révélation"
  wfctl.sh gate validate
  wfctl.sh next
  wfctl.sh mode pipeline
EOF
}

# --- Initialiser state.json si absent ---
ensure_state() {
  if [ ! -f "$STATE_FILE" ]; then
    cat > "$STATE_FILE" << INIT
{
  "phase": "idle",
  "epic": null,
  "task": null,
  "mode": "free",
  "gate_validated": false,
  "last_updated": "$(now)"
}
INIT
  fi
}

# --- Lire une valeur ---
get_val() {
  jq -r "$1" "$STATE_FILE" 2>/dev/null
}

# --- Écrire le state ---
write_state() {
  local tmp="$STATE_FILE.tmp"
  echo "$1" | jq . > "$tmp" 2>/dev/null && mv "$tmp" "$STATE_FILE"
}

# --- Séquence des phases ---
next_phase() {
  local current="$1"
  case "$current" in
    "idle")            echo "comprehension" ;;
    "comprehension")   echo "specification" ;;
    "specification")   echo "architecture" ;;
    "architecture")    echo "implementation" ;;
    "implementation")  echo "review" ;;
    "review")          echo "test" ;;
    "test")            echo "closure" ;;
    "closure")         echo "idle" ;;
    *)                 echo "idle" ;;
  esac
}

# =============================================================================
# Commandes
# =============================================================================

ensure_state

case "${1:-help}" in

  status)
    echo "=== État du workflow ==="
    echo ""
    echo "  Phase         : $(get_val '.phase')"
    echo "  Epic          : $(get_val '.epic // "—"')"
    echo "  Task          : $(get_val '.task // "—"')"
    echo "  Mode          : $(get_val '.mode')"
    echo "  Gate validé   : $(get_val '.gate_validated')"
    echo "  Dernière MAJ  : $(get_val '.last_updated // "—"')"
    echo ""

    # Afficher le gate requis pour la phase
    PHASE=$(get_val '.phase')
    case "$PHASE" in
      "comprehension")  echo "  Gate requis : utilisateur valide la reformulation" ;;
      "specification")  echo "  Gate requis : utilisateur valide la spec" ;;
      "architecture")   echo "  Gate requis : utilisateur valide le plan et les tasks" ;;
      "implementation") echo "  Gate requis : code compile/tourne sans erreur" ;;
      "review")         echo "  Gate requis : aucun point BLOQUANT restant" ;;
      "test")           echo "  Gate requis : tous les tests passent" ;;
      "closure")        echo "  Gate requis : utilisateur confirme la clôture" ;;
      *)                echo "  Aucun gate (phase idle)" ;;
    esac
    ;;

  phase)
    if [ -z "$2" ]; then
      echo "Erreur : nom de phase requis" >&2
      echo "Phases valides : $VALID_PHASES" >&2
      exit 1
    fi

    # Vérifier que la phase est valide
    if ! echo "$VALID_PHASES" | grep -qw "$2"; then
      echo "Erreur : phase '$2' inconnue" >&2
      echo "Phases valides : $VALID_PHASES" >&2
      exit 1
    fi

    NEW_STATE=$(jq --arg p "$2" --arg d "$(now)" \
      '.phase=$p | .gate_validated=false | .last_updated=$d' "$STATE_FILE")
    write_state "$NEW_STATE"
    echo "✓ Phase → $2 (gate reset)"
    ;;

  epic)
    if [ -z "$2" ]; then
      echo "Erreur : nom d'epic requis" >&2
      exit 1
    fi
    NEW_STATE=$(jq --arg e "$2" --arg d "$(now)" \
      '.epic=$e | .last_updated=$d' "$STATE_FILE")
    write_state "$NEW_STATE"
    echo "✓ Epic → $2"
    ;;

  task)
    if [ -z "$2" ]; then
      echo "Erreur : description de task requise" >&2
      exit 1
    fi
    NEW_STATE=$(jq --arg t "$2" --arg d "$(now)" \
      '.task=$t | .last_updated=$d' "$STATE_FILE")
    write_state "$NEW_STATE"
    echo "✓ Task → $2"
    ;;

  mode)
    if [ "$2" != "free" ] && [ "$2" != "pipeline" ]; then
      echo "Erreur : mode invalide. Utiliser 'free' ou 'pipeline'" >&2
      exit 1
    fi
    NEW_STATE=$(jq --arg m "$2" --arg d "$(now)" \
      '.mode=$m | .last_updated=$d' "$STATE_FILE")
    write_state "$NEW_STATE"
    echo "✓ Mode → $2"
    if [ "$2" = "pipeline" ]; then
      echo "  Le blocage code est maintenant ACTIF (fichiers .gd/.dart sans plan → bloqués)"
    else
      echo "  Le blocage code est DÉSACTIVÉ (mode libre)"
    fi
    ;;

  gate)
    if [ "$2" = "validate" ]; then
      NEW_STATE=$(jq --arg d "$(now)" \
        '.gate_validated=true | .last_updated=$d' "$STATE_FILE")
      write_state "$NEW_STATE"
      echo "✓ Gate validé"
    else
      echo "Usage : wfctl.sh gate validate" >&2
      exit 1
    fi
    ;;

  next)
    CURRENT=$(get_val '.phase')
    GATE=$(get_val '.gate_validated')

    if [ "$GATE" != "true" ] && [ "$CURRENT" != "idle" ]; then
      echo "⚠️  Gate non validé pour la phase '$CURRENT'" >&2
      echo "   Utilise 'wfctl.sh gate validate' d'abord, ou demande à l'utilisateur." >&2
      exit 1
    fi

    NEXT=$(next_phase "$CURRENT")
    NEW_STATE=$(jq --arg p "$NEXT" --arg d "$(now)" \
      '.phase=$p | .gate_validated=false | .last_updated=$d' "$STATE_FILE")
    write_state "$NEW_STATE"
    echo "✓ $CURRENT → $NEXT (gate reset)"

    if [ "$NEXT" = "idle" ]; then
      # Reset complet en revenant à idle
      NEW_STATE=$(jq --arg d "$(now)" \
        '.epic=null | .task=null | .mode="free" | .last_updated=$d' "$STATE_FILE")
      write_state "$NEW_STATE"
      echo "  Epic et task remises à zéro (cycle terminé)"
    fi
    ;;

  reset)
    cat > "$STATE_FILE" << RESET
{
  "phase": "idle",
  "epic": null,
  "task": null,
  "mode": "free",
  "gate_validated": false,
  "last_updated": "$(now)"
}
RESET
    echo "✓ Workflow remis à zéro (idle, mode free)"
    ;;

  help|--help|-h)
    usage
    ;;

  *)
    echo "Commande inconnue : $1" >&2
    usage >&2
    exit 1
    ;;
esac
