use std::time::Duration;
use std::time::Instant;

use crate::domain::waydroid_status::{
    ContainerStatus, SessionStatus, UiStatus, WaydroidStatus,
};
use crate::infrastructure::process::result::CommandResult;
use crate::infrastructure::process::runner::ProcessRunner;

use super::backend_trait::WaydroidBackend;

/// Production implementation of WaydroidBackend using real waydroid CLI.
pub struct RealWaydroidBackend {
    runner: ProcessRunner,
    waydroid_path: String,
}

impl RealWaydroidBackend {
    pub fn new() -> Self {
        Self {
            runner: ProcessRunner::new(Duration::from_secs(30)),
            waydroid_path: "waydroid".to_string(),
        }
    }

    pub fn with_runner(runner: ProcessRunner) -> Self {
        Self {
            runner,
            waydroid_path: "waydroid".to_string(),
        }
    }

    fn waydroid_cmd(&self, args: &[&str]) -> anyhow::Result<CommandResult> {
        self.runner.run(&self.waydroid_path, args)
    }

    /// Parse `waydroid status` output for container and session state.
    ///
    /// `waydroid status` outputs lines like:
    /// ```text
    /// Session:    RUNNING
    /// Container:  RUNNING
    /// ```
    fn parse_status_output(&self) -> (ContainerStatus, SessionStatus) {
        let mut container = ContainerStatus::Stopped;
        let mut session = SessionStatus::Stopped;

        if let Ok(result) = self.waydroid_cmd(&["status"]) {
            let stdout = result.stdout.to_lowercase();
            for line in stdout.lines() {
                let line = line.trim();
                if line.starts_with("session:") && line.contains("running") {
                    session = SessionStatus::Running;
                }
                if line.starts_with("container:") && line.contains("running") {
                    container = ContainerStatus::Running;
                }
            }
        }

        (container, session)
    }

