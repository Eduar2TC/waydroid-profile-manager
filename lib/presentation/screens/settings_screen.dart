import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:waydroid_profile_manager/application/controllers/app_controller.dart';


class SettingsScreen extends StatelessWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Consumer<AppController>(
      builder: (context, controller, _) {
        return Scaffold(
          appBar: AppBar(
            title: const Text('Settings'),
          ),
          body: ListView(
            padding: const EdgeInsets.all(16),
            children: [
              // Waydroid info
              _buildSectionHeader(context, 'Waydroid'),
              Card(
                child: Column(
                  children: [
                    ListTile(
                      leading: Icon(
                        Icons.check_circle,
                        color: controller.waydroidStatus.installed
                            ? Colors.green
                            : Colors.red,
                      ),
                      title: const Text('Waydroid Installation'),
                      subtitle: Text(
                        controller.waydroidStatus.installed
                            ? 'Detected'
                            : 'Not found',
                      ),
                    ),
                    const Divider(height: 1),
                    ListTile(
                      leading: Icon(
                        Icons.circle,
                        color: controller.waydroidStatus.isRunning
                            ? Colors.green
                            : Colors.orange,
                        size: 12,
                      ),
                      title: const Text('Status'),
                      subtitle: Text(controller.waydroidStatus.summary),
                    ),
                    const Divider(height: 1),
                    ListTile(
                      leading: const Icon(Icons.computer),
                      title: const Text('Display Server'),
                      subtitle: const Text('Wayland / X11'),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 24),

              // App info
              _buildSectionHeader(context, 'Application'),
              Card(
                child: Column(
                  children: [
                    const ListTile(
                      leading: Icon(Icons.info_outline),
                      title: Text('Version'),
                      subtitle: Text('1.0.0 (MVP)'),
                    ),
                    const Divider(height: 1),
                    const ListTile(
                      leading: Icon(Icons.code),
                      title: Text('Backend'),
                      subtitle: Text('Rust + Flutter'),
                    ),
                    const Divider(height: 1),
                    const ListTile(
                      leading: Icon(Icons.storage),
                      title: Text('Profiles Location'),
                      subtitle: Text('~/.config/waydroid-profile-manager/'),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 24),

              // Advanced section
              _buildSectionHeader(context, 'Advanced'),
              Card(
                child: Column(
                  children: [
                    ListTile(
                      leading: const Icon(Icons.warning_amber, color: Colors.orange),
                      title: const Text('Properties Used'),
                      subtitle: const Text('persist.waydroid.width/height/density'),
                      isThreeLine: true,
                    ),
                    const Divider(height: 1),
                    const ListTile(
                      leading: Icon(Icons.block, color: Colors.red),
                      title: Text('Avoided Commands'),
                      subtitle: Text('wm size, wm density'),
                      isThreeLine: true,
                    ),
                  ],
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildSectionHeader(BuildContext context, String title) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Text(
        title.toUpperCase(),
        style: TextStyle(
          fontSize: 12,
          fontWeight: FontWeight.w600,
          color: Theme.of(context).colorScheme.primary,
          letterSpacing: 1.2,
        ),
      ),
    );
  }
}
