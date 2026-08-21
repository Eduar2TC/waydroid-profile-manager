import 'package:flutter_test/flutter_test.dart';
import 'package:waydroid_profile_manager/domain/enums/orientation.dart';

void main() {
  group('WaydroidOrientation', () {
    test('landscape orientation swaps dimensions correctly', () {
      final dims = WaydroidOrientation.landscape.toActual(800, 450);
      expect(dims[0], 800);
      expect(dims[1], 450);
      expect(dims[2], 1); // rotation
    });

    test('portrait orientation swaps dimensions correctly', () {
      final dims = WaydroidOrientation.portrait.toActual(800, 450);
      expect(dims[0], 450);
      expect(dims[1], 800);
      expect(dims[2], 0); // rotation
    });

    test('display name', () {
      expect(WaydroidOrientation.landscape.displayName, 'Landscape');
      expect(WaydroidOrientation.portrait.displayName, 'Portrait');
    });

    test('api key', () {
      expect(WaydroidOrientation.landscape.apiKey, 'landscape');
      expect(WaydroidOrientation.portrait.apiKey, 'portrait');
    });

    test('fromString', () {
      expect(WaydroidOrientation.fromString('landscape'), WaydroidOrientation.landscape);
      expect(WaydroidOrientation.fromString('portrait'), WaydroidOrientation.portrait);
      expect(WaydroidOrientation.fromString('invalid'), WaydroidOrientation.portrait);
    });

    test('portrait phone small', () {
      final dims = WaydroidOrientation.portrait.toActual(640, 360);
      expect(dims[0], 360);
      expect(dims[1], 640);
      expect(dims[2], 0);
    });

    test('landscape tablet medium', () {
      final dims = WaydroidOrientation.landscape.toActual(1280, 800);
      expect(dims[0], 1280);
      expect(dims[1], 800);
      expect(dims[2], 1);
    });
  });
}