    /// Restart session by stopping then starting.
    /// Waits for the container to fully stop before starting.
    pub fn session_restart(&self) -> anyhow::Result<()> {
        let _ = self.session_stop();
        // Wait for container to fully stop (poll status)
        for _ in 0..20 {
            std::thread::sleep(std::time::Duration::from_secs(1));
            let (container, session) = self.parse_status_output();
            if container == ContainerStatus::Stopped && session == SessionStatus::Stopped {
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
        self.session_start()?;
        // Wait for session to be running
        for _ in 0..30 {
            std::thread::sleep(std::time::Duration::from_secs(1));
            let (_, session) = self.parse_status_output();
            if session == SessionStatus::Running {
                return Ok(());
            }
        }
        Err(anyhow::anyhow!("Session restart timed out"))
    }
}

impl WaydroidBackend for RealWaydroidBackend {
    fn check_installed(&self) -> bool {
        ProcessRunner::program_exists(&self.waydroid_path)
    }

    fn get_container_status(&self) -> ContainerStatus {
        self.parse_status_output().0
    }

    fn get_session_status(&self) -> SessionStatus {
        self.parse_status_output().1
    }

    fn get_ui_status(&self) -> UiStatus {
        // Check if waydroid show-full-ui process is running
        match self.runner.run("pgrep", &["-f", "waydroid show-full-ui"]) {
            Ok(result) if result.success && !result.stdout.trim().is_empty() => UiStatus::Open,
            _ => UiStatus::Closed,
        }
    }

    fn get_full_status(&self) -> WaydroidStatus {
        let installed = self.check_installed();
        let (container, session) = self.parse_status_output();
        let ui = self.get_ui_status();
        WaydroidStatus {
            installed,
            container,
            session,
            ui,
        }
    }

    fn set_prop(&self, key: &str, value: &str) -> anyhow::Result<CommandResult> {
        self.waydroid_cmd(&["prop", "set", key, value])
    }

    fn session_start(&self) -> anyhow::Result<CommandResult> {
        self.waydroid_cmd(&["session", "start"])
    }

    fn session_stop(&self) -> anyhow::Result<CommandResult> {
        self.waydroid_cmd(&["session", "stop"])
    }

    fn show_full_ui(&self) -> anyhow::Result<CommandResult> {
        // Launch UI in background - the command starts and returns immediately
        use std::process::Command;
        Command::new(&self.waydroid_path)
            .args(["show-full-ui"])
            .spawn()
            .map(|_| CommandResult::success(String::new(), String::new(), Duration::from_millis(0)))
            .map_err(|e| anyhow::anyhow!("Failed to launch waydroid UI: {}", e))
    }

    fn close_full_ui(&self) -> anyhow::Result<CommandResult> {
        // First try graceful close via pgrep + kill
        match self.runner.run("pgrep", &["-f", "waydroid show-full-ui"]) {
            Ok(result) if result.success => {
                let pids: Vec<&str> = result.stdout.trim().split_whitespace().collect();
                for pid in pids {
                    if !pid.is_empty() {
                        let _ = self.runner.run("kill", &[pid]);
                    }
                }
                Ok(CommandResult::success(
                    String::new(),
                    String::new(),
                    Duration::from_millis(100),
                ))
            }
            _ => Ok(CommandResult::success(
                String::new(),
                "No UI process found".to_string(),
                Duration::from_millis(0),
            )),
        }
    }

    fn shell_command(&self, args: &[&str]) -> anyhow::Result<CommandResult> {
        // waydroid shell requires root (UID 0 check in waydroid's Python code).
        // Use `sudo -n` which works without password prompt when sudoers is configured.
        let mut shell_args = vec!["-n", "waydroid", "shell"];
        shell_args.extend_from_slice(args);
        self.runner.run("sudo", &shell_args)
    }

    fn wait_for_session(&self, timeout_secs: u64) -> bool {
        let start = Instant::now();
        let timeout = Duration::from_secs(timeout_secs);
        let poll_interval = Duration::from_millis(500);

        loop {
            if self.get_session_status() == SessionStatus::Running {
                return true;
            }
            if start.elapsed() >= timeout {
                return false;
            }
            std::thread::sleep(poll_interval);
        }
    }

    fn wait_for_session_stopped(&self, timeout_secs: u64) -> bool {
        let start = Instant::now();
        let timeout = Duration::from_secs(timeout_secs);
        let poll_interval = Duration::from_millis(1000);

        loop {
            let (container, session) = self.parse_status_output();
            if container == ContainerStatus::Stopped && session == SessionStatus::Stopped {
                return true;
            }
            if start.elapsed() >= timeout {
                return false;
            }
            std::thread::sleep(poll_interval);
        }
    }

    fn wait_for_ui(&self, timeout_secs: u64) -> bool {
        let start = Instant::now();
        let timeout = Duration::from_secs(timeout_secs);
        let poll_interval = Duration::from_millis(500);

        loop {
            if self.get_ui_status() == UiStatus::Open {
                return true;
            }
            if start.elapsed() >= timeout {
                return false;
            }
            std::thread::sleep(poll_interval);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_waydroid_installed() {
        let backend = RealWaydroidBackend::new();
        let installed = backend.check_installed();
        // On the dev machine waydroid should be installed
        assert!(installed, "Waydroid should be installed on this machine");
    }

    #[test]
    fn get_full_status_works() {
        let backend = RealWaydroidBackend::new();
        let status = backend.get_full_status();
        assert!(status.installed);
        // Session/container should be parseable (may or may not be running)
        println!("Full status: {:?}", status);
    }

    #[test]
    fn parse_status_output_parses_correctly() {
        let backend = RealWaydroidBackend::new();
        let (container, session) = backend.parse_status_output();
        // Should not panic, values should be valid enum variants
        println!("Container: {:?}, Session: {:?}", container, session);
    }

    #[test]
    fn set_prop_works() {
        let backend = RealWaydroidBackend::new();
        let result = backend.set_prop("persist.waydroid.width", "640");
        assert!(result.is_ok(), "set_prop failed: {:?}", result);
    }
}
