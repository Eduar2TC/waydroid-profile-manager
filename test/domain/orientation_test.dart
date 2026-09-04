import 'package:flutter_test/flutter_test.dart';
import 'package:waydroid_profile_manager/domain/enums/orientation.dart';

void main() {
  group('WaydroidOrientation', () {
    test('landscape orientation keeps dimensions and rotation=0', () {
      final dims = WaydroidOrientation.landscape.toActual(800, 450);
      expect(dims[0], 800);
      expect(dims[1], 450);
      expect(dims[2], 0); // rotation=0 since wm size matches window
    });

    test('portrait orientation swaps dimensions correctly', () {
      final dims = WaydroidOrientation.portrait.toActual(800, 450);
      expect(dims[0], 450);
      expect(dims[1], 800);
      expect(dims[2], 0); // no rotation for portrait
    });

    test('wm size matches orientation', () {
      // Landscape wm size matches host window dimensions
      final wm1 = WaydroidOrientation.landscape.toWmSize(640, 360);
      expect(wm1[0], 640);
      expect(wm1[1], 360);

      // Portrait wm size matches host window dimensions
      final wm2 = WaydroidOrientation.portrait.toWmSize(640, 360);
      expect(wm2[0], 360);
      expect(wm2[1], 640);

      // Reversed base (e.g. Pixel 7: 1080x2400)
      final wm4 = WaydroidOrientation.landscape.toWmSize(1080, 2400);
      expect(wm4[0], 1080);
      expect(wm4[1], 2400);

      final wm5 = WaydroidOrientation.portrait.toWmSize(1080, 2400);
      expect(wm5[0], 2400);
      expect(wm5[1], 1080);

      // Square stays the same
      final wm3 = WaydroidOrientation.landscape.toWmSize(800, 800);
      expect(wm3[0], 800);
      expect(wm3[1], 800);
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
      expect(dims[2], 0); // rotation=0 since wm size matches window
    });
  });
}
