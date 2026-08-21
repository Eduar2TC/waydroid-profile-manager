import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../application/controllers/app_controller.dart';

class LogsScreen extends StatelessWidget {
  const LogsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Consumer<AppController>(
      builder: (context, controller, _) {
        return Scaffold(
          appBar: AppBar(
            title: const Text('Logs'),
            actions: [
              IconButton(
                icon: const Icon(Icons.delete_sweep),
                onPressed: () => controller.clearLogs(),
                tooltip: 'Clear Logs',
              ),
            ],
          ),
          body: controller.logs.isEmpty
              ? const Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(Icons.terminal, size: 48, color: Colors.white30),
                      SizedBox(height: 16),
                      Text(
                        'No logs yet',
                        style: TextStyle(color: Colors.white54),
                      ),
                      SizedBox(height: 8),
                      Text(
                        'Logs will appear here when you perform actions',
                        style: TextStyle(color: Colors.white38, fontSize: 13),
                      ),
                    ],
                  ),
                )
              : _buildLogList(controller),
        );
      },
    );
  }

  Widget _buildLogList(AppController controller) {
    return ListView.builder(
      padding: const EdgeInsets.all(8),
      itemCount: controller.logs.length,
      itemBuilder: (context, index) {
        final log = controller.logs[controller.logs.length - 1 - index];
        return _buildLogEntry(log);
      },
    );
  }

  Widget _buildLogEntry(String log) {
    Color? color;
    if (log.contains('✓')) {
      color = Colors.green;
    } else if (log.contains('✗') || log.contains('Error')) {
      color = Colors.red;
    } else if (log.contains('Warning')) {
      color = Colors.orange;
    }

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
      decoration: BoxDecoration(
        border: Border(
          bottom: BorderSide(
            color: Colors.white.withValues(alpha: 0.05),
            width: 0.5,
          ),
        ),
      ),
      child: Text(
        log,
        style: TextStyle(
          fontFamily: 'monospace',
          fontSize: 13,
          color: color,
          height: 1.5,
        ),
      ),
    );
  }
}
