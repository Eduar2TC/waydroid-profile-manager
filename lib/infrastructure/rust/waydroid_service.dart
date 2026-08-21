import '../../domain/entities/profile.dart';
import '../../domain/entities/waydroid_status.dart';

/// Abstract interface for Waydroid operations.
/// Production: FfiWaydroidService (calls Rust)
/// Development: MockWaydroidService (simulated)
abstract class WaydroidService {
  bool init();
  WaydroidStatus getStatus();
  List<Profile> getProfiles();
  Profile? getActiveProfile();
  bool createProfile({
    required String name,
    String? description,
    required int baseWidth,
    required int baseHeight,
    required int density,
    required String orientation,
  });
  bool deleteProfile(String id);
  Profile? duplicateProfile(String id, {String? newName});
  bool applyProfile(String profileId, String orientation);
  bool reset();

  /// Export non-builtin profiles as JSON string.
  String exportProfiles();

  /// Import profiles from JSON string. Returns count imported.
  int importProfiles(String json);
}
