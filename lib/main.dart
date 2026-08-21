import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'application/controllers/app_controller.dart';
import 'infrastructure/rust/ffi_waydroid_service.dart';
import 'infrastructure/rust/mock_waydroid_service.dart';
import 'infrastructure/rust/waydroid_service.dart';
import 'presentation/screens/home_screen.dart';
import 'presentation/tray/system_tray_manager.dart';

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

class WaydroidProfileManagerApp extends StatefulWidget {
  const WaydroidProfileManagerApp({super.key});

  @override
  State<WaydroidProfileManagerApp> createState() =>
      _WaydroidProfileManagerAppState();
}

class _WaydroidProfileManagerAppState
    extends State<WaydroidProfileManagerApp> {
  late final AppController _controller;
  final AppSystemTray _tray = AppSystemTray();

  @override
  void initState() {
    super.initState();
    _controller = AppController(_createService());
    _controller.init();
    _initTray();
  }

  Future<void> _initTray() async {
    // Wait for controller to load profiles
    await Future.delayed(const Duration(milliseconds: 300));
    if (!mounted) return;

    await _tray.init(
      profiles: _controller.profiles,
      activeProfile: _controller.activeProfile,
      onProfileSelected: (profile, orientation) {
        _controller.setOrientation(orientation);
        _controller.applyProfile(profile);
      },
      onShowWindow: () {
        // Window management placeholder
      },
      onQuit: () {
        _tray.dispose();
      },
    );

    // Listen for profile changes and update tray
    _controller.addListener(() {
      if (mounted) {
        _tray.updateMenu(
          profiles: _controller.profiles,
          activeProfile: _controller.activeProfile,
        );
      }
    });
  }

  @override
  void dispose() {
    _tray.dispose();
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return ChangeNotifierProvider.value(
      value: _controller,
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
