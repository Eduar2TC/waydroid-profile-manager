/// Screen orientation for Waydroid profiles.
enum WaydroidOrientation {
  landscape,
  portrait;

  /// Convert base dimensions to actual dimensions based on orientation.
  /// Landscape: width = baseWidth, height = baseHeight, rotation = 1
  /// Portrait:  width = baseHeight, height = baseWidth, rotation = 0
  List<int> toActual(int baseWidth, int baseHeight) {
    switch (this) {
      case WaydroidOrientation.landscape:
        return [baseWidth, baseHeight, 1];
      case WaydroidOrientation.portrait:
        return [baseHeight, baseWidth, 0];
    }
  }

  String get displayName {
    switch (this) {
      case WaydroidOrientation.landscape:
        return 'Landscape';
      case WaydroidOrientation.portrait:
        return 'Portrait';
    }
  }

  String get apiKey {
    switch (this) {
      case WaydroidOrientation.landscape:
        return 'landscape';
      case WaydroidOrientation.portrait:
        return 'portrait';
    }
  }

  static WaydroidOrientation fromString(String value) {
    switch (value) {
      case 'landscape':
        return WaydroidOrientation.landscape;
      case 'portrait':
        return WaydroidOrientation.portrait;
      default:
        return WaydroidOrientation.portrait;
    }
  }
}
