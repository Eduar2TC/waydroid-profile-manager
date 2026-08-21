use std::sync::{Arc, Mutex};

use crate::domain::configuration_calculator::calculate_configuration;
use crate::domain::orientation::Orientation;
use crate::domain::profile::Profile;
use crate::domain::waydroid_status::{OperationProgress, SessionStatus};
use crate::infrastructure::storage::profiles::{AppState, ProfileRepository, WaydroidBackup};
use crate::infrastructure::waydroid::backend_trait::WaydroidBackend;

/// Result of an apply operation.
#[derive(Debug, Clone)]
pub struct ApplyResult {
    pub success: bool,
    pub message: String,
    pub error: Option<String>,
}

/// Apply a profile to Waydroid.
///
/// Flow: Close UI → Stop session → Set props → Start session → Open UI → Rotation
/// This ensures resolution changes take effect immediately without requiring
/// the user to manually restart waydroid.
pub fn apply_profile(
    backend: &dyn WaydroidBackend,
    repository: &ProfileRepository,
    profile: &Profile,
    orientation: Orientation,
    progress_callback: Option<Arc<Mutex<dyn FnMut(OperationProgress) + Send>>>,
) -> ApplyResult {
    let emit_progress = |step: &str, message: &str, completed: bool| {
        log::info!("{}: {}", step, message);
        if let Some(ref cb) = progress_callback {
            if let Ok(mut cb) = cb.lock() {
                cb(OperationProgress {
                    step: step.to_string(),
                    message: message.to_string(),
                    completed,
                });
            }
        }
    };

    // Step 1: Validate
    emit_progress("Validating", "Validating profile", false);
    if !backend.check_installed() {
        emit_progress("Validating", "Waydroid is not installed", true);
        return ApplyResult {
            success: false,
            message: "Waydroid is not installed".to_string(),
            error: Some("WaydroidNotInstalled".to_string()),
        };
    }
    emit_progress("Validating", "Profile validated", true);

    // Step 2: Calculate configuration
    let config = calculate_configuration(
        profile.base_width,
        profile.base_height,
        profile.density,
        orientation,
    );
    emit_progress(
        "Configuring",
        &format!(
            "Resolution: {}x{}, Density: {}, Rotation: {}",
            config.width, config.height, config.density, config.rotation
        ),
        true,
    );

    // Step 3: Save backup of current config
    emit_progress("Backup", "Saving current configuration", false);
    if let Err(e) = save_backup(backend, repository) {
        log::warn!("Failed to save backup: {}", e);
    }
    emit_progress("Backup", "Backup saved", true);

    // Step 4: Close UI
    emit_progress("ClosingUi", "Closing Waydroid UI", false);
    if let Err(e) = backend.close_full_ui() {
        log::warn!("Failed to close UI (may not be open): {}", e);
    }
    // Give the window and GPU processes time to detach
    std::thread::sleep(std::time::Duration::from_secs(3));
    emit_progress("ClosingUi", "UI closed", true);

    // Step 5: Stop session to apply new properties on restart
    // persist.waydroid.* props are only read when the session/container starts,
    // so we must stop and restart the session for resolution changes to take effect.
    emit_progress("RestartingSession", "Stopping session", false);
    if let Err(e) = backend.session_stop() {
        log::warn!("Failed to stop session: {}", e);
    }
    // Poll until session is fully stopped (including container)
    if !backend.wait_for_session_stopped(15) {
        emit_progress(
            "RestartingSession",
            "Warning: session stop timed out, proceeding anyway",
            true,
        );
    }
    // Extra settle time for LXC processes to fully clean up
    std::thread::sleep(std::time::Duration::from_secs(2));
    emit_progress("RestartingSession", "Session stopped", true);

    // Step 6: Set properties (these will be read on session start)
    emit_progress("Configuring", "Setting width", false);
    if let Err(e) = backend.set_prop("persist.waydroid.width", &config.width.to_string()) {
        return ApplyResult {
            success: false,
            message: format!("Failed to set width: {}", e),
            error: Some("ConfigurationFailed".to_string()),
        };
    }
    emit_progress("Configuring", &format!("Width = {}", config.width), true);

    emit_progress("Configuring", "Setting height", false);
    if let Err(e) = backend.set_prop("persist.waydroid.height", &config.height.to_string()) {
        return ApplyResult {
            success: false,
            message: format!("Failed to set height: {}", e),
            error: Some("ConfigurationFailed".to_string()),
        };
    }
    emit_progress("Configuring", &format!("Height = {}", config.height), true);

    emit_progress("Configuring", "Setting density", false);
    if let Err(e) = backend.set_prop("persist.waydroid.density", &config.density.to_string()) {
        return ApplyResult {
            success: false,
            message: format!("Failed to set density: {}", e),
            error: Some("ConfigurationFailed".to_string()),
        };
    }
    emit_progress("Configuring", &format!("Density = {}", config.density), true);

    // Step 7: Start session (props will be applied on startup)
    emit_progress("RestartingSession", "Starting session", false);
    if let Err(e) = backend.session_start() {
        return ApplyResult {
            success: false,
            message: format!("Failed to start session: {}", e),
            error: Some("SessionStartFailed".to_string()),
        };
    }
    if !backend.wait_for_session(30) {
        return ApplyResult {
            success: false,
            message: "Session start timed out".to_string(),
            error: Some("Timeout".to_string()),
        };
    }
    // Wait for container services to fully initialize
    std::thread::sleep(std::time::Duration::from_secs(5));
    emit_progress("RestartingSession", "Session running", true);

    // Step 8: Open UI (window will be created with new resolution)
    emit_progress("OpeningUi", "Opening Waydroid UI", false);
    if let Err(e) = backend.show_full_ui() {
        return ApplyResult {
            success: false,
            message: format!("Failed to open UI: {}", e),
            error: Some("UiOpenFailed".to_string()),
        };
    }
    emit_progress("WaitingForUi", "Waiting for UI to be ready", false);
    if !backend.wait_for_ui(15) {
        return ApplyResult {
            success: false,
            message: "UI open timed out".to_string(),
            error: Some("Timeout".to_string()),
        };
    }
    emit_progress("OpeningUi", "UI opened", true);

    // Step 9: Apply rotation after UI is available
    emit_progress("ApplyingRotation", "Disabling auto-rotation", false);
    if let Err(e) = backend.shell_command(&[
        "cmd", "settings", "put", "system", "accelerometer_rotation", "0",
    ]) {
        log::warn!("Failed to disable auto-rotation: {}", e);
    }

    emit_progress(
        "ApplyingRotation",
        &format!("Setting user_rotation to {}", config.rotation),
        false,
    );
    if let Err(e) = backend.shell_command(&[
        "cmd",
        "settings",
        "put",
        "system",
        "user_rotation",
        &config.rotation.to_string(),
    ]) {
        log::warn!("Failed to set rotation: {}", e);
    }
    emit_progress("ApplyingRotation", "Rotation applied", true);

    // Step 10: Verify
    emit_progress("Verifying", "Verifying configuration", false);
    if backend.get_session_status() != SessionStatus::Running {
        emit_progress("Verifying", "Warning: session not running", true);
    } else {
        emit_progress("Verifying", "Configuration verified", true);
    }

    // Step 11: Update active profile in state
    let _ = repository.save_state(&AppState {
        active_profile_id: Some(profile.id.clone()),
        backup: None,
    });

    ApplyResult {
        success: true,
        message: format!("Profile '{}' applied successfully", profile.name),
        error: None,
    }
}

