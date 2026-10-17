use crate::domain::waydroid_status::{SessionStatus, UiStatus};
use crate::infrastructure::storage::profiles::{AppState, ProfileRepository};
use crate::infrastructure::waydroid::backend_trait::WaydroidBackend;

/// Result of a reset operation.
#[derive(Debug, Clone)]
pub struct ResetResult {
    pub success: bool,
    pub message: String,
    pub error: Option<String>,
}

/// Reset Waydroid configuration to the state before any profile was applied.
pub fn reset_waydroid(
    backend: &dyn WaydroidBackend,
    repository: &ProfileRepository,
) -> ResetResult {
    log::info!("Resetting Waydroid configuration");

    // Load backup
    let state = match repository.load_state() {
        Ok(s) => s,
        Err(e) => {
            return ResetResult {
                success: false,
                message: format!("Failed to load state: {}", e),
                error: Some("StateLoadFailed".to_string()),
            };
        }
    };

    let backup = match state.backup {
        Some(b) => b,
        None => {
            return ResetResult {
                success: false,
                message: "No backup available to restore".to_string(),
                error: Some("NoBackup".to_string()),
            };
        }
    };

    // Close UI — remember whether it was open so we only reopen what we closed.
    let ui_was_open = backend.get_ui_status() == UiStatus::Open;
    let _ = backend.close_full_ui();

    let mut restored: Vec<&str> = Vec::new();

    // 1. Restore resolution and density props (host window config).
    // `waydroid prop set` requires the session to be running, and set_prop
    // returns Ok(CommandResult) even when the CLI exits non-zero — only a
    // command that reports success counts as restored.
    if let Some(ref width) = backup.width {
        if set_prop_ok(backend, "persist.waydroid.width", width) {
            restored.push("width");
        }
    }
    if let Some(ref height) = backup.height {
        if set_prop_ok(backend, "persist.waydroid.height", height) {
            restored.push("height");
        }
    }
    if let Some(ref density) = backup.density {
        if set_prop_ok(backend, "persist.waydroid.density", density) {
            restored.push("density");
        }
    }

    let session_was_running = backend.get_session_status() == SessionStatus::Running;

    // 2. Restart the session so the restored props take effect
    // (same flow as apply: props set while running, then restart).
    if session_was_running {
        log::info!("Reset: restarting session to apply restored configuration");
        let _ = backend.container_stop_graceful(10);
        let _ = backend.wait_for_session_stopped(15);
        if let Err(e) = backend.container_start() {
            log::warn!("Reset: failed to restart session: {}", e);
        } else {
            let _ = backend.wait_for_session(45);
            let _ = backend.wait_for_shell_ready(30);
        }
    }

    // 3. Restore the internal Android display override captured before
    // applying. `persist.waydroid.width/height` (host window) and `wm size`
    // (internal display) are different values, so the backup records the
    // actual wm override rather than reusing the window size. When no
    // override was recorded, clear the one apply() left behind so the display
    // returns to its physical size/density.
    if let Some(ref size) = backup.wm_size {
        if restore_shell(backend, &["wm", "size", size]) {
            restored.push("wm size");
        }
    } else {
        restore_shell(backend, &["wm", "size", "reset"]);
    }
    if let Some(ref density) = backup.wm_density {
        if restore_shell(backend, &["wm", "density", density]) {
            restored.push("wm density");
        }
    } else {
        restore_shell(backend, &["wm", "density", "reset"]);
    }

    // 4. Restore rotation settings if available
    let mut rotation_restored = false;
    if let Some(ref accel) = backup.accelerometer_rotation {
        rotation_restored |= restore_shell(
            backend,
            &[
                "cmd",
                "settings",
                "put",
                "system",
                "accelerometer_rotation",
                accel,
            ],
        );
    }

    if let Some(ref rotation) = backup.user_rotation {
        rotation_restored |= restore_shell(
            backend,
            &["cmd", "settings", "put", "system", "user_rotation", rotation],
        );
    }
    if rotation_restored {
        restored.push("rotation");
    }

    // 5. Reopen the UI only if we closed an open one
    if ui_was_open {
        let _ = backend.show_full_ui();
    }

    // Clear active profile (this reset consumes the backup)
    let _ = repository.save_state(&AppState {
        active_profile_id: None,
        backup: None,
    });

    let message = if restored.is_empty() {
        "Nothing to restore".to_string()
    } else {
        format!("Restored: {}", restored.join(", "))
    };

    ResetResult {
        success: true,
        message,
        error: None,
    }
}

/// `set_prop` returns Ok(CommandResult) even when the CLI itself failed, so
/// check `.success` before claiming a value was restored.
fn set_prop_ok(backend: &dyn WaydroidBackend, key: &str, value: &str) -> bool {
    matches!(backend.set_prop(key, value), Ok(result) if result.success)
}

