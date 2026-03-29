# Conventions Flutter — Dart

> Ce fichier est lu par le Codeur avant chaque phase d'implémentation sur un projet Flutter.
> À compléter/adapter selon le projet.

## Versions
- Flutter : {{x.x}}
- Dart : {{x.x}}
- State management : {{Provider / Riverpod / BLoC / autre}}

## Structure du projet

```
lib/
├── main.dart
├── app.dart                ← MaterialApp, routing, theme
├── core/                   ← Utilitaires partagés
│   ├── constants.dart
│   ├── theme.dart
│   └── utils/
├── features/               ← Feature-first architecture
│   ├── auth/
│   │   ├── data/           ← Repositories, data sources
│   │   ├── domain/         ← Models, entities
│   │   └── presentation/   ← Screens, widgets
│   └── home/
│       ├── data/
│       ├── domain/
│       └── presentation/
├── shared/                 ← Widgets et services partagés
│   ├── widgets/
│   └── services/
└── l10n/                   ← Internationalisation (si applicable)
```

## Naming

### Fichiers
- Tout en `snake_case.dart` (ex: `user_profile_screen.dart`)
- Un fichier = une classe publique principale
- Suffixes : `_screen.dart`, `_widget.dart`, `_model.dart`, `_repository.dart`, `_service.dart`

### Code Dart
- Classes : `PascalCase` (ex: `UserProfileScreen`)
- Fonctions/méthodes : `camelCase` (ex: `getUserProfile()`)
- Variables : `camelCase` (ex: `final userName = '...'`)
- Constantes : `camelCase` avec `const` (ex: `const defaultPadding = 16.0`)
- Privé : préfixe `_` (ex: `_isLoading`)
- Fichiers : `snake_case` (ex: `user_profile.dart`)

## Style de code

### Widgets
- Préférer `StatelessWidget` quand pas de state local
- Extraire les widgets complexes en sous-widgets (pas de `build()` > 80 lignes)
- Utiliser `const` constructors quand possible

```dart
class UserAvatar extends StatelessWidget {
  const UserAvatar({super.key, required this.imageUrl, this.size = 40});

  final String imageUrl;
  final double size;

  @override
  Widget build(BuildContext context) {
    return ClipOval(
      child: Image.network(imageUrl, width: size, height: size),
    );
  }
}
```

### Organisation d'un fichier
1. Imports (dart: → package: → relative)
2. Constantes du fichier
3. Classe principale
4. Classes helper (si petites et liées)

### Null safety
- Toujours utiliser le null safety strict
- Préférer `final` pour les variables non réassignées
- Éviter `!` (force unwrap) — gérer le cas null explicitement

## Patterns recommandés

### Feature-first
- Chaque feature a son propre dossier avec data/domain/presentation
- Les dépendances entre features passent par des interfaces (pas de couplage direct)

### Repository pattern
- Les repositories abstraient l'accès aux données (API, local DB, cache)
- Les screens ne parlent jamais directement aux data sources

### State management (à adapter selon le choix du projet)
- Documenter le choix dans un ADR
- Être cohérent : un seul pattern de state management dans tout le projet

## Anti-patterns à éviter
- Widgets monolithiques > 100 lignes de `build()` → extraire
- Logique métier dans les widgets → déplacer dans services/repositories
- `setState()` pour du state partagé entre screens → utiliser le state management choisi
- Imports circulaires entre features → revoir l'architecture
- Packages non déclarés dans `pubspec.yaml` → toujours vérifier
