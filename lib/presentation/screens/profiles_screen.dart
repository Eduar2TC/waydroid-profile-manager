import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
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
                icon: const Icon(Icons.file_upload),
                onPressed: () => _importProfiles(context, controller),
                tooltip: 'Import Profiles',
              ),
              IconButton(
                icon: const Icon(Icons.file_download),
                onPressed: () => _exportProfiles(context, controller),
                tooltip: 'Export Profiles',
              ),
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

  void _exportProfiles(BuildContext context, AppController controller) {
    final json = controller.exportProfiles();
    // Show the JSON in a dialog with copy option
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Export Profiles'),
        content: SizedBox(
          width: 500,
          height: 400,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Text(
                'Copy this JSON and share it:',
                style: TextStyle(color: Colors.white.withValues(alpha: 0.7)),
              ),
              const SizedBox(height: 8),
              Expanded(
                child: Container(
                  padding: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: Colors.black.withValues(alpha: 0.3),
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(
                      color: Colors.white.withValues(alpha: 0.1),
                    ),
                  ),
                  child: SingleChildScrollView(
                    child: SelectableText(
                      json,
                      style: const TextStyle(
                        fontFamily: 'monospace',
                        fontSize: 12,
                      ),
                    ),
                  ),
                ),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('Close'),
          ),
          FilledButton.icon(
            onPressed: () {
              Clipboard.setData(ClipboardData(text: json));
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text('Copied to clipboard')),
              );
              Navigator.pop(ctx);
            },
            icon: const Icon(Icons.copy, size: 16),
            label: const Text('Copy'),
          ),
        ],
      ),
    );
  }

  void _importProfiles(BuildContext context, AppController controller) {
    final textController = TextEditingController();
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Import Profiles'),
        content: SizedBox(
          width: 500,
          height: 400,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Text(
                'Paste a JSON array of profiles:',
                style: TextStyle(color: Colors.white.withValues(alpha: 0.7)),
              ),
              const SizedBox(height: 8),
              Expanded(
                child: TextField(
                  controller: textController,
                  maxLines: null,
                  expands: true,
                  textAlignVertical: TextAlignVertical.top,
                  style: const TextStyle(
                    fontFamily: 'monospace',
                    fontSize: 12,
                  ),
                  decoration: InputDecoration(
                    hintText: '[{"name": "My Profile", ...}]',
                    border: const OutlineInputBorder(),
                    filled: true,
                    fillColor: Colors.black.withValues(alpha: 0.3),
                  ),
                ),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () {
              final json = textController.text.trim();
              if (json.isEmpty) return;
              final count = controller.importProfiles(json);
              Navigator.pop(ctx);
              if (count > 0) {
                ScaffoldMessenger.of(context).showSnackBar(
                  SnackBar(content: Text('Imported $count profile(s)')),
                );
              } else {
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(
                    content: Text('No profiles imported — check JSON format'),
                    backgroundColor: Colors.red,
                  ),
                );
              }
            },
            child: const Text('Import'),
          ),
        ],
      ),
    );
  }
}
