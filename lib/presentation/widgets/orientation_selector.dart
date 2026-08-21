import 'package:flutter/material.dart';

import '../../domain/enums/orientation.dart';

class OrientationSelector extends StatelessWidget {
  final WaydroidOrientation selected;
  final ValueChanged<WaydroidOrientation> onChanged;

  const OrientationSelector({
    super.key,
    required this.selected,
    required this.onChanged,
  });

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Expanded(
          child: _OrientationButton(
            icon: Icons.screen_rotation,
            label: 'Landscape',
            isSelected: selected == WaydroidOrientation.landscape,
            onTap: () => onChanged(WaydroidOrientation.landscape),
          ),
        ),
        const SizedBox(width: 12),
        Expanded(
          child: _OrientationButton(
            icon: Icons.screen_lock_rotation,
            label: 'Portrait',
            isSelected: selected == WaydroidOrientation.portrait,
            onTap: () => onChanged(WaydroidOrientation.portrait),
          ),
        ),
      ],
    );
  }
}

class _OrientationButton extends StatelessWidget {
  final IconData icon;
  final String label;
  final bool isSelected;
  final VoidCallback onTap;

  const _OrientationButton({
    required this.icon,
    required this.label,
    required this.isSelected,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(12),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 200),
        padding: const EdgeInsets.symmetric(vertical: 16),
        decoration: BoxDecoration(
          color: isSelected
              ? Theme.of(context).colorScheme.primary.withValues(alpha: 0.2)
              : Colors.white.withValues(alpha: 0.05),
          borderRadius: BorderRadius.circular(12),
          border: Border.all(
            color: isSelected
                ? Theme.of(context).colorScheme.primary
                : Colors.white.withValues(alpha: 0.1),
            width: isSelected ? 2 : 1,
          ),
        ),
        child: Column(
          children: [
            Icon(
              icon,
              size: 32,
              color: isSelected
                  ? Theme.of(context).colorScheme.primary
                  : Colors.white54,
            ),
            const SizedBox(height: 8),
            Text(
              label,
              style: TextStyle(
                fontSize: 14,
                fontWeight: isSelected ? FontWeight.w600 : FontWeight.w400,
                color: isSelected ? Colors.white : Colors.white54,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
