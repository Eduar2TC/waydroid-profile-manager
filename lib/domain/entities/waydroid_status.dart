/// Overall Waydroid state.
class WaydroidStatus {
  final bool installed;
  final String container; // "running" or "stopped"
  final String session; // "running" or "stopped"
  final String ui; // "open" or "closed"

  const WaydroidStatus({
    required this.installed,
    required this.container,
    required this.session,
    required this.ui,
  });

  factory WaydroidStatus.fromJson(Map<String, dynamic> json) {
    return WaydroidStatus(
      installed: json['installed'] as bool,
      container: json['container'] as String,
      session: json['session'] as String,
      ui: json['ui'] as String,
    );
  }

  factory WaydroidStatus.notInstalled() {
    return const WaydroidStatus(
      installed: false,
      container: 'stopped',
      session: 'stopped',
      ui: 'closed',
    );
  }

  bool get isRunning => container == 'running' && session == 'running';
  bool get isUiOpen => ui == 'open';

  String get summary {
    if (!installed) return 'Waydroid not installed';
    if (isRunning && isUiOpen) return 'Waydroid Running (UI Open)';
    if (isRunning) return 'Waydroid Running (UI Closed)';
    return 'Waydroid Stopped';
  }
}
