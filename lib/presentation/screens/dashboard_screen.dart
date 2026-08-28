import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../application/controllers/app_controller.dart';
import '../../domain/entities/profile.dart';
import '../../domain/enums/orientation.dart';
import '../dialogs/confirm_dialog.dart';
import '../widgets/orientation_selector.dart';
import '../widgets/profile_card.dart';
import '../widgets/status_indicator.dart';class DashboardScreen extends StatelessWidget {
  const DashboardScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Consumer<AppController>(
      builder: (context, controller, _) {
        return Stack(
          children: [
            _buildScaffold(context, controller),
            if (controller.isApplying)
              _ApplyProgressOverlay(controller: controller),
          ],
        );
      },
    );
  }

  Widget _buildScaffold(BuildContext context, AppController controller) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Waydroid Profile Manager'),
        actions: [IconButton(icon: const Icon(Icons.refresh), onPressed: controller.isApplying ? null : () => controller.refresh(), tooltip: 'Refresh')],
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
              if (controller.activeProfile != null) ProfileCard(profile: controller.activeProfile!, isActive: true) else _buildNoActiveProfile(context),
              const SizedBox(height: 24),

              // Orientation selector
              _buildSectionTitle('Orientation'),
              const SizedBox(height: 8),
              OrientationSelector(selected: controller.selectedOrientation, onChanged: controller.isApplying ? (_) {} : controller.setOrientation),
              const SizedBox(height: 24),

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
  }

  Widget _buildSectionTitle(String title) {
    return Text(
      title,
      style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w500, color: Colors.white70, letterSpacing: 0.5),
    );
  }

  Widget _buildNoActiveProfile(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          children: [
            Icon(Icons.phone_android, size: 48, color: Theme.of(context).colorScheme.primary.withValues(alpha: 0.5)),
            const SizedBox(height: 12),
            Text('No profile applied', style: TextStyle(fontSize: 16, color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.7))),
            const SizedBox(height: 4),
            Text('Select a profile and orientation below', style: TextStyle(fontSize: 13, color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.5))),
          ],
        ),
      ),
    );
  }

  Widget _buildApplyButton(BuildContext context, AppController controller) {
    return FilledButton.icon(
      onPressed: controller.isApplying ? null : () => _applyProfile(context, controller),
      icon: const Icon(Icons.play_arrow),
      label: Text(controller.isApplying ? 'Applying...' : 'Apply Profile', style: const TextStyle(fontSize: 16)),
      style: FilledButton.styleFrom(padding: const EdgeInsets.symmetric(vertical: 16), minimumSize: const Size(double.infinity, 56)),
    );
  }

  Widget _buildQuickProfiles(BuildContext context, AppController controller) {
    final profiles = controller.profiles.take(6).toList();
    if (profiles.isEmpty) return const SizedBox.shrink();

    return LayoutBuilder(
      builder: (context, constraints) {
        const spacing = 8.0;
        const minChipWidth = 110.0;
        final cols = (constraints.maxWidth / (minChipWidth + spacing)).floor().clamp(1, 3);

        final rows = <List<Profile>>[];
        for (var i = 0; i < profiles.length; i += cols) {
          rows.add(profiles.sublist(i, (i + cols).clamp(0, profiles.length)));
        }

        return Column(
          children: rows.map((row) {
            final placeholders = cols - row.length;
            return Padding(
              padding: const EdgeInsets.only(bottom: spacing),
              child: Row(
                children: [
                  ...row.map((profile) {
                    return Expanded(
                      child: SizedBox(
                        width: double.infinity,
                        child: ActionChip(
                          avatar: Icon(profile.builtin ? Icons.star : Icons.star_border, size: 16, color: profile.builtin ? Colors.amber : Colors.white54),
                          label: Text(profile.name, style: const TextStyle(fontSize: 12), overflow: TextOverflow.ellipsis, maxLines: 1),
                          onPressed: controller.isApplying
                              ? null
                              : () {
                                  controller.setOrientation(profile.defaultOrientation);
                                  controller.applyProfile(profile);
                                },
                        ),
                      ),
                    );
                  }),
                  for (var i = 0; i < placeholders; i++) const Expanded(child: SizedBox()),
                ],
              ),
            );
          }).toList(),
        );
      },
    );
  }

  Widget _buildResetButton(BuildContext context, AppController controller) {
    return OutlinedButton.icon(
      onPressed: controller.activeProfile == null ? null : () => _confirmReset(context, controller),
      icon: const Icon(Icons.restore, size: 18),
      label: const Text('Reset Configuration'),
      style: OutlinedButton.styleFrom(
        foregroundColor: Colors.orange,
        side: const BorderSide(color: Colors.orange),
      ),
    );
  }

  Future<void> _applyProfile(BuildContext context, AppController controller) async {
    final profiles = controller.profiles;
    if (profiles.isEmpty) return;

    await showDialog<Profile>(
      context: context,
      barrierDismissible: false,
      builder: (ctx) => _ProfileSelectionDialog(profiles: profiles, orientation: controller.selectedOrientation, controller: controller),
    );
  }

  Future<void> _confirmReset(BuildContext context, AppController controller) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (ctx) => ConfirmDialog(
        title: 'Reset Configuration',
        message:
            'This will restore the previous Waydroid configuration.\n\n'
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

class _ProfileSelectionDialog extends StatefulWidget {
  final List<Profile> profiles;
  final WaydroidOrientation orientation;
  final AppController controller;

  const _ProfileSelectionDialog({required this.profiles, required this.orientation, required this.controller});

  @override
  State<_ProfileSelectionDialog> createState() => _ProfileSelectionDialogState();
}

class _ProfileSelectionDialogState extends State<_ProfileSelectionDialog> {
  Profile? _applyingProfile;

  bool get _isBusy => _applyingProfile != null;

  Future<void> _onSelect(Profile profile) async {
    if (_isBusy) return;

    // Close dialog immediately — the dashboard overlay will show progress
    Navigator.of(context).pop(profile);

    // Start the apply (non-blocking, progress shown via overlay)
    await widget.controller.applyProfile(profile);
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('Select Profile'),
      content: SizedBox(
        width: 400,
        child: ListView.builder(
          shrinkWrap: true,
          itemCount: widget.profiles.length,
          itemBuilder: (context, index) {
            final profile = widget.profiles[index];
            final dims = profile.getActualDimensions(widget.orientation);
            final isApplyingThis = _applyingProfile == profile;

            return ListTile(
              enabled: !_isBusy,
              leading: isApplyingThis
                  ? const SizedBox(width: 24, height: 24, child: CircularProgressIndicator(strokeWidth: 2))
                  : Icon(profile.builtin ? Icons.star : Icons.phone_android, color: profile.builtin ? Colors.amber : null),
              title: Text(profile.name),
              subtitle: Text('${dims[0]}×${dims[1]} • ${profile.density} DPI • ${widget.orientation.displayName}'),
              onTap: _isBusy ? null : () => _onSelect(profile),
            );
          },
        ),
      ),
      actions: [TextButton(onPressed: _isBusy ? null : () => Navigator.of(context).pop(), child: const Text('Cancel'))],
    );
  }
}

