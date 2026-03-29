# Patterns courants — Flutter

> Référence rapide des patterns recommandés pour les projets Flutter.
> Le Codeur et l'Architecte consultent ce fichier pour choisir le bon pattern.

## Repository pattern

```dart
// domain/repositories/user_repository.dart
abstract class UserRepository {
  Future<User> getUser(String id);
  Future<void> saveUser(User user);
}

// data/repositories/user_repository_impl.dart
class UserRepositoryImpl implements UserRepository {
  final ApiClient _apiClient;
  final LocalDatabase _localDb;

  UserRepositoryImpl(this._apiClient, this._localDb);

  @override
  Future<User> getUser(String id) async {
    try {
      final user = await _apiClient.fetchUser(id);
      await _localDb.cacheUser(user);
      return user;
    } catch (e) {
      // Fallback to cache
      return await _localDb.getUser(id);
    }
  }
}
```

## Feature-first structure

```
features/
└── inventory/
    ├── data/
    │   ├── models/
    │   │   └── inventory_item_model.dart
    │   └── repositories/
    │       └── inventory_repository_impl.dart
    ├── domain/
    │   ├── entities/
    │   │   └── inventory_item.dart
    │   └── repositories/
    │       └── inventory_repository.dart
    └── presentation/
        ├── screens/
        │   └── inventory_screen.dart
        └── widgets/
            ├── inventory_grid.dart
            └── item_card.dart
```

## State management avec Riverpod (exemple)

```dart
// Provider
final userProvider = FutureProvider.family<User, String>((ref, userId) async {
  final repository = ref.read(userRepositoryProvider);
  return repository.getUser(userId);
});

// Screen
class UserProfileScreen extends ConsumerWidget {
  const UserProfileScreen({super.key, required this.userId});
  final String userId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final userAsync = ref.watch(userProvider(userId));

    return userAsync.when(
      data: (user) => UserProfileView(user: user),
      loading: () => const CircularProgressIndicator(),
      error: (err, stack) => ErrorWidget(message: err.toString()),
    );
  }
}
```

## Service pattern

```dart
// shared/services/notification_service.dart
class NotificationService {
  Future<void> showLocalNotification({
    required String title,
    required String body,
  }) async {
    // implementation
  }

  Future<bool> requestPermission() async {
    // implementation
  }
}
```

## Extension methods (utilitaires)

```dart
// core/extensions/context_extensions.dart
extension BuildContextExtensions on BuildContext {
  ThemeData get theme => Theme.of(this);
  TextTheme get textTheme => theme.textTheme;
  ColorScheme get colorScheme => theme.colorScheme;
  double get screenWidth => MediaQuery.sizeOf(this).width;
}
```

## Error handling pattern

```dart
// core/utils/result.dart
sealed class Result<T> {
  const Result();
}

class Success<T> extends Result<T> {
  final T data;
  const Success(this.data);
}

class Failure<T> extends Result<T> {
  final String message;
  final Exception? exception;
  const Failure(this.message, [this.exception]);
}
```
