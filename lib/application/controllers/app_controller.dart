import 'dart:async';

import 'package:flutter/foundation.dart';

import '../../domain/entities/profile.dart';
import '../../domain/entities/waydroid_status.dart';
import '../../domain/enums/orientation.dart';
import '../../infrastructure/rust/waydroid_service.dart';

/// Main application state controller.
/// Manages all application state and coordinates with the Waydroid service.
class AppController extends ChangeNotifier {
  final WaydroidService _service;

  WaydroidStatus _waydroidStatus = WaydroidStatus.notInstalled();
  List<Profile> _profiles = [];
  Profile? _activeProfile;
  WaydroidOrientation _selectedOrientation = WaydroidOrientation.portrait;
  bool _isApplying = false;
  String? _error;
  List<String> _logs = [];
  ApplyProgress? _applyProgress;
  Timer? _progressTimer;

  AppController(this._service);

  // Getters
  WaydroidStatus get waydroidStatus => _waydroidStatus;
  List<Profile> get profiles => _profiles;
  Profile? get activeProfile => _activeProfile;
  WaydroidOrientation get selectedOrientation => _selectedOrientation;
  bool get isApplying => _isApplying;
  ApplyProgress? get applyProgress => _applyProgress;
  String? get error => _error;
  List<String> get logs => List.unmodifiable(_logs);
  bool get isReady => _waydroidStatus.installed;

  /// Initialize the application.
  Future<void> init() async {
    _addLog('Initializing Waydroid Profile Manager...');
    final success = _service.init();
    if (!success) {
      _error = 'Failed to initialize Rust backend';
      notifyListeners();
      return;
    }
    _addLog('Backend initialized');
    await refresh();
  }

  /// Refresh all state from the service.
  Future<void> refresh() async {
    _waydroidStatus = _service.getStatus();
    _profiles = _service.getProfiles();
    _activeProfile = _service.getActiveProfile();
    _addLog('Status: ${_waydroidStatus.summary}');
    notifyListeners();
  }

  /// Set the selected orientation.
  void setOrientation(WaydroidOrientation orientation) {
    if (_selectedOrientation != orientation) {
      _selectedOrientation = orientation;
      notifyListeners();
    }
  }

  /// Apply a profile to Waydroid.
  /// Uses async FFI with progress polling for a responsive UI.
  Future<void> applyProfile(Profile profile) async {
    if (_isApplying) return;

    _isApplying = true;
    _error = null;
    _applyProgress = null;
    _addLog('Applying profile: ${profile.name}');
    _addLog('Orientation: ${_selectedOrientation.displayName}');
    notifyListeners();

    final dims = profile.getActualDimensions(_selectedOrientation);
    _addLog('Resolution: ${dims[0]}×${dims[1]}, Density: ${profile.density}');

    // Try async apply with progress polling
    final started = _service.applyProfileAsync(
      profile.id,
      _selectedOrientation.apiKey,
    );

    if (started) {
      // Poll for progress until done
      final completer = Completer<Map<String, dynamic>>();
      _progressTimer = Timer.periodic(const Duration(milliseconds: 300), (timer) {
        final progress = _service.getApplyProgress();
        if (progress != null) {
          _applyProgress = progress;
          _addLog('${progress.step}: ${progress.message}');
          notifyListeners();
        }

        if (_service.isApplyDone()) {
          timer.cancel();
          _progressTimer = null;
          final finalProgress = _service.getApplyProgress();
          final success = finalProgress?.success ?? false;
          final message = finalProgress?.message ?? 'Unknown result';
          completer.complete({
            'success': success,
            'message': message,
          });
        }
      });

      try {
        final result = await completer.future;
        final success = result['success'] as bool? ?? false;

        if (success) {
          _activeProfile = profile;
          _addLog('✓ Profile applied successfully');
        } else {
          _error = result['message'] as String? ?? 'Failed to apply profile';
          _addLog('✗ ${_error}');
        }
      } catch (e) {
        _error = e.toString();
        _addLog('✗ Error: $e');
      }
    } else {
      // Fallback to sync apply
      try {
        final result = _service.applyProfile(
          profile.id,
          _selectedOrientation.apiKey,
        );

        final success = result['success'] as bool? ?? false;

        if (success) {
          _activeProfile = profile;
          _addLog('✓ Profile applied successfully');
        } else {
          _error = result['message'] as String? ?? 'Failed to apply profile';
          _addLog('✗ ${_error}');
        }
      } catch (e) {
        _error = e.toString();
        _addLog('✗ Error: $e');
      }
    }

    _isApplying = false;
    _applyProgress = null;
    notifyListeners();
  }

