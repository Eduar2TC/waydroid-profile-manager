import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'package:ffi/ffi.dart';

/// Dart FFI bindings to the Rust Waydroid Profile Manager C-ABI.
///
/// This class loads `libwaydroid_profile_manager.so` and exposes
/// the raw FFI functions. Higher-level code should use
/// [FfiWaydroidService] instead of calling this directly.
class WaydroidBridge {
  static WaydroidBridge? _instance;
  late final DynamicLibrary _lib;
  late final int Function() _init;
  late final Pointer<Utf8> Function() _getStatus;
  late final Pointer<Utf8> Function() _getProfiles;
  late final Pointer<Utf8> Function() _getActiveProfile;
  late final int Function(Pointer<Utf8>, Pointer<Utf8>, int, int, int,
      Pointer<Utf8>) _createProfile;
  late final int Function(Pointer<Utf8>) _deleteProfile;
  late final Pointer<Utf8> Function(
      Pointer<Utf8>, Pointer<Utf8>) _duplicateProfile;
  late final int Function(Pointer<Utf8>, Pointer<Utf8>) _applyProfile;
  late final int Function() _reset;
  late final void Function(Pointer<Utf8>) _freeString;

  bool _initialized = false;
  bool _available = false;

  WaydroidBridge._() {
    try {
      _lib = _loadLibrary();
      _init = _lib.lookupFunction<Int32 Function(), int Function()>(
          'wpm_init');
      _getStatus = _lib.lookupFunction<Pointer<Utf8> Function(),
          Pointer<Utf8> Function()>('wpm_get_status');
      _getProfiles = _lib.lookupFunction<Pointer<Utf8> Function(),
          Pointer<Utf8> Function()>('wpm_get_profiles');
      _getActiveProfile = _lib.lookupFunction<Pointer<Utf8> Function(),
          Pointer<Utf8> Function()>('wpm_get_active_profile');
      _createProfile = _lib.lookupFunction<
          Int32 Function(Pointer<Utf8>, Pointer<Utf8>, Int32, Int32, Int32,
              Pointer<Utf8>),
          int Function(Pointer<Utf8>, Pointer<Utf8>, int, int, int,
              Pointer<Utf8>)>('wpm_create_profile');
      _deleteProfile = _lib.lookupFunction<
          Int32 Function(Pointer<Utf8>),
          int Function(Pointer<Utf8>)>('wpm_delete_profile');
      _duplicateProfile = _lib.lookupFunction<
          Pointer<Utf8> Function(Pointer<Utf8>, Pointer<Utf8>),
          Pointer<Utf8> Function(
              Pointer<Utf8>, Pointer<Utf8>)>('wpm_duplicate_profile');
      _applyProfile = _lib.lookupFunction<
          Int32 Function(Pointer<Utf8>, Pointer<Utf8>),
          int Function(
              Pointer<Utf8>, Pointer<Utf8>)>('wpm_apply_profile');
      _reset = _lib.lookupFunction<Int32 Function(), int Function()>(
          'wpm_reset');
      _freeString = _lib.lookupFunction<Void Function(Pointer<Utf8>),
          void Function(Pointer<Utf8>)>('wpm_free_string');
      _bindExportImport();
      _available = true;
    } catch (_) {
      _available = false;
    }
  }

  static WaydroidBridge get instance {
    _instance ??= WaydroidBridge._();
    return _instance!;
  }

  /// Whether the Rust library was loaded successfully.
  bool get isAvailable => _available;

  DynamicLibrary _loadLibrary() {
    if (!Platform.isLinux) {
      throw UnsupportedError('Rust FFI only supported on Linux');
    }

    // Search paths: try relative to CWD (flutter run), then absolute
    final candidates = [
      'rust/target/debug/libwaydroid_profile_manager.so',
      'rust/target/release/libwaydroid_profile_manager.so',
      'build/linux/x64/debug/bundle/libwaydroid_profile_manager.so',
      'build/linux/x64/release/bundle/libwaydroid_profile_manager.so',
      'libwaydroid_profile_manager.so',
    ];

    for (final path in candidates) {
      if (File(path).existsSync()) {
        return DynamicLibrary.open(path);
      }
    }

    throw FileSystemException(
        'Could not find libwaydroid_profile_manager.so');
  }

  /// Initialize the Rust backend.
  bool init() {
    if (!_available) return false;
    if (_initialized) return true;
    try {
      _initialized = _init() == 0;
      return _initialized;
    } catch (_) {
      return false;
    }
  }

  /// Call a Rust function that returns a C string, handle memory.
  String _callString(Pointer<Utf8> Function() fn) {
    if (!_available) return '';
    try {
      final ptr = fn();
      if (ptr == nullptr) return '';
      final result = ptr.toDartString();
      _freeString(ptr);
      return result;
    } catch (_) {
      return '';
    }
  }

