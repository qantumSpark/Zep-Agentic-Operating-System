# Patterns courants — Godot

> Référence rapide des patterns recommandés pour les projets Godot.
> Le Codeur et l'Architecte consultent ce fichier pour choisir le bon pattern.

## State Machine (FSM simple)

```gdscript
class_name StateMachine
extends Node

enum State { IDLE, RUN, JUMP, ATTACK }

var current_state: State = State.IDLE

func transition_to(new_state: State) -> void:
    _exit_state(current_state)
    current_state = new_state
    _enter_state(new_state)

func _enter_state(state: State) -> void:
    match state:
        State.IDLE:
            pass # setup idle
        State.RUN:
            pass # setup run

func _exit_state(state: State) -> void:
    pass # cleanup si nécessaire

func _physics_process(delta: float) -> void:
    match current_state:
        State.IDLE:
            _process_idle(delta)
        State.RUN:
            _process_run(delta)
```

## Component pattern

```gdscript
# health_component.gd
class_name HealthComponent
extends Node

signal health_changed(new_health: int, max_health: int)
signal died

@export var max_health: int = 100
var current_health: int

func _ready() -> void:
    current_health = max_health

func take_damage(amount: int) -> void:
    current_health = max(0, current_health - amount)
    health_changed.emit(current_health, max_health)
    if current_health <= 0:
        died.emit()

func heal(amount: int) -> void:
    current_health = min(max_health, current_health + amount)
    health_changed.emit(current_health, max_health)
```

## Custom Resource (données)

```gdscript
# item_data.gd
class_name ItemData
extends Resource

@export var name: String = ""
@export var description: String = ""
@export var icon: Texture2D
@export var stack_size: int = 1
@export var value: int = 0
```

Usage : créer des `.tres` dans l'éditeur pour chaque item.

## Observer via Signals

```gdscript
# Émetteur
signal inventory_changed(items: Array[ItemData])

func add_item(item: ItemData) -> void:
    _items.append(item)
    inventory_changed.emit(_items)

# Récepteur (UI par exemple)
func _ready() -> void:
    Inventory.inventory_changed.connect(_on_inventory_changed)

func _on_inventory_changed(items: Array[ItemData]) -> void:
    _refresh_display(items)
```

## Service Locator (alternative aux Autoloads)

```gdscript
# game_services.gd (Autoload unique)
class_name GameServices
extends Node

var audio: AudioManager
var save: SaveManager

func _ready() -> void:
    audio = AudioManager.new()
    save = SaveManager.new()
    add_child(audio)
    add_child(save)
```

Un seul autoload qui centralise l'accès aux services.
