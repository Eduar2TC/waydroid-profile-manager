use std::process::Command;
use std::time::{Duration, Instant};

use super::result::CommandResult;

/// Runs external processes safely with timeout support.
/// Uses separate arguments (no shell injection).
pub struct ProcessRunner {
    timeout: Duration,
}

impl ProcessRunner {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    pub fn with_default_timeout() -> Self {
        Self {
            timeout: Duration::from_secs(30),
        }
    }

    /// Execute a command with separate arguments.
    /// Returns a CommandResult with stdout, stderr, exit code, and duration.
    pub fn run(&self, program: &str, args: &[&str]) -> anyhow::Result<CommandResult> {
        let start = Instant::now();

        log::debug!("Running: {} {}", program, args.join(" "));

        let output = Command::new(program)
            .args(args)
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute '{}': {}", program, e))?;

        let duration = start.elapsed();

        if duration > self.timeout {
            return Ok(CommandResult::failure(
                None,
                String::new(),
                format!("Command timed out after {}s", self.timeout.as_secs()),
                duration,
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code();
        let success = output.status.success();

        if !success {
            log::warn!(
                "Command '{}' failed with exit code {:?}: {}",
                program,
                exit_code,
                stderr.trim()
            );
        }

        Ok(CommandResult {
            exit_code,
            stdout,
            stderr,
            duration,
            success,
        })
    }

    /// Execute a command with a custom timeout.
    pub fn run_with_timeout(
        &self,
        program: &str,
        args: &[&str],
        timeout: Duration,
    ) -> anyhow::Result<CommandResult> {
        let start = Instant::now();

        log::debug!("Running (timeout {}s): {} {}", timeout.as_secs(), program, args.join(" "));

        let output = Command::new(program)
            .args(args)
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute '{}': {}", program, e))?;

        let duration = start.elapsed();

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code();
        let success = output.status.success();

        if duration > timeout {
            return Ok(CommandResult::failure(
                None,
                stdout,
                stderr,
                duration,
            ));
        }

        Ok(CommandResult {
            exit_code,
            stdout,
            stderr,
            duration,
            success,
        })
    }

    /// Launch a command in the background (fire-and-forget).
    /// Returns Ok immediately without waiting for the process to finish.
    pub fn run_background(&self, program: &str, args: &[&str]) -> anyhow::Result<()> {
        log::info!("Launching background: {} {}", program, args.join(" "));
        Command::new(program)
            .args(args)
            .spawn()
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!("Failed to launch '{}': {}", program, e))
    }

    /// Check if a program exists in PATH.
    pub fn program_exists(program: &str) -> bool {
        Command::new("which")
            .arg(program)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_echo() {
        let runner = ProcessRunner::with_default_timeout();
        let result = runner.run("echo", &["hello"]).unwrap();
        assert!(result.success);
        assert_eq!(result.stdout.trim(), "hello");
    }

    #[test]
    fn run_with_stderr() {
        let runner = ProcessRunner::with_default_timeout();
        let result = runner.run("ls", &["/nonexistent_path_12345"]).unwrap();
        assert!(!result.success);
    }

    #[test]
    fn program_exists_check() {
        assert!(ProcessRunner::program_exists("ls"));
        assert!(!ProcessRunner::program_exists("nonexistent_program_xyz"));
    }

    #[test]
    fn command_result_success() {
        let result = CommandResult::success(
            "output".into(),
            String::new(),
            Duration::from_millis(10),
        );
        assert!(result.success);
        assert_eq!(result.exit_code, Some(0));
    }

    #[test]
    fn command_result_failure() {
        let result = CommandResult::failure(
            Some(1),
            String::new(),
            "error".into(),
            Duration::from_millis(10),
        );
        assert!(!result.success);
        assert_eq!(result.exit_code, Some(1));
    }
}
