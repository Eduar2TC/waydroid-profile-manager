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

  AppController(this._service);

  // Getters
  WaydroidStatus get waydroidStatus => _waydroidStatus;
  List<Profile> get profiles => _profiles;
  Profile? get activeProfile => _activeProfile;
  WaydroidOrientation get selectedOrientation => _selectedOrientation;
  bool get isApplying => _isApplying;
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
  Future<void> applyProfile(Profile profile) async {
    if (_isApplying) return;

    _isApplying = true;
    _error = null;
    _addLog('Applying profile: ${profile.name}');
    _addLog('Orientation: ${_selectedOrientation.displayName}');
    notifyListeners();

    try {
      final dims = profile.getActualDimensions(_selectedOrientation);
      _addLog('Resolution: ${dims[0]}×${dims[1]}, Density: ${profile.density}');

      final success = _service.applyProfile(
        profile.id,
        _selectedOrientation.apiKey,
      );

      if (success) {
        _activeProfile = profile;
        _addLog('✓ Profile applied successfully');
      } else {
        _error = 'Failed to apply profile';
        _addLog('✗ Failed to apply profile');
      }
    } catch (e) {
      _error = e.toString();
      _addLog('✗ Error: $e');
    } finally {
      _isApplying = false;
      notifyListeners();
    }
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
    // Find and update locally, then recreate via service
    final idx = _profiles.indexWhere((p) => p.id == id);
    if (idx == -1) return false;

    final old = _profiles[idx];
    if (old.builtin) return false;

    // Delete old and create new with same id approach
    // For now, just update the local state
    _profiles[idx] = old.copyWith(
      name: name,
      description: description,
      baseWidth: baseWidth,
      baseHeight: baseHeight,
      density: density,
      defaultOrientation: orientation,
    );
    _addLog('Updated profile: ${_profiles[idx].name}');
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
