import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';

import 'package:waydroid_profile_manager/application/controllers/app_controller.dart';
import 'package:waydroid_profile_manager/infrastructure/rust/mock_waydroid_service.dart';
import 'package:waydroid_profile_manager/main.dart';

void main() {
  testWidgets('App smoke test', (WidgetTester tester) async {
    await tester.pumpWidget(const WaydroidProfileManagerApp());
    await tester.pumpAndSettle();

    // Verify the app title is shown
    expect(find.text('Waydroid Profile Manager'), findsOneWidget);

    // Verify dashboard is shown
    expect(find.text('Dashboard'), findsOneWidget);
    expect(find.text('Profiles'), findsOneWidget);
    expect(find.text('Settings'), findsOneWidget);
    expect(find.text('Logs'), findsOneWidget);
  });
}
