import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';

import 'package:waydroid_profile_manager/application/controllers/app_controller.dart';
import 'package:waydroid_profile_manager/infrastructure/rust/mock_waydroid_service.dart';

void main() {
  testWidgets('App smoke test', (WidgetTester tester) async {
    final controller = AppController(MockWaydroidService());
    controller.init();

    await tester.pumpWidget(
      ChangeNotifierProvider.value(
        value: controller,
        child: MaterialApp(
          title: 'Waydroid Profile Manager',
          theme: ThemeData(
            colorScheme: ColorScheme.fromSeed(
              seedColor: const Color(0xFF6750A4),
              brightness: Brightness.dark,
            ),
            useMaterial3: true,
          ),
          home: const Scaffold(
            body: Center(child: Text('Waydroid Profile Manager')),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    // Verify the app renders
    expect(find.text('Waydroid Profile Manager'), findsOneWidget);
  });
}
