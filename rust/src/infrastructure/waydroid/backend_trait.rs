use crate::domain::waydroid_status::{
    ContainerStatus, SessionStatus, UiStatus, WaydroidStatus,
};
use crate::infrastructure::process::result::CommandResult;

/// Trait abstracting Waydroid operations.
/// Production: RealWaydroidBackend
/// Tests: FakeWaydroidBackend
pub trait WaydroidBackend: Send + Sync {
    fn check_installed(&self) -> bool;
    fn get_container_status(&self) -> ContainerStatus;
    fn get_session_status(&self) -> SessionStatus;
    fn get_ui_status(&self) -> UiStatus;
    fn get_full_status(&self) -> WaydroidStatus;

    fn set_prop(&self, key: &str, value: &str) -> anyhow::Result<CommandResult>;
    fn session_start(&self) -> anyhow::Result<CommandResult>;
    fn session_stop(&self) -> anyhow::Result<CommandResult>;
    fn show_full_ui(&self) -> anyhow::Result<CommandResult>;
    fn close_full_ui(&self) -> anyhow::Result<CommandResult>;
    fn shell_command(&self, args: &[&str]) -> anyhow::Result<CommandResult>;
    fn wait_for_session(&self, timeout_secs: u64) -> bool;
    fn wait_for_session_stopped(&self, timeout_secs: u64) -> bool;
    fn wait_for_ui(&self, timeout_secs: u64) -> bool;
}