  WaydroidStatusData getStatusData() {
    final json = _callString(_getStatus);
    if (json.isEmpty) return WaydroidStatusData.notInstalled();
    try {
      return WaydroidStatusData.fromJson(jsonDecode(json));
    } catch (_) {
      return WaydroidStatusData.notInstalled();
    }
  }

  List<Map<String, dynamic>> getProfilesJson() {
    final json = _callString(_getProfiles);
    if (json.isEmpty) return [];
    try {
      final list = jsonDecode(json) as List;
      return list.cast<Map<String, dynamic>>();
    } catch (_) {
      return [];
    }
  }

  Map<String, dynamic>? getActiveProfileJson() {
    final json = _callString(_getActiveProfile);
    if (json.isEmpty || json == 'null') return null;
    try {
      return jsonDecode(json) as Map<String, dynamic>;
    } catch (_) {
      return null;
    }
  }

  bool createProfile({
    required String name,
    String? description,
    required int baseWidth,
    required int baseHeight,
    required int density,
    required String orientation,
  }) {
    if (!_available) return false;
    try {
      final namePtr = name.toNativeUtf8();
      final descPtr = (description ?? '').toNativeUtf8();
      final orientPtr = orientation.toNativeUtf8();
      try {
        return _createProfile(
                namePtr, descPtr, baseWidth, baseHeight, density, orientPtr) ==
            0;
      } finally {
        calloc.free(namePtr);
        calloc.free(descPtr);
        calloc.free(orientPtr);
      }
    } catch (_) {
      return false;
    }
  }

  bool deleteProfile(String id) {
    if (!_available) return false;
    try {
      final idPtr = id.toNativeUtf8();
      try {
        return _deleteProfile(idPtr) == 0;
      } finally {
        calloc.free(idPtr);
      }
    } catch (_) {
      return false;
    }
  }

  Map<String, dynamic>? duplicateProfile(String id, {String? newName}) {
    if (!_available) return null;
    try {
      final idPtr = id.toNativeUtf8();
      final namePtr = (newName ?? '').toNativeUtf8();
      try {
        final jsonPtr = _duplicateProfile(idPtr, namePtr);
        if (jsonPtr == nullptr) return null;
        final json = jsonPtr.toDartString();
        _freeString(jsonPtr);
        if (json.isEmpty) return null;
        return jsonDecode(json) as Map<String, dynamic>;
      } finally {
        calloc.free(idPtr);
        calloc.free(namePtr);
      }
    } catch (_) {
      return null;
    }
  }

  bool applyProfile(String profileId, String orientation) {
    if (!_available) return false;
    try {
      final idPtr = profileId.toNativeUtf8();
      final orientPtr = orientation.toNativeUtf8();
      try {
        return _applyProfile(idPtr, orientPtr) == 0;
      } finally {
        calloc.free(idPtr);
        calloc.free(orientPtr);
      }
    } catch (_) {
      return false;
    }
  }

  bool reset() {
    if (!_available) return false;
    try {
      return _reset() == 0;
    } catch (_) {
      return false;
    }
  }

  // --- Export/Import ---

  late final Pointer<Utf8> Function() _exportProfiles;
  late final int Function(Pointer<Utf8>) _importProfiles;

  // Call this after _lib is loaded to bind export/import symbols.
  void _bindExportImport() {
    try {
      _exportProfiles = _lib.lookupFunction<Pointer<Utf8> Function(),
          Pointer<Utf8> Function()>('wpm_export_profiles');
      _importProfiles = _lib.lookupFunction<
          Int32 Function(Pointer<Utf8>),
          int Function(Pointer<Utf8>)>('wpm_import_profiles');
    } catch (_) {
      // symbols not found — disable
    }
  }

  /// Export non-builtin profiles as pretty-printed JSON.
  String exportProfiles() {
    if (!_available) return '[]';
    try {
      return _callString(_exportProfiles);
    } catch (_) {
      return '[]';
    }
  }

  /// Import profiles from a JSON string.
  /// Returns the number of profiles imported, or -1 on error.
  int importProfiles(String json) {
    if (!_available) return -1;
    try {
      final jsonPtr = json.toNativeUtf8();
      try {
        return _importProfiles(jsonPtr);
      } finally {
        calloc.free(jsonPtr);
      }
    } catch (_) {
      return -1;
    }
  }
}

/// Simple data class for status coming from Rust.
class WaydroidStatusData {
  final bool installed;
  final String container;
  final String session;
  final String ui;

  WaydroidStatusData({
    required this.installed,
    required this.container,
    required this.session,
    required this.ui,
  });

  factory WaydroidStatusData.fromJson(Map<String, dynamic> json) {
    return WaydroidStatusData(
      installed: json['installed'] as bool? ?? false,
      container: json['container'] as String? ?? 'stopped',
      session: json['session'] as String? ?? 'stopped',
      ui: json['ui'] as String? ?? 'closed',
    );
  }

  factory WaydroidStatusData.notInstalled() {
    return WaydroidStatusData(
        installed: false, container: 'stopped', session: 'stopped', ui: 'closed');
  }
}
