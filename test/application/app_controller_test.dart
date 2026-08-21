import 'package:flutter_test/flutter_test.dart';
import 'package:waydroid_profile_manager/application/controllers/app_controller.dart';
import 'package:waydroid_profile_manager/domain/enums/orientation.dart';
import 'package:waydroid_profile_manager/infrastructure/rust/mock_waydroid_service.dart';

void main() {
  group('AppController', () {
    late AppController controller;

    setUp(() {
      controller = AppController(MockWaydroidService());
      controller.init();
    });

    test('initializes with profiles', () {
      expect(controller.profiles.length, 6);
    });

    test('initializes with status', () {
      expect(controller.waydroidStatus.installed, isTrue);
    });

    test('set orientation', () {
      expect(controller.selectedOrientation, WaydroidOrientation.portrait);
      controller.setOrientation(WaydroidOrientation.landscape);
      expect(controller.selectedOrientation, WaydroidOrientation.landscape);
    });

    test('set same orientation does not trigger notify', () {
      var notifyCount = 0;
      controller.addListener(() => notifyCount++);
      controller.setOrientation(WaydroidOrientation.portrait); // same
      expect(notifyCount, 0);
    });

    test('create profile', () {
      final success = controller.createProfile(
        name: 'Custom Phone',
        baseWidth: 1080,
        baseHeight: 2400,
        density: 420,
        orientation: WaydroidOrientation.portrait,
      );
      expect(success, isTrue);
      expect(controller.profiles.length, 7);
    });

    test('apply profile sets active', () async {
      final profile = controller.profiles.first;
      await controller.applyProfile(profile);
      expect(controller.activeProfile?.id, profile.id);
      expect(controller.error, isNull);
    });

    test('delete custom profile', () {
      controller.createProfile(
        name: 'Delete Me',
        baseWidth: 800,
        baseHeight: 600,
        density: 160,
        orientation: WaydroidOrientation.portrait,
      );
      expect(controller.profiles.length, 7);

      final customId = controller.profiles.last.id;
      final success = controller.deleteProfile(customId);
      expect(success, isTrue);
      expect(controller.profiles.length, 6);
    });

    test('cannot delete built-in profile', () {
      final success = controller.deleteProfile('phone-sm');
      expect(success, isFalse);
    });

    test('reset clears active profile', () async {
      await controller.applyProfile(controller.profiles.first);
      expect(controller.activeProfile, isNotNull);

      await controller.reset();
      expect(controller.activeProfile, isNull);
    });

    test('logs are recorded', () {
      expect(controller.logs.isNotEmpty, isTrue);
      expect(controller.logs.any((l) => l.contains('Initializing')), isTrue);
    });

    test('clear logs', () {
      controller.clearLogs();
      expect(controller.logs.isEmpty, isTrue);
    });

    test('clear error', () {
      controller.clearError();
      expect(controller.error, isNull);
    });

    test('duplicate profile', () {
      final dup = controller.duplicateProfile('phone-sm', newName: 'My Phone');
      expect(dup, isNotNull);
      expect(dup!.name, 'My Phone');
      expect(controller.profiles.length, 7);
    });
  });
}
