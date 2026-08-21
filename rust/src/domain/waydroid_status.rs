use serde::{Deserialize, Serialize};

/// Status of the Waydroid container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContainerStatus {
    Stopped,
    Running,
}

/// Status of the Waydroid session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Stopped,
    Running,
}

/// Status of the Waydroid UI (show-full-ui window).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiStatus {
    Closed,
    Open,
}

/// Overall Waydroid state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaydroidStatus {
    pub installed: bool,
    pub container: ContainerStatus,
    pub session: SessionStatus,
    pub ui: UiStatus,
}

impl WaydroidStatus {
    pub fn not_installed() -> Self {
        Self {
            installed: false,
            container: ContainerStatus::Stopped,
            session: SessionStatus::Stopped,
            ui: UiStatus::Closed,
        }
    }

    pub fn is_running(&self) -> bool {
        self.container == ContainerStatus::Running && self.session == SessionStatus::Running
    }

    pub fn summary(&self) -> String {
        if !self.installed {
            return "Waydroid not installed".to_string();
        }
        match (self.container, self.session, self.ui) {
            (ContainerStatus::Running, SessionStatus::Running, UiStatus::Open) => {
                "Waydroid Running (UI Open)".to_string()
            }
            (ContainerStatus::Running, SessionStatus::Running, UiStatus::Closed) => {
                "Waydroid Running (UI Closed)".to_string()
            }
            (ContainerStatus::Running, SessionStatus::Stopped, _) => {
                "Container Running, Session Stopped".to_string()
            }
            _ => "Waydroid Stopped".to_string(),
        }
    }
}

/// Progress step during an operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationProgress {
    pub step: String,
    pub message: String,
    pub completed: bool,
}

/// The state of an ongoing operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationState {
    Idle,
    Validating,
    ClosingUi,
    Configuring,
    StartingSession,
    OpeningUi,
    WaitingForUi,
    ApplyingRotation,
    Verifying,
    Completed,
    Error,
    Cancelled,
    Timeout,
}

impl std::fmt::Display for OperationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationState::Idle => write!(f, "Idle"),
            OperationState::Validating => write!(f, "Validating"),
            OperationState::ClosingUi => write!(f, "Closing UI"),
            OperationState::Configuring => write!(f, "Configuring"),
            OperationState::StartingSession => write!(f, "Starting Session"),
            OperationState::OpeningUi => write!(f, "Opening UI"),
            OperationState::WaitingForUi => write!(f, "Waiting for UI"),
            OperationState::ApplyingRotation => write!(f, "Applying Rotation"),
            OperationState::Verifying => write!(f, "Verifying"),
            OperationState::Completed => write!(f, "Completed"),
            OperationState::Error => write!(f, "Error"),
            OperationState::Cancelled => write!(f, "Cancelled"),
            OperationState::Timeout => write!(f, "Timeout"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_installed_status() {
        let s = WaydroidStatus::not_installed();
        assert!(!s.installed);
        assert!(!s.is_running());
    }

    #[test]
    fn running_status() {
        let s = WaydroidStatus {
            installed: true,
            container: ContainerStatus::Running,
            session: SessionStatus::Running,
            ui: UiStatus::Open,
        };
        assert!(s.is_running());
    }

    #[test]
    fn session_stopped_not_running() {
        let s = WaydroidStatus {
            installed: true,
            container: ContainerStatus::Running,
            session: SessionStatus::Stopped,
            ui: UiStatus::Closed,
        };
        assert!(!s.is_running());
    }

    #[test]
    fn summary_messages() {
        let s = WaydroidStatus::not_installed();
        assert_eq!(s.summary(), "Waydroid not installed");

        let s = WaydroidStatus {
            installed: true,
            container: ContainerStatus::Running,
            session: SessionStatus::Running,
            ui: UiStatus::Open,
        };
        assert!(s.summary().contains("Running"));
    }

    #[test]
    fn operation_state_display() {
        assert_eq!(OperationState::Validating.to_string(), "Validating");
        assert_eq!(OperationState::Completed.to_string(), "Completed");
    }
}
