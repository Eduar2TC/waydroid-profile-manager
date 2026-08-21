/// The state of an ongoing Waydroid operation.
enum OperationState {
  idle,
  validating,
  closingUi,
  configuring,
  startingSession,
  openingUi,
  waitingForUi,
  applyingRotation,
  verifying,
  completed,
  error,
  cancelled,
  timeout;

  String get displayName {
    switch (this) {
      case OperationState.idle:
        return 'Idle';
      case OperationState.validating:
        return 'Validating';
      case OperationState.closingUi:
        return 'Closing UI';
      case OperationState.configuring:
        return 'Configuring';
      case OperationState.startingSession:
        return 'Starting Session';
      case OperationState.openingUi:
        return 'Opening UI';
      case OperationState.waitingForUi:
        return 'Waiting for UI';
      case OperationState.applyingRotation:
        return 'Applying Rotation';
      case OperationState.verifying:
        return 'Verifying';
      case OperationState.completed:
        return 'Completed';
      case OperationState.error:
        return 'Error';
      case OperationState.cancelled:
        return 'Cancelled';
      case OperationState.timeout:
        return 'Timeout';
    }
  }

  bool get isCompleted =>
      this == OperationState.completed ||
      this == OperationState.error ||
      this == OperationState.cancelled ||
      this == OperationState.timeout;

  bool get isRunning =>
      !isCompleted && this != OperationState.idle;

  static OperationState fromString(String value) {
    return OperationState.values.firstWhere(
      (e) => e.name == value,
      orElse: () => OperationState.idle,
    );
  }
}
