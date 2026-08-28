import 'package:flutter/material.dart';

import 'dashboard_screen.dart';
import 'logs_screen.dart';
import 'profiles_screen.dart';
import 'settings_screen.dart';

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  static const double _windowCornerRadius = 12.0;

  int _currentIndex = 0;
  final _screens = const [DashboardScreen(), ProfilesScreen(), SettingsScreen(), LogsScreen()];

  @override
  Widget build(BuildContext context) {
    // Clip the ENTIRE Scaffold as one unit. Clipping body and
    // bottomNavigationBar separately leaves a gap: Scaffold paints its own
    // backgroundColor as a full square rect behind both of them, so the
    // corner "cut out" by two separate ClipRRects just exposes that square
    // background instead of true transparency.
    return ClipRRect(
      borderRadius: const BorderRadius.only(bottomLeft: Radius.circular(_windowCornerRadius), bottomRight: Radius.circular(_windowCornerRadius)),
      child: Scaffold(
        body: _screens[_currentIndex],
        bottomNavigationBar: NavigationBar(
          selectedIndex: _currentIndex,
          onDestinationSelected: (index) {
            setState(() => _currentIndex = index);
          },
          destinations: const [
            NavigationDestination(
              icon: Icon(Icons.dashboard),
              selectedIcon: Icon(Icons.dashboard, color: Colors.white),
              label: 'Dashboard',
            ),
            NavigationDestination(
              icon: Icon(Icons.phone_android),
              selectedIcon: Icon(Icons.phone_android, color: Colors.white),
              label: 'Profiles',
            ),
            NavigationDestination(
              icon: Icon(Icons.settings),
              selectedIcon: Icon(Icons.settings, color: Colors.white),
              label: 'Settings',
            ),
            NavigationDestination(
              icon: Icon(Icons.terminal),
              selectedIcon: Icon(Icons.terminal, color: Colors.white),
              label: 'Logs',
            ),
          ],
        ),
      ),
    );
  }
}
