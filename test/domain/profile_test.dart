import 'package:flutter_test/flutter_test.dart';
import 'package:waydroid_profile_manager/domain/entities/profile.dart';
import 'package:waydroid_profile_manager/domain/enums/orientation.dart';

void main() {
  group('Profile', () {
    test('fromJson roundtrip', () {
      final now = DateTime.now();
      final profile = Profile(
        id: 'test-1',
        name: 'Test Phone',
        description: 'A test profile',
        baseWidth: 1080,
        baseHeight: 2400,
        density: 420,
        defaultOrientation: WaydroidOrientation.portrait,
        createdAt: now,
        updatedAt: now,
        builtin: false,
      );

      final json = profile.toJson();
      final parsed = Profile.fromJson(json);

      expect(parsed.id, profile.id);
      expect(parsed.name, profile.name);
      expect(parsed.description, profile.description);
      expect(parsed.baseWidth, profile.baseWidth);
      expect(parsed.baseHeight, profile.baseHeight);
      expect(parsed.density, profile.density);
      expect(parsed.defaultOrientation, profile.defaultOrientation);
      expect(parsed.builtin, profile.builtin);
    });

    test('copyWith creates new instance', () {
      final profile = Profile(
        id: 'test-1',
        name: 'Original',
        baseWidth: 800,
        baseHeight: 600,
        density: 160,
        defaultOrientation: WaydroidOrientation.portrait,
        createdAt: DateTime.now(),
        updatedAt: DateTime.now(),
        builtin: false,
      );

      final updated = profile.copyWith(name: 'Updated', baseWidth: 1920);
      expect(updated.name, 'Updated');
      expect(updated.baseWidth, 1920);
      expect(updated.baseHeight, 600); // unchanged
      expect(profile.name, 'Original'); // original unchanged
    });

    test('getActualDimensions', () {
      final profile = Profile(
        id: 'test-1',
        name: 'Phone',
        baseWidth: 640,
        baseHeight: 360,
        density: 160,
        defaultOrientation: WaydroidOrientation.portrait,
        createdAt: DateTime.now(),
        updatedAt: DateTime.now(),
        builtin: false,
      );

      final landscape = profile.getActualDimensions(WaydroidOrientation.landscape);
      expect(landscape[0], 640);
      expect(landscape[1], 360);

      final portrait = profile.getActualDimensions(WaydroidOrientation.portrait);
      expect(portrait[0], 360);
      expect(portrait[1], 640);
    });

    test('equality', () {
      final now = DateTime.now();
      final p1 = Profile(
        id: 'test-1',
        name: 'Phone',
        baseWidth: 800,
        baseHeight: 600,
        density: 160,
        defaultOrientation: WaydroidOrientation.portrait,
        createdAt: now,
        updatedAt: now,
        builtin: false,
      );
      final p2 = Profile(
        id: 'test-1',
        name: 'Phone',
        baseWidth: 800,
        baseHeight: 600,
        density: 160,
        defaultOrientation: WaydroidOrientation.portrait,
        createdAt: now,
        updatedAt: now,
        builtin: false,
      );

      expect(p1, equals(p2));
      expect(p1.hashCode, equals(p2.hashCode));
    });

    test('inequality', () {
      final now = DateTime.now();
      final p1 = Profile(
        id: 'test-1',
        name: 'Phone',
        baseWidth: 800,
        baseHeight: 600,
        density: 160,
        defaultOrientation: WaydroidOrientation.portrait,
        createdAt: now,
        updatedAt: now,
        builtin: false,
      );
      final p2 = Profile(
        id: 'test-2',
        name: 'Tablet',
        baseWidth: 1200,
        baseHeight: 800,
        density: 160,
        defaultOrientation: WaydroidOrientation.landscape,
        createdAt: now,
        updatedAt: now,
        builtin: true,
      );

      expect(p1, isNot(equals(p2)));
    });
  });
}
