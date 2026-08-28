import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../application/controllers/app_controller.dart';
import '../../domain/entities/profile.dart';
import '../dialogs/confirm_dialog.dart';
import '../widgets/orientation_selector.dart';
import '../widgets/profile_card.dart';
import '../widgets/status_indicator.dart';

class DashboardScreen extends StatefulWidget {
  const DashboardScreen({super.key});

  @override
  State<DashboardScreen> createState() => _DashboardScreenState();
}

class _DashboardScreenState extends State<DashboardScreen> {
  Profile? _selectedProfile;

  @override
  Widget build(BuildContext context) {
    return Consumer<AppController>(
      builder: (context, controller, _) {
        if (_selectedProfile == null && controller.profiles.isNotEmpty) {
          _selectedProfile = controller.profiles.first;
        }
        return Stack(
          children: [
            _buildScaffold(context, controller),
            if (controller.isApplying) _ApplyProgressOverlay(controller: controller),
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

              // Profile selector + Apply
              _buildSectionTitle('Select Profile'),
              const SizedBox(height: 8),
              _buildProfileDropdown(context, controller),
              const SizedBox(height: 12),
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

  Widget _buildProfileDropdown(BuildContext context, AppController controller) {
    return DropdownMenu<Profile>(
      expandedInsets: EdgeInsets.zero,
      initialSelection: _selectedProfile,
      onSelected: (profile) => setState(() => _selectedProfile = profile),
      enabled: !controller.isApplying,
      dropdownMenuEntries: controller.profiles.map((p) {
        return DropdownMenuEntry<Profile>(
          value: p,
          label: p.name,
          leadingIcon: Icon(p.builtin ? Icons.star : Icons.star_border, size: 18, color: p.builtin ? Colors.amber : Colors.white54),
        );
      }).toList(),
    );
  }

  Widget _buildApplyButton(BuildContext context, AppController controller) {
    return FilledButton.icon(
      onPressed: controller.isApplying || _selectedProfile == null
          ? null
          : () => controller.applyProfile(_selectedProfile!),
      icon: const Icon(Icons.play_arrow),
      label: Text(controller.isApplying ? 'Applying...' : 'Apply', style: const TextStyle(fontSize: 16)),
      style: FilledButton.styleFrom(padding: const EdgeInsets.symmetric(vertical: 16), minimumSize: const Size(double.infinity, 56)),
    );
  }

  Widget _buildQuickProfiles(BuildContext context, AppController controller) {
    return Wrap(
      spacing: 8,
      runSpacing: 8,
      children: controller.profiles.take(6).map((profile) {
        return ActionChip(
          avatar: Icon(profile.builtin ? Icons.star : Icons.star_border, size: 16, color: profile.builtin ? Colors.amber : Colors.white54),
          label: Text(profile.name, style: const TextStyle(fontSize: 12)),
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
      onPressed: controller.activeProfile == null ? null : () => _confirmReset(context, controller),
      icon: const Icon(Icons.restore, size: 18),
      label: const Text('Reset Configuration'),
      style: OutlinedButton.styleFrom(
        foregroundColor: Colors.orange,
        side: const BorderSide(color: Colors.orange),
      ),
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
            decoration: BoxDecoration(color: Theme.of(context).colorScheme.surface, borderRadius: BorderRadius.circular(20)),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                SizedBox(width: 56, height: 56, child: CircularProgressIndicator(strokeWidth: 4, valueColor: AlwaysStoppedAnimation<Color>(Theme.of(context).colorScheme.primary))),
                const SizedBox(height: 24),
                Text(
                  step,
                  style: Theme.of(context).textTheme.titleMedium?.copyWith(fontWeight: FontWeight.w600),
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 8),
                Text(
                  message,
                  style: Theme.of(context).textTheme.bodyMedium?.copyWith(color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.7)),
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 20),
                Text(
                  'This may take up to 60 seconds\nwhile Waydroid restarts',
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.4)),
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