/// Run a restore shell command; returns whether it succeeded.
fn restore_shell(backend: &dyn WaydroidBackend, args: &[&str]) -> bool {
    match backend.shell_command(args) {
        Ok(result) if result.success => true,
        Ok(result) => {
            log::warn!(
                "Reset: '{}' failed: {}",
                args.join(" "),
                result.stderr.trim()
            );
            false
        }
        Err(e) => {
            log::warn!("Reset: '{}' errored: {}", args.join(" "), e);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::storage::profiles::{ProfileRepository, WaydroidBackup};
    use crate::infrastructure::waydroid::service::FakeWaydroidBackend;
    use tempfile::TempDir;

    fn setup_with_backup() -> (FakeWaydroidBackend, ProfileRepository, TempDir) {
        let backend = FakeWaydroidBackend::new();
        let tmp = TempDir::new().unwrap();
        let repo = ProfileRepository::with_config_dir(tmp.path().to_path_buf());

        // Create a backup
        let state = AppState {
            active_profile_id: Some("phone-sm".into()),
            backup: Some(WaydroidBackup {
                width: Some("800".into()),
                height: Some("600".into()),
                density: Some("160".into()),
                // Deliberately different from the host window size above: the
                // internal display override is its own value.
                wm_size: Some("480x800".into()),
                wm_density: Some("120".into()),
                accelerometer_rotation: Some("1".into()),
                user_rotation: Some("0".into()),
                timestamp: "2024-01-01T00:00:00Z".to_string(),
            }),
        };
        repo.save_state(&state).unwrap();

        (backend, repo, tmp)
    }

    #[test]
    fn reset_restores_backup() {
        let (backend, repo, _tmp) = setup_with_backup();
        let result = reset_waydroid(&backend, &repo);
        assert!(result.success);

        let state = backend.state.lock().unwrap();
        // Resolution and density props were restored
        assert_eq!(state.props["persist.waydroid.width"], "800");
        assert_eq!(state.props["persist.waydroid.height"], "600");
        assert_eq!(state.props["persist.waydroid.density"], "160");
        // The captured wm override was restored (not the host window size)
        assert!(state
            .commands_executed
            .iter()
            .any(|c| c.contains("wm size 480x800")));
        assert!(state
            .commands_executed
            .iter()
            .any(|c| c.contains("wm density 120")));
        // Rotation was restored
        assert!(state
            .commands_executed
            .iter()
            .any(|c| c.contains("accelerometer_rotation")));
        assert_eq!(
            result.message,
            "Restored: width, height, density, wm size, wm density, rotation"
        );
        // State cleared
        drop(state);
        let final_state = repo.load_state().unwrap();
        assert!(final_state.active_profile_id.is_none());
        assert!(final_state.backup.is_none());
    }

    #[test]
    fn reset_skips_missing_props() {
        let (backend, repo, _tmp) = setup_with_backup();
        // Remove everything to simulate an old backup without these fields
        {
            let mut st = repo.load_state().unwrap();
            if let Some(backup) = st.backup.as_mut() {
                backup.width = None;
                backup.height = None;
                backup.density = None;
                backup.wm_size = None;
                backup.wm_density = None;
            }
            repo.save_state(&st).unwrap();
        }
        let result = reset_waydroid(&backend, &repo);
        assert!(result.success);
        assert_eq!(result.message, "Restored: rotation");
        let state = backend.state.lock().unwrap();
        // No host window size may leak into `wm size`: with no override
        // recorded, apply()'s override must be cleared instead.
        assert!(state
            .commands_executed
            .iter()
            .any(|c| c.contains("wm size reset")));
        assert!(state
            .commands_executed
            .iter()
            .any(|c| c.contains("wm density reset")));
        assert!(!state.commands_executed.iter().any(|c| {
            (c.starts_with("shell wm size ") || c.starts_with("shell wm density "))
                && !c.ends_with("reset")
        }));
        assert!(!state
            .commands_executed
            .iter()
            .any(|c| c.starts_with("prop set")));
    }

    #[test]
    fn reset_does_not_report_props_that_failed_to_restore() {
        let (backend, repo, _tmp) = setup_with_backup();
        {
            // set_prop returns Ok(CommandResult) with success=false here
            let mut state = backend.state.lock().unwrap();
            state.set_prop_should_fail = true;
        }
        let result = reset_waydroid(&backend, &repo);
        assert!(result.success);
        assert_eq!(result.message, "Restored: wm size, wm density, rotation");
    }

    #[test]
    fn reset_restarts_running_session() {
        let (backend, repo, _tmp) = setup_with_backup();
        let result = reset_waydroid(&backend, &repo);
        assert!(result.success);
        let state = backend.state.lock().unwrap();
        assert!(state
            .commands_executed
            .contains(&"container stop graceful".to_string()));
        assert!(state
            .commands_executed
            .contains(&"container start".to_string()));
    }

    #[test]
    fn reset_reopens_ui_only_if_it_was_open() {
        let (backend, repo, _tmp) = setup_with_backup();
        reset_waydroid(&backend, &repo);
        {
            let state = backend.state.lock().unwrap();
            assert!(!state
                .commands_executed
                .contains(&"show-full-ui".to_string()));
        }

        let (backend, repo, _tmp) = setup_with_backup();
        {
            let mut state = backend.state.lock().unwrap();
            state.ui = UiStatus::Open;
        }
        reset_waydroid(&backend, &repo);
        let state = backend.state.lock().unwrap();
        assert!(state
            .commands_executed
            .contains(&"show-full-ui".to_string()));
    }

    #[test]
    fn reset_fails_without_backup() {
        let backend = FakeWaydroidBackend::new();
        let tmp = TempDir::new().unwrap();
        let repo = ProfileRepository::with_config_dir(tmp.path().to_path_buf());

        let result = reset_waydroid(&backend, &repo);
        assert!(!result.success);
        assert_eq!(result.error.as_deref(), Some("NoBackup"));
    }
}