import 'dart:convert';
import 'dart:math';


import 'package:waydroid_profile_manager/domain/entities/profile.dart';
import 'package:waydroid_profile_manager/domain/entities/waydroid_status.dart';
import 'package:waydroid_profile_manager/domain/enums/orientation.dart';

import 'waydroid_service.dart';

/// Mock Waydroid service for development and testing.
/// Simulates waydroid behavior without requiring real installation.
class MockWaydroidService implements WaydroidService {
  List<Profile> _profiles = [];
  Profile? _activeProfile;
  final Random _random = Random();

  MockWaydroidService() {
    _initBuiltinProfiles();
  }

  void _initBuiltinProfiles() {
    final now = DateTime.now();
    _profiles = [
      Profile(
        id: 'phone-sm',
        name: 'Phone Small',
        description: 'Small phone profile',
        baseWidth: 640,
        baseHeight: 360,
        density: 160,
        defaultOrientation: WaydroidOrientation.portrait,
        createdAt: now,
        updatedAt: now,
        builtin: true,
      ),
      Profile(
        id: 'phone-md',
        name: 'Phone Medium',
        description: 'Medium phone profile',
        baseWidth: 800,
        baseHeight: 450,
        density: 180,
        defaultOrientation: WaydroidOrientation.portrait,
        createdAt: now,
        updatedAt: now,
        builtin: true,
      ),
      Profile(
        id: 'phone-lg',
        name: 'Phone Large',
        description: 'Large phone profile',
        baseWidth: 960,
        baseHeight: 540,
        density: 200,
        defaultOrientation: WaydroidOrientation.portrait,
        createdAt: now,
        updatedAt: now,
        builtin: true,
      ),
      Profile(
        id: 'tablet-sm',
        name: 'Tablet Small',
        description: 'Small tablet profile',
        baseWidth: 1024,
        baseHeight: 640,
        density: 160,
        defaultOrientation: WaydroidOrientation.landscape,
        createdAt: now,
        updatedAt: now,
        builtin: true,
      ),
      Profile(
        id: 'tablet-md',
        name: 'Tablet Medium',
        description: 'Medium tablet profile',
        baseWidth: 1280,
        baseHeight: 800,
        density: 160,
        defaultOrientation: WaydroidOrientation.landscape,
        createdAt: now,
        updatedAt: now,
        builtin: true,
      ),
      Profile(
        id: 'tablet-lg',
        name: 'Tablet Large',
        description: 'Large tablet profile',
        baseWidth: 1600,
        baseHeight: 1000,
        density: 160,
        defaultOrientation: WaydroidOrientation.landscape,
        createdAt: now,
        updatedAt: now,
        builtin: true,
      ),
    ];
  }

  @override
  bool init() {
    return true;
  }

  @override
  WaydroidStatus getStatus() {
    return WaydroidStatus(
      installed: true,
      container: 'running',
      session: 'running',
      ui: _activeProfile != null ? 'open' : 'closed',
    );
  }

  @override
  List<Profile> getProfiles() => List.unmodifiable(_profiles);

  @override
  Profile? getActiveProfile() => _activeProfile;

  @override
  bool createProfile({
    required String name,
    String? description,
    required int baseWidth,
    required int baseHeight,
    required int density,
    required String orientation,
  }) {
    final now = DateTime.now();
    final profile = Profile(
      id: 'custom-${_random.nextInt(100000)}',
      name: name,
      description: description,
      baseWidth: baseWidth,
      baseHeight: baseHeight,
      density: density,
      defaultOrientation: WaydroidOrientation.fromString(orientation),
      createdAt: now,
      updatedAt: now,
      builtin: false,
    );
    _profiles.add(profile);
    return true;
  }

  @override
  bool updateProfile({
    required String id,
    String? name,
    String? description,
    required int baseWidth,
    required int baseHeight,
    required int density,
    required String orientation,
  }) {
    final idx = _profiles.indexWhere((p) => p.id == id);
    if (idx == -1) return false;
    if (_profiles[idx].builtin) return false;
    _profiles[idx] = _profiles[idx].copyWith(
      name: name,
      description: description,
      baseWidth: baseWidth,
      baseHeight: baseHeight,
      density: density,
      defaultOrientation: WaydroidOrientation.fromString(orientation),
    );
    return true;
  }

  @override
  bool deleteProfile(String id) {
    final idx = _profiles.indexWhere((p) => p.id == id);
    if (idx == -1) return false;
    if (_profiles[idx].builtin) return false;
    _profiles.removeAt(idx);
    if (_activeProfile?.id == id) _activeProfile = null;
    return true;
  }

  @override
  Profile? duplicateProfile(String id, {String? newName}) {
    final original = _profiles.firstWhere(
      (p) => p.id == id,
      orElse: () => throw Exception('Profile not found'),
    );
    final now = DateTime.now();
    final dup = Profile(
      id: 'custom-${_random.nextInt(100000)}',
      name: newName ?? '${original.name} (Copy)',
      description: original.description,
      baseWidth: original.baseWidth,
      baseHeight: original.baseHeight,
      density: original.density,
      defaultOrientation: original.defaultOrientation,
      createdAt: now,
      updatedAt: now,
      builtin: false,
    );
    _profiles.add(dup);
    return dup;
  }

  @override
  Map<String, dynamic> applyProfile(String profileId, String orientation) {
    final profile = _profiles.firstWhere(
      (p) => p.id == profileId,
      orElse: () => throw Exception('Profile not found'),
    );
    _activeProfile = profile;
    return {'success': true, 'message': 'Applied'};
  }

  bool _mockAsyncDone = true;
  ApplyProgress? _mockProgress;

  @override
  bool applyProfileAsync(String profileId, String orientation) {
    // Mock: apply synchronously, set progress to completed
    final result = applyProfile(profileId, orientation);
    _mockProgress = ApplyProgress(
      step: 'Completed',
      message: result['message'] as String? ?? 'Applied',
      completed: true,
      done: true,
      success: result['success'] as bool? ?? false,
    );
    _mockAsyncDone = true;
    return true;
  }

  @override
  ApplyProgress? getApplyProgress() => _mockProgress;

  @override
  bool isApplyDone() => _mockAsyncDone;

  @override
  bool reset() {
    _activeProfile = null;
    return true;
  }

  @override
  String exportProfiles() {
    final custom = _profiles.where((p) => !p.builtin).toList();
    return const JsonEncoder.withIndent('  ').convert(
      custom.map((p) => p.toJson()).toList(),
    );
  }

  @override
  int importProfiles(String json) {
    try {
      final list = jsonDecode(json) as List;
      var imported = 0;
      for (final item in list) {
        final map = item as Map<String, dynamic>;
        if (map['builtin'] == true) continue;
        final profile = Profile.fromJson(map);
        // Re-assign with new ID to avoid collisions
        final newProfile = Profile(
          id: 'custom-${_random.nextInt(100000)}',
          name: profile.name,
          description: profile.description,
          baseWidth: profile.baseWidth,
          baseHeight: profile.baseHeight,
          density: profile.density,
          defaultOrientation: profile.defaultOrientation,
          createdAt: DateTime.now(),
          updatedAt: DateTime.now(),
          builtin: false,
        );
        _profiles.add(newProfile);
        imported++;
      }
      return imported;
    } catch (_) {
      return -1;
    }
  }
}