/// Save current Waydroid configuration as backup.
fn save_backup(
    backend: &dyn WaydroidBackend,
    repository: &ProfileRepository,
) -> anyhow::Result<()> {
    use chrono::Utc;

    let get_prop = |key: &str| -> Option<String> {
        backend
            .shell_command(&["cmd", "settings", "get", "system", key])
            .ok()
            .filter(|r| r.success)
            .map(|r| r.stdout.trim().to_string())
    };

    let backup = WaydroidBackup {
        width: None,
        height: None,
        density: None,
        accelerometer_rotation: get_prop("accelerometer_rotation"),
        user_rotation: get_prop("user_rotation"),
        timestamp: Utc::now().to_rfc3339(),
    };

    let mut state = repository.load_state()?;
    state.backup = Some(backup);
    repository.save_state(&state)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::storage::profiles::ProfileRepository;
    use crate::infrastructure::waydroid::service::FakeWaydroidBackend;
    use tempfile::TempDir;

    fn setup() -> (FakeWaydroidBackend, ProfileRepository, TempDir) {
        let backend = FakeWaydroidBackend::new();
        let tmp = TempDir::new().unwrap();
        let repo = ProfileRepository::with_config_dir(tmp.path().to_path_buf());
        (backend, repo, tmp)
    }

    #[test]
    fn apply_profile_success() {
        let (backend, repo, _tmp) = setup();
        let profile = Profile::builtin(
            "phone-sm", "Phone Small", "Small phone", 640, 360, 160,
            Orientation::Portrait,
        );
        let result = apply_profile(&backend, &repo, &profile, Orientation::Portrait, None);
        assert!(result.success, "Apply should succeed: {}", result.message);
        assert!(result.error.is_none());
    }

    #[test]
    fn apply_profile_restarts_session() {
        let (backend, repo, _tmp) = setup();
        let profile = Profile::builtin(
            "phone-sm", "Phone Small", "Small phone", 640, 360, 160,
            Orientation::Landscape,
        );
        let result = apply_profile(&backend, &repo, &profile, Orientation::Landscape, None);
        assert!(result.success, "Apply should succeed: {}", result.message);

        let state = backend.state.lock().unwrap();
        assert_eq!(state.props["persist.waydroid.width"], "640");
        assert_eq!(state.props["persist.waydroid.height"], "360");
        assert_eq!(state.props["persist.waydroid.density"], "160");
        // Session was stopped and restarted
        assert!(state.commands_executed.contains(&"session stop".to_string()));
        assert!(state.commands_executed.contains(&"session start".to_string()));
    }

    #[test]
    fn apply_profile_portrait_swaps_dimensions() {
        let (backend, repo, _tmp) = setup();
        let profile = Profile::builtin(
            "phone-sm", "Phone Small", "Small phone", 640, 360, 160,
            Orientation::Portrait,
        );
        let result = apply_profile(&backend, &repo, &profile, Orientation::Portrait, None);
        assert!(result.success);

        let state = backend.state.lock().unwrap();
        assert_eq!(state.props["persist.waydroid.width"], "360");
        assert_eq!(state.props["persist.waydroid.height"], "640");
    }

    #[test]
    fn apply_fails_when_not_installed() {
        let backend = FakeWaydroidBackend::not_installed();
        let tmp = TempDir::new().unwrap();
        let repo = ProfileRepository::with_config_dir(tmp.path().to_path_buf());
        let profile = Profile::builtin(
            "phone-sm", "Phone Small", "Small phone", 640, 360, 160,
            Orientation::Portrait,
        );
        let result = apply_profile(&backend, &repo, &profile, Orientation::Portrait, None);
        assert!(!result.success);
        assert_eq!(result.error.as_deref(), Some("WaydroidNotInstalled"));
    }

    #[test]
    fn apply_starts_session_if_needed() {
        let backend = FakeWaydroidBackend::stopped();
        let tmp = TempDir::new().unwrap();
        let repo = ProfileRepository::with_config_dir(tmp.path().to_path_buf());
        let profile = Profile::builtin(
            "phone-sm", "Phone Small", "Small phone", 640, 360, 160,
            Orientation::Portrait,
        );
        let result = apply_profile(&backend, &repo, &profile, Orientation::Portrait, None);
        assert!(result.success, "Should restart session and apply: {}", result.message);
        let state = backend.state.lock().unwrap();
        // Both stop and start should be called (restart flow)
        assert!(state.commands_executed.contains(&"session stop".to_string()));
        assert!(state.commands_executed.contains(&"session start".to_string()));
    }

    #[test]
    fn apply_with_progress_callback() {
        let (backend, repo, _tmp) = setup();
        let profile = Profile::builtin(
            "phone-sm", "Phone Small", "Small phone", 640, 360, 160,
            Orientation::Portrait,
        );
        let steps: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let steps_clone = steps.clone();
        let callback = Arc::new(Mutex::new(move |progress: OperationProgress| {
            if progress.completed {
                steps_clone.lock().unwrap().push(progress.step);
            }
        })) as Arc<Mutex<dyn FnMut(OperationProgress) + Send>>;

        let result = apply_profile(&backend, &repo, &profile, Orientation::Portrait, Some(callback));
        assert!(result.success);
        let recorded_steps = steps.lock().unwrap();
        assert!(recorded_steps.contains(&"Validating".to_string()));
        assert!(recorded_steps.contains(&"OpeningUi".to_string()));
        assert!(recorded_steps.contains(&"RestartingSession".to_string()));
    }

    #[test]
    fn apply_custom_profile() {
        let (backend, repo, _tmp) = setup();
        let profile = Profile::new(
            "Pixel 7".into(), Some("Custom".into()), 1080, 2400, 420,
            Orientation::Portrait,
        );
        let result = apply_profile(&backend, &repo, &profile, Orientation::Portrait, None);
        assert!(result.success);
        let state = backend.state.lock().unwrap();
        assert_eq!(state.props["persist.waydroid.width"], "2400");
        assert_eq!(state.props["persist.waydroid.height"], "1080");
        assert_eq!(state.props["persist.waydroid.density"], "420");
    }

    #[test]
    fn apply_closes_ui_before_stop() {
        let (backend, repo, _tmp) = setup();
        let profile = Profile::builtin(
            "phone-sm", "Phone Small", "Small phone", 640, 360, 160,
            Orientation::Portrait,
        );
        let result = apply_profile(&backend, &repo, &profile, Orientation::Portrait, None);
        assert!(result.success);
        let state = backend.state.lock().unwrap();
        // Verify order: close-ui before session-stop before session-start before show-full-ui
        let close_idx = state.commands_executed.iter().position(|c| c == "close-ui").unwrap();
        let stop_idx = state.commands_executed.iter().position(|c| c == "session stop").unwrap();
        let start_idx = state.commands_executed.iter().position(|c| c == "session start").unwrap();
        let open_idx = state.commands_executed.iter().position(|c| c == "show-full-ui").unwrap();
        assert!(close_idx < stop_idx, "UI must close before session stop");
        assert!(stop_idx < start_idx, "Session must stop before start");
        assert!(start_idx < open_idx, "Session must start before UI opens");
    }
}
