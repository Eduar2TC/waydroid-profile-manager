import 'package:flutter_test/flutter_test.dart';
import 'package:waydroid_profile_manager/infrastructure/rust/mock_waydroid_service.dart';

void main() {
  group('MockWaydroidService', () {
    late MockWaydroidService service;

    setUp(() {
      service = MockWaydroidService();
      service.init();
    });

    test('initializes with 6 built-in profiles', () {
      final profiles = service.getProfiles();
      expect(profiles.length, 6);
    });

    test('all built-in profiles are marked as builtin', () {
      final profiles = service.getProfiles();
      for (final p in profiles) {
        expect(p.builtin, isTrue);
      }
    });

    test('status shows installed and running', () {
      final status = service.getStatus();
      expect(status.installed, isTrue);
      expect(status.isRunning, isTrue);
    });

    test('no active profile initially', () {
      expect(service.getActiveProfile(), isNull);
    });

    test('create custom profile', () {
      final success = service.createProfile(
        name: 'Pixel 7',
        description: 'Custom profile',
        baseWidth: 1080,
        baseHeight: 2400,
        density: 420,
        orientation: 'portrait',
      );
      expect(success, isTrue);
      expect(service.getProfiles().length, 7);
    });

    test('delete custom profile', () {
      service.createProfile(
        name: 'Delete Me',
        baseWidth: 800,
        baseHeight: 600,
        density: 160,
        orientation: 'portrait',
      );
      expect(service.getProfiles().length, 7);

      final customId = service.getProfiles().last.id;
      final success = service.deleteProfile(customId);
      expect(success, isTrue);
      expect(service.getProfiles().length, 6);
    });

    test('cannot delete built-in profile', () {
      final success = service.deleteProfile('phone-sm');
      expect(success, isFalse);
    });

    test('apply profile sets active', () {
      final success = service.applyProfile('phone-sm', 'portrait');
      expect(success, isTrue);
      expect(service.getActiveProfile()?.id, 'phone-sm');
    });

    test('reset clears active profile', () {
      service.applyProfile('phone-sm', 'portrait');
      expect(service.getActiveProfile(), isNotNull);

      final success = service.reset();
      expect(success, isTrue);
      expect(service.getActiveProfile(), isNull);
    });

    test('duplicate profile', () {
      final dup = service.duplicateProfile('phone-sm', newName: 'My Phone');
      expect(dup, isNotNull);
      expect(dup!.name, 'My Phone');
      expect(dup.builtin, isFalse);
      expect(service.getProfiles().length, 7);
    });

    test('builtin profile names', () {
      final names = service.getProfiles().map((p) => p.name).toList();
      expect(names, containsAll([
        'Phone Small',
        'Phone Medium',
        'Phone Large',
        'Tablet Small',
        'Tablet Medium',
        'Tablet Large',
      ]));
    });

    // --- Import/Export tests ---

    test('export returns empty array when no custom profiles', () {
      final json = service.exportProfiles();
      expect(json, '[]');
    });

    test('export includes only custom profiles', () {
      service.createProfile(
        name: 'Custom',
        baseWidth: 800,
        baseHeight: 600,
        density: 160,
        orientation: 'portrait',
      );
      final json = service.exportProfiles();
      expect(json, contains('Custom'));
      expect(json, isNot(contains('Phone Small')));
    });

    test('import adds profiles', () {
      final json = '[{"name": "Imported", "base_width": 1920, "base_height": 1080, "density": 240, "default_orientation": "landscape", "builtin": false}]';
      final count = service.importProfiles(json);
      expect(count, 1);
      expect(service.getProfiles().length, 7);
      expect(service.getProfiles().last.name, 'Imported');
    });

    test('import skips builtin profiles', () {
      final json = '[{"name": "Builtin", "base_width": 800, "base_height": 600, "density": 160, "default_orientation": "portrait", "builtin": true}]';
      final count = service.importProfiles(json);
      expect(count, 0);
      expect(service.getProfiles().length, 6);
    });

    test('import invalid json returns -1', () {
      final count = service.importProfiles('not valid json');
      expect(count, -1);
    });

    test('export then import roundtrip', () {
      service.createProfile(
        name: 'Roundtrip',
        baseWidth: 1080,
        baseHeight: 2400,
        density: 420,
        orientation: 'portrait',
      );
      final exported = service.exportProfiles();
      expect(exported, contains('Roundtrip'));

      // Import into a fresh service
      final freshService = MockWaydroidService();
      freshService.init();
      expect(freshService.getProfiles().length, 6);

      final count = freshService.importProfiles(exported);
      expect(count, 1);
      expect(freshService.getProfiles().length, 7);
      expect(freshService.getProfiles().last.name, 'Roundtrip');
    });
  });
}
