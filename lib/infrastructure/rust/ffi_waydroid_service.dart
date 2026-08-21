import '../../domain/entities/profile.dart';
import '../../domain/entities/waydroid_status.dart';
import 'waydroid_bridge.dart';
import 'waydroid_service.dart';

/// Real Waydroid service backed by the Rust FFI library.
/// Falls back gracefully if the .so isn't available.
class FfiWaydroidService implements WaydroidService {
  final WaydroidBridge _bridge;
  bool _initOk = false;

  FfiWaydroidService() : _bridge = WaydroidBridge.instance;

  @override
  bool init() {
    if (!_bridge.isAvailable) return false;
    _initOk = _bridge.init();
    return _initOk;
  }

  /// Whether the Rust backend loaded successfully.
  bool get isAvailable => _bridge.isAvailable && _initOk;

  @override
  WaydroidStatus getStatus() {
    if (!isAvailable) return WaydroidStatus.notInstalled();
    try {
      final data = _bridge.getStatusData();
      return WaydroidStatus(
        installed: data.installed,
        container: data.container,
        session: data.session,
        ui: data.ui,
      );
    } catch (_) {
      return WaydroidStatus.notInstalled();
    }
  }

  @override
  List<Profile> getProfiles() {
    if (!isAvailable) return [];
    try {
      final jsonList = _bridge.getProfilesJson();
      return jsonList.map((e) => Profile.fromJson(e)).toList();
    } catch (_) {
      return [];
    }
  }

  @override
  Profile? getActiveProfile() {
    if (!isAvailable) return null;
    try {
      final json = _bridge.getActiveProfileJson();
      if (json == null) return null;
      return Profile.fromJson(json);
    } catch (_) {
      return null;
    }
  }

  @override
  bool createProfile({
    required String name,
    String? description,
    required int baseWidth,
    required int baseHeight,
    required int density,
    required String orientation,
  }) {
    if (!isAvailable) return false;
    return _bridge.createProfile(
      name: name,
      description: description,
      baseWidth: baseWidth,
      baseHeight: baseHeight,
      density: density,
      orientation: orientation,
    );
  }

  @override
  bool deleteProfile(String id) {
    if (!isAvailable) return false;
    return _bridge.deleteProfile(id);
  }

  @override
  Profile? duplicateProfile(String id, {String? newName}) {
    if (!isAvailable) return null;
    try {
      final json = _bridge.duplicateProfile(id, newName: newName);
      if (json == null) return null;
      return Profile.fromJson(json);
    } catch (_) {
      return null;
    }
  }

  @override
  bool applyProfile(String profileId, String orientation) {
    if (!isAvailable) return false;
    return _bridge.applyProfile(profileId, orientation);
  }

  @override
  bool reset() {
    if (!isAvailable) return false;
    return _bridge.reset();
  }

  @override
  String exportProfiles() {
    if (!isAvailable) return '[]';
    return _bridge.exportProfiles();
  }

  @override
  int importProfiles(String json) {
    if (!isAvailable) return -1;
    return _bridge.importProfiles(json);
  }
}
