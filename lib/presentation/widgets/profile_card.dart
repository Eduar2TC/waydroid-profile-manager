import 'package:flutter/material.dart';
import 'package:waydroid_profile_manager/domain/entities/profile.dart';

class ProfileCard extends StatelessWidget {
  final Profile profile;
  final bool isActive;
  final VoidCallback? onTap;

  const ProfileCard({
    super.key,
    required this.profile,
    this.isActive = false,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      color: isActive
          ? Theme.of(context).colorScheme.primaryContainer.withValues(alpha: 0.3)
          : null,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Row(
            children: [
              // Profile icon
              Container(
                width: 48,
                height: 48,
                decoration: BoxDecoration(
                  color: isActive
                      ? Theme.of(context).colorScheme.primary.withValues(alpha: 0.2)
                      : Colors.white.withValues(alpha: 0.05),
                  borderRadius: BorderRadius.circular(12),
                ),
                child: Icon(
                  _getIcon(),
                  color: isActive
                      ? Theme.of(context).colorScheme.primary
                      : Colors.white54,
                  size: 24,
                ),
              ),
              const SizedBox(width: 16),
              // Profile info
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Text(
                          profile.name,
                          style: TextStyle(
                            fontSize: 16,
                            fontWeight: FontWeight.w500,
                            color: isActive ? Colors.white : null,
                          ),
                        ),
                        if (profile.builtin) ...[
                          const SizedBox(width: 6),
                          Icon(
                            Icons.star,
                            size: 14,
                            color: Colors.amber.withValues(alpha: 0.8),
                          ),
                        ],
                      ],
                    ),
                    const SizedBox(height: 4),
                    Text(
                      '${profile.baseWidth}×${profile.baseHeight} • ${profile.density} DPI',
                      style: TextStyle(
                        fontSize: 13,
                        color: Colors.white.withValues(alpha: 0.6),
                      ),
                    ),
                    if (profile.description != null &&
                        profile.description!.isNotEmpty) ...[
                      const SizedBox(height: 2),
                      Text(
                        profile.description!,
                        style: TextStyle(
                          fontSize: 12,
                          color: Colors.white.withValues(alpha: 0.4),
                        ),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ],
                  ],
                ),
              ),
              // Active badge or chevron
              if (isActive)
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                  decoration: BoxDecoration(
                    color: Theme.of(context).colorScheme.primary,
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: const Text(
                    'ACTIVE',
                    style: TextStyle(
                      fontSize: 10,
                      fontWeight: FontWeight.w700,
                      letterSpacing: 0.5,
                    ),
                  ),
                )
              else
                Icon(
                  Icons.chevron_right,
                  color: Colors.white.withValues(alpha: 0.3),
                ),
            ],
          ),
        ),
      ),
    );
  }

  IconData _getIcon() {
    if (profile.baseWidth > profile.baseHeight) {
      return Icons.tablet; // Landscape-ish
    }
    return Icons.phone_android;
  }
}
