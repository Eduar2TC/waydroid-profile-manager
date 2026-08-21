
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

    // Restore rotation settings if available
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

    // Open UI
    let _ = backend.show_full_ui();

    // Clear active profile
    let _ = repository.save_state(&AppState {
        active_profile_id: None,
        backup: None,
    });

    ResetResult {
        success: true,
        message: "Configuration restored".to_string(),
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
                width: None,
                height: None,
                density: None,
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

        // Verify rotation was restored
        let state = backend.state.lock().unwrap();
        assert!(state
            .commands_executed
            .iter()
            .any(|c| c.contains("accelerometer_rotation")));
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
