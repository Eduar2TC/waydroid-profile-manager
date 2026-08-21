import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../application/controllers/app_controller.dart';
import '../../domain/entities/profile.dart';
import '../../domain/enums/orientation.dart';
import '../dialogs/confirm_dialog.dart';
import '../dialogs/profile_editor_dialog.dart';
import '../widgets/profile_card.dart';

class ProfilesScreen extends StatelessWidget {
  const ProfilesScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Consumer<AppController>(
      builder: (context, controller, _) {
        return Scaffold(
          appBar: AppBar(
            title: const Text('Profiles'),
            actions: [
              IconButton(
                icon: const Icon(Icons.add),
                onPressed: () => _createProfile(context, controller),
                tooltip: 'Create Profile',
              ),
            ],
          ),
          body: controller.profiles.isEmpty
              ? _buildEmptyState(context)
              : _buildProfileList(context, controller),
        );
      },
    );
  }

  Widget _buildEmptyState(BuildContext context) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(
            Icons.phone_android,
            size: 64,
            color: Theme.of(context).colorScheme.primary.withValues(alpha: 0.5),
          ),
          const SizedBox(height: 16),
          Text(
            'No profiles',
            style: Theme.of(context).textTheme.headlineSmall,
          ),
          const SizedBox(height: 8),
          Text(
            'Create a new profile to get started',
            style: TextStyle(color: Colors.white54),
          ),
          const SizedBox(height: 24),
          FilledButton.icon(
            onPressed: () => _createProfile(context, context.read<AppController>()),
            icon: const Icon(Icons.add),
            label: const Text('Create Profile'),
          ),
        ],
      ),
    );
  }

  Widget _buildProfileList(BuildContext context, AppController controller) {
    return ListView.builder(
      padding: const EdgeInsets.all(8),
      itemCount: controller.profiles.length,
      itemBuilder: (context, index) {
        final profile = controller.profiles[index];
        final isActive = controller.activeProfile?.id == profile.id;
        return ProfileCard(
          profile: profile,
          isActive: isActive,
          onTap: () => _showProfileActions(context, controller, profile),
        );
      },
    );
  }

  Future<void> _createProfile(BuildContext context, AppController controller) async {
    final result = await showDialog<Profile>(
      context: context,
      builder: (ctx) => const ProfileEditorDialog(),
    );

    if (result != null && context.mounted) {
      controller.createProfile(
        name: result.name,
        description: result.description,
        baseWidth: result.baseWidth,
        baseHeight: result.baseHeight,
        density: result.density,
        orientation: result.defaultOrientation,
      );
    }
  }

  void _showProfileActions(
    BuildContext context,
    AppController controller,
    Profile profile,
  ) {
    showModalBottomSheet(
      context: context,
      builder: (ctx) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(
              leading: const Icon(Icons.play_arrow),
              title: const Text('Apply Profile'),
              onTap: () {
                Navigator.pop(ctx);
                controller.setOrientation(profile.defaultOrientation);
                controller.applyProfile(profile);
              },
            ),
            ListTile(
              leading: const Icon(Icons.copy),
              title: const Text('Duplicate'),
              onTap: () {
                Navigator.pop(ctx);
                controller.duplicateProfile(profile.id);
              },
            ),
            if (!profile.builtin)
              ListTile(
                leading: const Icon(Icons.edit),
                title: const Text('Edit'),
                onTap: () {
                  Navigator.pop(ctx);
                  _editProfile(context, controller, profile);
                },
              ),
            if (!profile.builtin)
              ListTile(
                leading: const Icon(Icons.delete, color: Colors.red),
                title: const Text('Delete', style: TextStyle(color: Colors.red)),
                onTap: () async {
                  Navigator.pop(ctx);
                  final confirmed = await showDialog<bool>(
                    context: context,
                    builder: (ctx) => ConfirmDialog(
                      title: 'Delete Profile',
                      message: 'Are you sure you want to delete "${profile.name}"?',
                      confirmLabel: 'Delete',
                      confirmColor: Colors.red,
                    ),
                  );
                  if (confirmed == true && context.mounted) {
                    controller.deleteProfile(profile.id);
                  }
                },
              ),
          ],
        ),
      ),
    );
  }

  void _editProfile(
    BuildContext context,
    AppController controller,
    Profile profile,
  ) async {
    final result = await showDialog<Profile>(
      context: context,
      builder: (ctx) => ProfileEditorDialog(profile: profile),
    );

    if (result != null && context.mounted) {
      controller.updateProfile(
        id: profile.id,
        name: result.name,
        description: result.description,
        baseWidth: result.baseWidth,
        baseHeight: result.baseHeight,
        density: result.density,
        orientation: result.defaultOrientation,
      );
    }
  }
}
