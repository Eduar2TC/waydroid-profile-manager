import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'application/controllers/app_controller.dart';
import 'infrastructure/rust/ffi_waydroid_service.dart';
import 'infrastructure/rust/mock_waydroid_service.dart';
import 'infrastructure/rust/waydroid_service.dart';
import 'presentation/screens/home_screen.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const WaydroidProfileManagerApp());
}

/// Try to create the Rust FFI service; fall back to mock if unavailable.
WaydroidService _createService() {
  try {
    final ffiService = FfiWaydroidService();
    if (ffiService.init()) {
      debugPrint('✅ Rust backend loaded successfully');
      return ffiService;
    }
    debugPrint('⚠️  Rust backend init failed, using mock');
  } catch (e) {
    debugPrint('⚠️  Rust backend not available ($e), using mock');
  }
  debugPrint('ℹ️  Using mock Waydroid service');
  return MockWaydroidService();
}

class WaydroidProfileManagerApp extends StatelessWidget {
  const WaydroidProfileManagerApp({super.key});

  @override
  Widget build(BuildContext context) {
    return ChangeNotifierProvider(
      create: (_) {
        final controller = AppController(_createService());
        controller.init();
        return controller;
      },
      child: MaterialApp(
        title: 'Waydroid Profile Manager',
        debugShowCheckedModeBanner: false,
        theme: ThemeData(
          colorScheme: ColorScheme.fromSeed(
            seedColor: const Color(0xFF6750A4),
            brightness: Brightness.dark,
          ),
          useMaterial3: true,
          fontFamily: 'Roboto',
        ),
        home: const HomeScreen(),
      ),
    );
  }
}
