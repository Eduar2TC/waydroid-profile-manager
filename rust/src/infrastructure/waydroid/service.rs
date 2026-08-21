use std::collections::HashMap;
use std::sync::Mutex;

use crate::domain::waydroid_status::{
    ContainerStatus, SessionStatus, UiStatus, WaydroidStatus,
};
use crate::infrastructure::process::result::CommandResult;

use super::backend_trait::WaydroidBackend;

use std::time::Duration;

/// Fake Waydroid backend for testing.
/// Simulates waydroid behavior without requiring real installation.
pub struct FakeWaydroidBackend {
    pub state: Mutex<FakeState>,
}

pub struct FakeState {
    pub installed: bool,
    pub container: ContainerStatus,
    pub session: SessionStatus,
    pub ui: UiStatus,
    pub props: HashMap<String, String>,
    pub commands_executed: Vec<String>,
    pub session_start_should_fail: bool,
    pub set_prop_should_fail: bool,
}

impl FakeWaydroidBackend {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(FakeState {
                installed: true,
                container: ContainerStatus::Running,
                session: SessionStatus::Running,
                ui: UiStatus::Closed,
                props: HashMap::new(),
                commands_executed: Vec::new(),
                session_start_should_fail: false,
                set_prop_should_fail: false,
            }),
        }
    }

    pub fn not_installed() -> Self {
        let fake = Self::new();
        {
            let mut state = fake.state.lock().unwrap();
            state.installed = false;
        }
        fake
    }

    pub fn stopped() -> Self {
        let fake = Self::new();
        {
            let mut state = fake.state.lock().unwrap();
            state.container = ContainerStatus::Stopped;
            state.session = SessionStatus::Stopped;
        }
        fake
    }
}

impl WaydroidBackend for FakeWaydroidBackend {
    fn check_installed(&self) -> bool {
        self.state.lock().unwrap().installed
    }

    fn get_container_status(&self) -> ContainerStatus {
        self.state.lock().unwrap().container
    }

    fn get_session_status(&self) -> SessionStatus {
        self.state.lock().unwrap().session
    }

    fn get_ui_status(&self) -> UiStatus {
        self.state.lock().unwrap().ui
    }

    fn get_full_status(&self) -> WaydroidStatus {
        let state = self.state.lock().unwrap();
        WaydroidStatus {
            installed: state.installed,
            container: state.container,
            session: state.session,
            ui: state.ui,
        }
    }

    fn set_prop(&self, key: &str, value: &str) -> anyhow::Result<CommandResult> {
        let mut state = self.state.lock().unwrap();
        state
            .commands_executed
            .push(format!("prop set {} {}", key, value));

        if state.set_prop_should_fail {
            return Ok(CommandResult::failure(
                Some(1),
                String::new(),
                "Fake error setting property".to_string(),
                Duration::from_millis(10),
            ));
        }

        state.props.insert(key.to_string(), value.to_string());
        Ok(CommandResult::success(
            String::new(),
            String::new(),
            Duration::from_millis(10),
        ))
    }

    fn session_start(&self) -> anyhow::Result<CommandResult> {
        let mut state = self.state.lock().unwrap();
        state.commands_executed.push("session start".to_string());

        if state.session_start_should_fail {
            return Ok(CommandResult::failure(
                Some(1),
                String::new(),
                "Fake session start error".to_string(),
                Duration::from_millis(10),
            ));
        }

        state.session = SessionStatus::Running;
        state.container = ContainerStatus::Running;
        Ok(CommandResult::success(
            String::new(),
            String::new(),
            Duration::from_millis(10),
        ))
    }

    fn session_stop(&self) -> anyhow::Result<CommandResult> {
        let mut state = self.state.lock().unwrap();
        state.commands_executed.push("session stop".to_string());
        state.session = SessionStatus::Stopped;
        state.container = ContainerStatus::Stopped;
        Ok(CommandResult::success(
            String::new(),
            String::new(),
            Duration::from_millis(10),
        ))
    }

    fn show_full_ui(&self) -> anyhow::Result<CommandResult> {
        let mut state = self.state.lock().unwrap();
        state.commands_executed.push("show-full-ui".to_string());
        state.ui = UiStatus::Open;
        Ok(CommandResult::success(
            String::new(),
            String::new(),
            Duration::from_millis(10),
        ))
    }

    fn close_full_ui(&self) -> anyhow::Result<CommandResult> {
        let mut state = self.state.lock().unwrap();
        state.commands_executed.push("close-ui".to_string());
        state.ui = UiStatus::Closed;
        Ok(CommandResult::success(
            String::new(),
            String::new(),
            Duration::from_millis(10),
        ))
    }

    fn shell_command(&self, args: &[&str]) -> anyhow::Result<CommandResult> {
        let mut state = self.state.lock().unwrap();
        state
            .commands_executed
            .push(format!("shell {}", args.join(" ")));
        Ok(CommandResult::success(
            String::new(),
            String::new(),
            Duration::from_millis(10),
        ))
    }

    fn wait_for_session(&self, _timeout_secs: u64) -> bool {
        let state = self.state.lock().unwrap();
        state.session == SessionStatus::Running
    }

    fn wait_for_session_stopped(&self, _timeout_secs: u64) -> bool {
        let state = self.state.lock().unwrap();
        state.session == SessionStatus::Stopped && state.container == ContainerStatus::Stopped
    }

    fn wait_for_ui(&self, _timeout_secs: u64) -> bool {
        let state = self.state.lock().unwrap();
        state.ui == UiStatus::Open
    }

    fn container_stop_graceful(&self, _timeout_secs: u32) -> anyhow::Result<()> {
        let mut state = self.state.lock().unwrap();
        state.commands_executed.push("container stop graceful".to_string());
        state.session = SessionStatus::Stopped;
        state.container = ContainerStatus::Stopped;
        Ok(())
    }

    fn container_start(&self) -> anyhow::Result<()> {
        let mut state = self.state.lock().unwrap();
        state.commands_executed.push("container start".to_string());
        state.session = SessionStatus::Running;
        state.container = ContainerStatus::Running;
        Ok(())
    }
}