/// Full-screen overlay shown while a profile is being applied.
/// Shows a spinner and the current step/message from the Rust backend.
class _ApplyProgressOverlay extends StatelessWidget {
  final AppController controller;

  const _ApplyProgressOverlay({required this.controller});

  @override
  Widget build(BuildContext context) {
    final progress = controller.applyProgress;
    final step = progress?.step ?? 'Starting...';
    final message = progress?.message ?? 'Preparing to apply profile';

    return Positioned.fill(
      child: Material(
        color: Colors.black54,
        child: Center(
          child: Container(
            constraints: const BoxConstraints(maxWidth: 360),
            padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 28),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surface,
              borderRadius: BorderRadius.circular(20),
            ),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                SizedBox(
                  width: 56,
                  height: 56,
                  child: CircularProgressIndicator(
                    strokeWidth: 4,
                    valueColor: AlwaysStoppedAnimation<Color>(
                      Theme.of(context).colorScheme.primary,
                    ),
                  ),
                ),
                const SizedBox(height: 24),
                Text(
                  step,
                  style: Theme.of(context).textTheme.titleMedium?.copyWith(
                        fontWeight: FontWeight.w600,
                      ),
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 8),
                Text(
                  message,
                  style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                        color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.7),
                      ),
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 20),
                Text(
                  'This may take up to 60 seconds\nwhile Waydroid restarts',
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.4),
                      ),
                  textAlign: TextAlign.center,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
