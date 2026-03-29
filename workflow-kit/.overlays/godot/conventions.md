# Conventions Godot — GDScript

> Ce fichier est lu par le Codeur avant chaque phase d'implémentation sur un projet Godot.
> À compléter/adapter selon le projet.

## Versions
- Godot : {{4.x}}
- GDScript : natif

## Structure du projet

```
project/
├── scenes/           ← Scènes (.tscn)
│   ├── ui/           ← Scènes d'interface
│   ├── entities/     ← Entités du jeu (player, enemies, items)
│   └── levels/       ← Niveaux / maps
├── scripts/          ← Scripts GDScript (.gd)
│   ├── autoloads/    ← Singletons globaux
│   ├── components/   ← Scripts réutilisables (composants)
│   ├── resources/    ← Custom Resources (.tres)
│   └── utils/        ← Utilitaires
├── assets/
│   ├── sprites/
│   ├── audio/
│   └── fonts/
└── addons/           ← Plugins Godot
```

## Naming

### Fichiers
- Scripts : `snake_case.gd` (ex: `player_controller.gd`)
- Scènes : `snake_case.tscn` (ex: `main_menu.tscn`)
- Resources : `snake_case.tres` (ex: `sword_item.tres`)

### Code GDScript
- Classes : `PascalCase` (ex: `class_name PlayerController`)
- Fonctions : `snake_case` (ex: `func get_health():`)
- Variables : `snake_case` (ex: `var max_health: int = 100`)
- Constantes : `UPPER_SNAKE_CASE` (ex: `const MAX_SPEED: float = 200.0`)
- Signaux : `snake_case` au passé (ex: `signal health_changed`)
- Enums : `PascalCase` pour le type, `UPPER_SNAKE_CASE` pour les valeurs

### Nodes dans les scènes
- `PascalCase` (ex: `PlayerSprite`, `HealthBar`, `HitboxArea2D`)
- Préfixer par le type si ambigu (ex: `UIHealthBar` vs `HealthComponent`)

## Style de code

### Typage
- Toujours typer les paramètres et retours de fonctions
- Typer les variables quand le type n'est pas évident

```gdscript
func take_damage(amount: int) -> void:
    var new_health: int = health - amount
```

### Organisation d'un script
1. `class_name` (si nécessaire)
2. `extends`
3. Signaux
4. Enums et constantes
5. `@export` variables
6. Variables publiques
7. Variables privées (préfixées `_`)
8. `@onready` variables
9. Fonctions built-in (`_ready`, `_process`, `_physics_process`)
10. Fonctions publiques
11. Fonctions privées (préfixées `_`)

### Signals
- Connecter les signaux par code plutôt que par l'éditeur (traçabilité)
- Nommer les handlers : `_on_NomEmetteur_nom_signal`

```gdscript
func _ready() -> void:
    health_component.health_changed.connect(_on_HealthComponent_health_changed)
```

## Patterns recommandés

### State Machine
- Utiliser un enum pour les états
- Switch dans `_process` ou `_physics_process`
- Fonctions `_enter_state()` et `_exit_state()` pour les transitions

### Composition > Héritage
- Préférer les scènes composées (components) à l'héritage profond
- Un component = un script réutilisable attaché à un node

### Autoloads (Singletons)
- Réservés au state global (GameManager, AudioManager, SaveManager)
- Maximum 5-6 autoloads — si plus, repenser l'architecture
- Documenter chaque autoload dans `architecture.md`

## Anti-patterns à éviter
- `get_node()` avec des chemins absolus → utiliser `@onready` et `$RelativePath`
- Logique métier dans les nodes UI → séparer UI et logique
- Signaux non typés → toujours typer les paramètres de signaux
- Scripts monolithiques > 200 lignes → découper en components
