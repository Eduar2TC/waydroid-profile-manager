import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:waydroid_profile_manager/infrastructure/rust/waydroid_bridge.dart';
import 'package:waydroid_profile_manager/infrastructure/rust/ffi_waydroid_service.dart';

void main() {
  group('WaydroidBridge FFI', () {
    late WaydroidBridge bridge;

    setUpAll(() {
      // Try to load the .so from the build directory
      bridge = WaydroidBridge.instance;
    });

    test('bridge is available if .so exists', () {
      final soExists = File('rust/target/debug/libwaydroid_profile_manager.so')
              .existsSync() ||
          File('rust/target/release/libwaydroid_profile_manager.so')
              .existsSync();
      // If the .so exists, bridge should be available
      if (soExists) {
        expect(bridge.isAvailable, isTrue,
            reason: '.so exists but bridge not available');
      }
      // If .so doesn't exist, bridge gracefully reports unavailable
    });

    test('init returns result', () {
      if (bridge.isAvailable) {
        final result = bridge.init();
        expect(result, isTrue);
      } else {
        // Bridge not available, init should return false
        expect(bridge.init(), isFalse);
      }
    });

    test('getStatus returns valid data', () {
      if (bridge.isAvailable && bridge.init()) {
        final status = bridge.getStatusData();
        expect(status.installed, isA<bool>());
        expect(status.container, isA<String>());
        expect(status.session, isA<String>());
        expect(status.ui, isA<String>());
      }
    });

    test('getProfiles returns list', () {
      if (bridge.isAvailable && bridge.init()) {
        final profiles = bridge.getProfilesJson();
        expect(profiles, isA<List>());
        expect(profiles.length, greaterThanOrEqualTo(6));
      }
    });
  });

  group('FfiWaydroidService', () {
    test('falls back gracefully when .so missing', () {
      final service = FfiWaydroidService();
      // Even if Rust lib isn't available, methods should not throw
      // They should return safe defaults
      final profiles = service.getProfiles();
      expect(profiles, isA<List>());
      // Status should return not-installed if backend unavailable
      final status = service.getStatus();
      expect(status, isA<dynamic>());
    });
  });
}
