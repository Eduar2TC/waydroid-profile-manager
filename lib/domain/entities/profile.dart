import 'package:waydroid_profile_manager/domain/enums/orientation.dart';

/// A Waydroid device profile.
class Profile {
  final String id;
  final String name;
  final String? description;
  final int baseWidth;
  final int baseHeight;
  final int density;
  final WaydroidOrientation defaultOrientation;
  final DateTime createdAt;
  final DateTime updatedAt;
  final bool builtin;

  const Profile({
    required this.id,
    required this.name,
    this.description,
    required this.baseWidth,
    required this.baseHeight,
    required this.density,
    required this.defaultOrientation,
    required this.createdAt,
    required this.updatedAt,
    required this.builtin,
  });

  factory Profile.fromJson(Map<String, dynamic> json) {
    final now = DateTime.now();
    return Profile(
      id: json['id'] as String? ?? 'imported-${now.millisecondsSinceEpoch}',
      name: json['name'] as String,
      description: json['description'] as String?,
      baseWidth: json['base_width'] as int,
      baseHeight: json['base_height'] as int,
      density: json['density'] as int,
      defaultOrientation: WaydroidOrientation.fromString(
        json['default_orientation'] as String? ?? 'portrait',
      ),
      createdAt: json['created_at'] != null
          ? DateTime.parse(json['created_at'] as String)
          : now,
      updatedAt: json['updated_at'] != null
          ? DateTime.parse(json['updated_at'] as String)
          : now,
      builtin: json['builtin'] as bool? ?? false,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'name': name,
      'description': description,
      'base_width': baseWidth,
      'base_height': baseHeight,
      'density': density,
      'default_orientation': defaultOrientation.apiKey,
      'created_at': createdAt.toIso8601String(),
      'updated_at': updatedAt.toIso8601String(),
      'builtin': builtin,
    };
  }

  Profile copyWith({
    String? name,
    String? description,
    int? baseWidth,
    int? baseHeight,
    int? density,
    WaydroidOrientation? defaultOrientation,
  }) {
    return Profile(
      id: id,
      name: name ?? this.name,
      description: description ?? this.description,
      baseWidth: baseWidth ?? this.baseWidth,
      baseHeight: baseHeight ?? this.baseHeight,
      density: density ?? this.density,
      defaultOrientation: defaultOrientation ?? this.defaultOrientation,
      createdAt: createdAt,
      updatedAt: DateTime.now(),
      builtin: builtin,
    );
  }

  /// Get the actual dimensions for a given orientation.
  List<int> getActualDimensions(WaydroidOrientation orientation) {
    return orientation.toActual(baseWidth, baseHeight);
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is Profile &&
          runtimeType == other.runtimeType &&
          id == other.id &&
          name == other.name &&
          baseWidth == other.baseWidth &&
          baseHeight == other.baseHeight &&
          density == other.density &&
          defaultOrientation == other.defaultOrientation;

  @override
  int get hashCode =>
      id.hashCode ^
      name.hashCode ^
      baseWidth.hashCode ^
      baseHeight.hashCode ^
      density.hashCode ^
      defaultOrientation.hashCode;
}
