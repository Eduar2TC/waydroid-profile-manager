import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:system_tray/system_tray.dart';

import '../../domain/entities/profile.dart';
import '../../domain/enums/orientation.dart';

/// Callback type for when a profile is selected from the tray.
typedef OnProfileSelected = void Function(
    Profile profile, WaydroidOrientation orientation);

/// Manages the system tray icon and menu for quick profile switching.
class AppSystemTray {
  final SystemTray _tray = SystemTray();
  bool _initialized = false;
  OnProfileSelected? _onProfileSelected;
  VoidCallback? _onShowWindow;
  VoidCallback? _onQuit;

  bool get isInitialized => _initialized;

  /// Initialize the system tray.
  Future<void> init({
    required List<Profile> profiles,
    required Profile? activeProfile,
    OnProfileSelected? onProfileSelected,
    VoidCallback? onShowWindow,
    VoidCallback? onQuit,
  }) async {
    _onProfileSelected = onProfileSelected;
    _onShowWindow = onShowWindow;
    _onQuit = onQuit;

    try {
      final iconPath = _findIcon();
      if (iconPath.isEmpty) {
        debugPrint('⚠️  Tray icon not found, skipping system tray');
        return;
      }

      await _tray.initSystemTray(
        title: 'Waydroid Profile Manager',
        iconPath: iconPath,
        toolTip: 'Waydroid Profile Manager',
      );

      await _buildMenu(profiles, activeProfile);

      _tray.registerSystemTrayEventHandler((event) {
        if (event == kSystemTrayEventClick) {
          _onShowWindow?.call();
        } else if (event == kSystemTrayEventRightClick) {
          _tray.popUpContextMenu();
        }
      });

      _initialized = true;
      debugPrint('✅ System tray initialized');
    } catch (e) {
      debugPrint('⚠️  System tray init failed: $e');
      _initialized = false;
    }
  }

  /// Update the tray menu with current profiles.
  Future<void> updateMenu({
    required List<Profile> profiles,
    required Profile? activeProfile,
  }) async {
    if (!_initialized) return;
    await _buildMenu(profiles, activeProfile);
  }

  /// Build the tray context menu.
  Future<void> _buildMenu(
      List<Profile> profiles, Profile? activeProfile) async {
    final items = <MenuItemBase>[
      MenuItemLabel(label: 'Waydroid Profile Manager'),
      MenuSeparator(),
    ];

    // Add profiles as submenus
    for (final profile in profiles) {
      final isActive = activeProfile?.id == profile.id;
      final prefix = isActive ? '● ' : '  ';

      final children = <MenuItemBase>[
        MenuItemLabel(
          label:
              '${profile.name} — ${profile.baseWidth}×${profile.baseHeight} • ${profile.density} DPI',
          enabled: false,
        ),
        MenuSeparator(),
        MenuItemLabel(
          label: '${prefix}Portrait',
          onClicked: (_) {
            _onProfileSelected?.call(profile, WaydroidOrientation.portrait);
          },
        ),
        MenuItemLabel(
          label: '${prefix}Landscape',
          onClicked: (_) {
            _onProfileSelected?.call(profile, WaydroidOrientation.landscape);
          },
        ),
      ];

      items.add(SubMenu(label: profile.name, children: children));
    }

    // Bottom items
    items.add(MenuSeparator());
    items.add(MenuItemLabel(
      label: 'Show Window',
      onClicked: (_) => _onShowWindow?.call(),
    ));
    items.add(MenuItemLabel(
      label: 'Quit',
      onClicked: (_) => _onQuit?.call(),
    ));

    final menu = Menu();
    await menu.buildFrom(items);
    await _tray.setContextMenu(menu);
  }

  /// Find the tray icon file.
  String _findIcon() {
    final candidates = [
      'assets/tray_icon.png',
      'build/linux/x64/release/bundle/assets/tray_icon.png',
      'build/linux/x64/debug/bundle/assets/tray_icon.png',
    ];
    for (final path in candidates) {
      if (File(path).existsSync()) {
        return path;
      }
    }
    return '';
  }

  /// Dispose the tray.
  void dispose() {
    _tray.destroy();
    _initialized = false;
  }
}
