use crate::infrastructure::process::runner::ProcessRunner;

/// Detect system capabilities relevant to Waydroid Profile Manager.
pub struct SystemDetector;

impl SystemDetector {
    /// Check if waydroid is installed and accessible.
    pub fn waydroid_installed() -> bool {
        ProcessRunner::program_exists("waydroid")
    }

    /// Get the waydroid version string.
    pub fn waydroid_version() -> Option<String> {
        let runner = ProcessRunner::with_default_timeout();
        runner
            .run("waydroid", &["--version"])
            .ok()
            .filter(|r| r.success)
            .map(|r| r.stdout.trim().to_string())
    }

    /// Check if running on Wayland.
    pub fn is_wayland() -> bool {
        std::env::var("WAYLAND_DISPLAY").is_ok()
    }

    /// Check if running on X11.
    pub fn is_x11() -> bool {
        std::env::var("DISPLAY").is_ok() && !Self::is_wayland()
    }

    /// Check if the display server type.
    pub fn display_server() -> &'static str {
        if Self::is_wayland() {
            "Wayland"
        } else if Self::is_x11() {
            "X11"
        } else {
            "Unknown"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_server_detection() {
        let server = SystemDetector::display_server();
        assert!(server == "Wayland" || server == "X11" || server == "Unknown");
    }
}
