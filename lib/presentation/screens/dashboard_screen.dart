import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../application/controllers/app_controller.dart';
import '../../domain/entities/profile.dart';
import '../../domain/enums/orientation.dart';
import '../widgets/profile_card.dart';
import '../widgets/status_indicator.dart';
import '../widgets/orientation_selector.dart';
import '../dialogs/confirm_dialog.dart';

class DashboardScreen extends StatelessWidget {
  const DashboardScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Consumer<AppController>(
      builder: (context, controller, _) {
        return Scaffold(
          appBar: AppBar(
            title: const Text('Waydroid Profile Manager'),
            actions: [
              IconButton(
                icon: const Icon(Icons.refresh),
                onPressed: () => controller.refresh(),
                tooltip: 'Refresh',
              ),
            ],
          ),
          body: RefreshIndicator(
            onRefresh: () => controller.refresh(),
            child: SingleChildScrollView(
              physics: const AlwaysScrollableScrollPhysics(),
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  // Status indicator
                  StatusIndicator(status: controller.waydroidStatus),
                  const SizedBox(height: 24),

                  // Active profile section
                  _buildSectionTitle('Active Profile'),
                  const SizedBox(height: 8),
                  if (controller.activeProfile != null)
                    ProfileCard(
                      profile: controller.activeProfile!,
                      isActive: true,
                    )
                  else
                    _buildNoActiveProfile(context),
                  const SizedBox(height: 24),

                  // Orientation selector
                  _buildSectionTitle('Orientation'),
                  const SizedBox(height: 8),
                  OrientationSelector(
                    selected: controller.selectedOrientation,
                    onChanged: controller.setOrientation,
                  ),
                  const SizedBox(height: 24),

                  // Restart banner (shown after apply when resolution changed)
                  if (controller.needsRestart) ...[
                    _buildRestartBanner(context, controller),
                    const SizedBox(height: 16),
                  ],

                  // Apply button
                  _buildApplyButton(context, controller),
                  const SizedBox(height: 16),

                  // Quick profile selection
                  _buildSectionTitle('Quick Select'),
                  const SizedBox(height: 8),
                  _buildQuickProfiles(context, controller),
                  const SizedBox(height: 16),

                  // Reset button
                  _buildResetButton(context, controller),
                ],
              ),
            ),
          ),
        );
      },
    );
  }

  Widget _buildSectionTitle(String title) {
    return Text(
      title,
      style: const TextStyle(
        fontSize: 14,
        fontWeight: FontWeight.w500,
        color: Colors.white70,
        letterSpacing: 0.5,
      ),
    );
  }

  Widget _buildNoActiveProfile(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          children: [
            Icon(
              Icons.phone_android,
              size: 48,
              color: Theme.of(context).colorScheme.primary.withValues(alpha: 0.5),
            ),
            const SizedBox(height: 12),
            Text(
              'No profile applied',
              style: TextStyle(
                fontSize: 16,
                color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.7),
              ),
            ),
            const SizedBox(height: 4),
            Text(
              'Select a profile and orientation below',
              style: TextStyle(
                fontSize: 13,
                color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.5),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildRestartBanner(BuildContext context, AppController controller) {
    return Card(
      color: Colors.orange.withValues(alpha: 0.15),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Row(
              children: [
                const Icon(Icons.info_outline, color: Colors.orange, size: 20),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    'Resolution changes require a session restart to take effect.',
                    style: TextStyle(
                      color: Colors.orange.shade200,
                      fontSize: 13,
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            FilledButton.icon(
              onPressed: controller.isApplying
                  ? null
                  : () => _confirmRestart(context, controller),
              icon: const Icon(Icons.restart_alt, size: 18),
              label: const Text('Restart Session'),
              style: FilledButton.styleFrom(
                backgroundColor: Colors.orange,
                foregroundColor: Colors.white,
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildApplyButton(BuildContext context, AppController controller) {
    return FilledButton.icon(
      onPressed: controller.isApplying ? null : () => _applyProfile(context, controller),
      icon: controller.isApplying
          ? const SizedBox(
              width: 20,
              height: 20,
              child: CircularProgressIndicator(strokeWidth: 2),
            )
          : const Icon(Icons.play_arrow),
      label: Text(
        controller.isApplying ? 'Applying...' : 'Apply Profile',
        style: const TextStyle(fontSize: 16),
      ),
      style: FilledButton.styleFrom(
        padding: const EdgeInsets.symmetric(vertical: 16),
        minimumSize: const Size(double.infinity, 56),
      ),
    );
  }

  Widget _buildQuickProfiles(BuildContext context, AppController controller) {
    return Wrap(
      spacing: 8,
      runSpacing: 8,
      children: controller.profiles.take(6).map((profile) {
        return ActionChip(
          avatar: Icon(
            profile.builtin ? Icons.star : Icons.star_border,
            size: 16,
            color: profile.builtin ? Colors.amber : Colors.white54,
          ),
          label: Text(
            profile.name,
            style: const TextStyle(fontSize: 12),
          ),
          onPressed: controller.isApplying
              ? null
              : () {
                  controller.setOrientation(profile.defaultOrientation);
                  controller.applyProfile(profile);
                },
        );
      }).toList(),
    );
  }

  Widget _buildResetButton(BuildContext context, AppController controller) {
    return OutlinedButton.icon(
      onPressed: controller.activeProfile == null
          ? null
          : () => _confirmReset(context, controller),
      icon: const Icon(Icons.restore, size: 18),
      label: const Text('Reset Configuration'),
      style: OutlinedButton.styleFrom(
        foregroundColor: Colors.orange,
        side: const BorderSide(color: Colors.orange),
      ),
    );
  }

  Future<void> _applyProfile(BuildContext context, AppController controller) async {
    // For quick apply, find first profile or prompt selection
    final profiles = controller.profiles;
    if (profiles.isEmpty) return;

    // Show profile selection dialog
    final selected = await showDialog<Profile>(
      context: context,
      builder: (ctx) => _ProfileSelectionDialog(
        profiles: profiles,
        orientation: controller.selectedOrientation,
      ),
    );

    if (selected != null && context.mounted) {
      await controller.applyProfile(selected);
    }
  }

  Future<void> _confirmRestart(BuildContext context, AppController controller) async {
    await showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Restart Waydroid'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'Run this command in a terminal to apply the new resolution:',
            ),
            const SizedBox(height: 12),
            Container(
              width: double.infinity,
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Colors.black.withValues(alpha: 0.3),
                borderRadius: BorderRadius.circular(8),
                border: Border.all(
                  color: Theme.of(context).colorScheme.primary.withValues(alpha: 0.5),
                ),
              ),
              child: const SelectableText(
                'waydroid session stop && waydroid session start',
                style: TextStyle(
                  fontFamily: 'monospace',
                  fontSize: 13,
                  color: Colors.white,
                ),
              ),
            ),
            const SizedBox(height: 12),
            Text(
              'This closes all Android apps. After restarting, '
              'click Refresh to update the status.',
              style: TextStyle(
                fontSize: 12,
                color: Colors.white.withValues(alpha: 0.6),
              ),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('OK'),
          ),
        ],
      ),
    );
  }

  Future<void> _confirmReset(BuildContext context, AppController controller) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (ctx) => ConfirmDialog(
        title: 'Reset Configuration',
        message: 'This will restore the previous Waydroid configuration.\n\n'
            'All changes made by Waydroid Profile Manager will be undone.',
        confirmLabel: 'Reset',
        confirmColor: Colors.orange,
      ),
    );

    if (confirmed == true && context.mounted) {
      await controller.reset();
    }
  }
}

class _ProfileSelectionDialog extends StatelessWidget {
  final List<Profile> profiles;
  final WaydroidOrientation orientation;

  const _ProfileSelectionDialog({
    required this.profiles,
    required this.orientation,
  });

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('Select Profile'),
      content: SizedBox(
        width: 400,
        child: ListView.builder(
          shrinkWrap: true,
          itemCount: profiles.length,
          itemBuilder: (context, index) {
            final profile = profiles[index];
            final dims = profile.getActualDimensions(orientation);
            return ListTile(
              leading: Icon(
                profile.builtin ? Icons.star : Icons.phone_android,
                color: profile.builtin ? Colors.amber : null,
              ),
              title: Text(profile.name),
              subtitle: Text(
                '${dims[0]}×${dims[1]} • ${profile.density} DPI • ${orientation.displayName}',
              ),
              onTap: () => Navigator.of(context).pop(profile),
            );
          },
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
      ],
    );
  }
}