  /// Create a new profile.
  bool createProfile({
    required String name,
    String? description,
    required int baseWidth,
    required int baseHeight,
    required int density,
    required WaydroidOrientation orientation,
  }) {
    final success = _service.createProfile(
      name: name,
      description: description,
      baseWidth: baseWidth,
      baseHeight: baseHeight,
      density: density,
      orientation: orientation.apiKey,
    );
    if (success) {
      _profiles = _service.getProfiles();
      _addLog('Created profile: $name');
      notifyListeners();
    }
    return success;
  }

  /// Update a profile.
  // Note: For MVP, we recreate profiles. Full update would require backend support.
  bool updateProfile({
    required String id,
    String? name,
    String? description,
    int? baseWidth,
    int? baseHeight,
    int? density,
    WaydroidOrientation? orientation,
  }) {
    final idx = _profiles.indexWhere((p) => p.id == id);
    if (idx == -1) return false;

    final old = _profiles[idx];
    if (old.builtin) return false;

    // Persist to Rust backend
    final success = _service.updateProfile(
      id: id,
      name: name,
      description: description,
      baseWidth: baseWidth ?? old.baseWidth,
      baseHeight: baseHeight ?? old.baseHeight,
      density: density ?? old.density,
      orientation: (orientation ?? old.defaultOrientation).apiKey,
    );

    if (success) {
      // Reload from service to get the persisted data
      _profiles = _service.getProfiles();
      _addLog('Updated profile: ${_profiles[idx].name}');
    } else {
      // Fallback to local update
      _profiles[idx] = old.copyWith(
        name: name,
        description: description,
        baseWidth: baseWidth,
        baseHeight: baseHeight,
        density: density,
        defaultOrientation: orientation,
      );
      _addLog('Updated profile locally: ${_profiles[idx].name}');
    }
    notifyListeners();
    return true;
  }

  /// Delete a profile.
  bool deleteProfile(String id) {
    final success = _service.deleteProfile(id);
    if (success) {
      _profiles = _service.getProfiles();
      if (_activeProfile?.id == id) {
        _activeProfile = null;
      }
      _addLog('Deleted profile');
      notifyListeners();
    } else {
      _error = 'Cannot delete built-in profile';
      notifyListeners();
    }
    return success;
  }

  /// Duplicate a profile.
  Profile? duplicateProfile(String id, {String? newName}) {
    final dup = _service.duplicateProfile(id, newName: newName);
    if (dup != null) {
      _profiles = _service.getProfiles();
      _addLog('Duplicated profile: ${dup.name}');
      notifyListeners();
    }
    return dup;
  }

  /// Reset Waydroid configuration.
  Future<void> reset() async {
    _addLog('Resetting configuration...');
    final success = _service.reset();
    if (success) {
      _activeProfile = null;
      _addLog('✓ Configuration reset');
    } else {
      _error = 'Failed to reset configuration';
      _addLog('✗ Reset failed');
    }
    notifyListeners();
  }

  /// Clear error state.
  void clearError() {
    _error = null;
    notifyListeners();
  }

  /// Export non-builtin profiles as JSON.
  String exportProfiles() {
    final json = _service.exportProfiles();
    _addLog('Exported profiles (${_profiles.where((p) => !p.builtin).length} custom)');
    return json;
  }

  /// Import profiles from JSON string.
  int importProfiles(String json) {
    final count = _service.importProfiles(json);
    if (count > 0) {
      _profiles = _service.getProfiles();
      _addLog('✓ Imported $count profile(s)');
      notifyListeners();
    } else if (count == -1) {
      _error = 'Failed to import profiles — invalid JSON';
      _addLog('✗ Import failed: invalid JSON');
      notifyListeners();
    } else {
      _addLog('No new profiles to import');
    }
    return count;
  }

  /// Clear logs.
  void clearLogs() {
    _logs.clear();
    notifyListeners();
  }

  void _addLog(String message) {
    final now = DateTime.now();
    final timestamp =
        '${now.hour.toString().padLeft(2, '0')}:'
        '${now.minute.toString().padLeft(2, '0')}:'
        '${now.second.toString().padLeft(2, '0')}';
    _logs.add('[$timestamp] $message');
    // Keep only last 200 entries
    if (_logs.length > 200) {
      _logs.removeRange(0, 50);
    }
  }
}
