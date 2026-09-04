import 'package:flutter/material.dart';
import 'package:waydroid_profile_manager/domain/entities/waydroid_status.dart';

class StatusIndicator extends StatelessWidget {
  final WaydroidStatus status;

  const StatusIndicator({super.key, required this.status});

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        child: Row(
          children: [
            // Status dot
            Container(
              width: 12,
              height: 12,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: status.isRunning ? Colors.green : Colors.orange,
                boxShadow: status.isRunning
                    ? [
                        BoxShadow(
                          color: Colors.green.withValues(alpha: 0.4),
                          blurRadius: 8,
                          spreadRadius: 2,
                        ),
                      ]
                    : null,
              ),
            ),
            const SizedBox(width: 12),
            // Status text
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    status.summary,
                    style: const TextStyle(
                      fontSize: 15,
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                  if (status.installed) ...[
                    const SizedBox(height: 2),
                    Text(
                      'Container: ${status.container} • Session: ${status.session} • UI: ${status.ui}',
                      style: TextStyle(
                        fontSize: 12,
                        color: Colors.white.withValues(alpha: 0.5),
                      ),
                    ),
                  ],
                ],
              ),
            ),
            // Icon
            Icon(
              status.isRunning ? Icons.android : Icons.warning_amber,
              color: status.isRunning ? Colors.green : Colors.orange,
              size: 24,
            ),
          ],
        ),
      ),
    );
  }
}
