
import 'package:waydroid_profile_manager/domain/entities/profile.dart';
import 'package:waydroid_profile_manager/domain/entities/waydroid_status.dart';

/// Progress update during an async apply operation.
class ApplyProgress {
  final String step;
  final String message;
  final bool completed;
  final bool done;
  final bool success;

  ApplyProgress({
    required this.step,
    required this.message,
    required this.completed,
    required this.done,
    required this.success,
  });

  factory ApplyProgress.fromJson(Map<String, dynamic> json) {
    return ApplyProgress(
      step: json['step'] as String? ?? '',
      message: json['message'] as String? ?? '',
      completed: json['completed'] as bool? ?? false,
      done: json['done'] as bool? ?? false,
      success: json['success'] as bool? ?? false,
    );
  }
}

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
  bool updateProfile({
    required String id,
    String? name,
    String? description,
    required int baseWidth,
    required int baseHeight,
    required int density,
    required String orientation,
  });
  bool deleteProfile(String id);
  Profile? duplicateProfile(String id, {String? newName});
  /// Apply a profile (auto-restarts session for resolution changes).
  Map<String, dynamic> applyProfile(String profileId, String orientation);
  /// Start an async apply operation. Returns true if started.
  bool applyProfileAsync(String profileId, String orientation);
  /// Get the latest progress. Returns null if no apply in progress.
  ApplyProgress? getApplyProgress();
  /// Check if async apply is done. Returns true if done, false if running.
  bool isApplyDone();
  bool reset();

  /// Export non-builtin profiles as JSON string.
  String exportProfiles();

  /// Import profiles from JSON string. Returns count imported.
  int importProfiles(String json);
}
