
use crate::domain::waydroid_status::SessionStatus;
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

    // Close UI
    let _ = backend.close_full_ui();

    let mut restored: Vec<&str> = Vec::new();

    // 1. Restore resolution and density props (host window config).
    // `waydroid prop set` requires the session to be running.
    if let Some(ref width) = backup.width {
        if backend.set_prop("persist.waydroid.width", width).is_ok() {
            restored.push("width");
        }
    }
    if let Some(ref height) = backup.height {
        if backend
            .set_prop("persist.waydroid.height", height)
            .is_ok()
        {
            restored.push("height");
        }
    }
    if let Some(ref density) = backup.density {
        if backend
            .set_prop("persist.waydroid.density", density)
            .is_ok()
        {
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

    // 3. Restore internal Android display resolution via wm size/density.
    if let Some(ref width) = backup.width {
        if let Some(ref height) = backup.height {
            let _ = backend.shell_command(&["wm", "size", &format!("{}x{}", width, height)]);
        }
    }
    if let Some(ref density) = backup.density {
        let _ = backend.shell_command(&["wm", "density", density]);
    }

    // 4. Restore rotation settings if available
    if let Some(ref accel) = backup.accelerometer_rotation {
        let _ = backend.shell_command(&[
            "cmd",
            "settings",
            "put",
            "system",
            "accelerometer_rotation",
            accel,
        ]);
    }

    if let Some(ref rotation) = backup.user_rotation {
        let _ = backend.shell_command(&[
            "cmd",
            "settings",
            "put",
            "system",
            "user_rotation",
            rotation,
        ]);
    }

    // 5. Open UI if it was open before
    let _ = backend.show_full_ui();

    // Clear active profile
    let _ = repository.save_state(&AppState {
        active_profile_id: None,
        backup: None,
    });

    let summary = if restored.is_empty() {
        "Rotation settings restored".to_string()
    } else {
        format!("Restored: {}", restored.join(", "))
    };

    ResetResult {
        success: true,
        message: summary,
        error: None,
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
        // Rotation was restored
        assert!(state
            .commands_executed
            .iter()
            .any(|c| c.contains("accelerometer_rotation")));
        // State cleared
        drop(state);
        let final_state = repo.load_state().unwrap();
        assert!(final_state.active_profile_id.is_none());
        assert!(final_state.backup.is_none());
    }

    #[test]
    fn reset_skips_missing_props() {
        let (backend, repo, _tmp) = setup_with_backup();
        // Remove width/height/density to simulate old backup without them
        {
            let mut st = repo.load_state().unwrap();
            if let Some(backup) = st.backup.as_mut() {
                backup.width = None;
                backup.height = None;
                backup.density = None;
            }
            repo.save_state(&st).unwrap();
        }
        let result = reset_waydroid(&backend, &repo);
        assert!(result.success);
        assert_eq!(result.message, "Rotation settings restored");
        let state = backend.state.lock().unwrap();
        assert!(!state.commands_executed.iter().any(|c| c.contains("wm size")));
    }

    #[test]
    fn reset_restarts_running_session() {
        let (backend, repo, _tmp) = setup_with_backup();
        let result = reset_waydroid(&backend, &repo);
        assert!(result.success);
        let state = backend.state.lock().unwrap();
        assert!(state.commands_executed.contains(&"container stop graceful".to_string()));
        assert!(state.commands_executed.contains(&"container start".to_string()));
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
