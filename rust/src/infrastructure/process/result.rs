use std::time::Duration;

/// Result of a process execution.
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
    pub success: bool,
}

impl CommandResult {
    pub fn success(stdout: String, stderr: String, duration: Duration) -> Self {
        Self {
            exit_code: Some(0),
            stdout,
            stderr,
            duration,
            success: true,
        }
    }

    pub fn failure(exit_code: Option<i32>, stdout: String, stderr: String, duration: Duration) -> Self {
        Self {
            exit_code,
            stdout,
            stderr,
            duration,
            success: false,
        }
    }
}
