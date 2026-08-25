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
/// Flow: Close UI → Set props (while running!) → Stop session → Start session → Open UI → Rotation
///
/// CRITICAL: `waydroid prop set` only works when the session is RUNNING.
/// Props must be set BEFORE stopping, so they're saved to the config file.
/// The new session then reads the updated config on startup.
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
    std::thread::sleep(std::time::Duration::from_secs(2));
    emit_progress("ClosingUi", "UI closed", true);

    // Step 5: Set properties WHILE SESSION IS STILL RUNNING.
    // `waydroid prop set` communicates with the container manager via D-Bus,
    // which requires the session to be active. The props are persisted to
    // the config file and will be read when the session restarts.
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

    // Step 6: Stop session.
    // `waydroid session stop` calls lxc-stop -k internally, which triggers a
    // SIGABRT in composer@2.1-service (Mesa/libEGL can't release GPU state).
    // This crash is inherent to Waydroid's architecture and happens even with
    // manual restarts. It is NOT fatal — waydroid recovers on next start.
    emit_progress("RestartingSession", "Stopping session", false);
    if let Err(e) = backend.container_stop_graceful(10) {
        log::warn!("Failed to stop session: {}", e);
    }
    // Poll until session/container is fully stopped
    if !backend.wait_for_session_stopped(15) {
        emit_progress(
            "RestartingSession",
            "Warning: stop timed out, proceeding anyway",
            true,
        );
    }
    // Wait for GPU crash to settle — Mesa/libEGL SIGABRT + coredump handler
    std::thread::sleep(std::time::Duration::from_secs(8));
    emit_progress("RestartingSession", "Session stopped", true);

    // Step 7: Start session (reads the new props from config on startup)
    emit_progress("RestartingSession", "Starting session", false);
    if let Err(e) = backend.container_start() {
        return ApplyResult {
            success: false,
            message: format!("Failed to start session: {} ", e),
            error: Some("SessionStartFailed".to_string()),
        };
    }
    // Poll until session is fully running (init starts all Android services)
    if !backend.wait_for_session(45) {
        return ApplyResult {
            success: false,
            message: "Session start timed out".to_string(),
            error: Some("Timeout".to_string()),
        };
    }
    // Wait for Android services (surfaceflinger, composer, platform, settings provider)
    // to fully initialize. After a nohup session start, the settings provider
    // inside the container may take extra time to become available via D-Bus.
    std::thread::sleep(std::time::Duration::from_secs(10));
    emit_progress("RestartingSession", "Session running", true);

    // Step 8: Open UI (window is created with new resolution from props)
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

    // Wait for the platform service and settings provider to settle
    // after UI creation. The settings provider must be ready for
    // `cmd settings put` to actually apply rotation.
    std::thread::sleep(std::time::Duration::from_secs(3));

    // Step 9: Apply internal Android display resolution via wm size.
    //
    // persist.waydroid.width/height only controls the HOST window size,
    // not the internal Android display resolution. We need wm size to make
    // Android render at the correct resolution so the content fills the window.
    //
    // Always use portrait dimensions for wm size — when user_rotation=1,
    // Android rotates the display 90°, so a portrait wm size becomes landscape.
    let wm_w = std::cmp::min(config.width, config.height);
    let wm_h = std::cmp::max(config.width, config.height);
    emit_progress("ApplyingDisplay", "Setting display resolution", false);
    if let Err(e) = backend.shell_command(&["wm", "size", "reset"]) {
        eprintln!("wm size reset failed (non-fatal): {}", e);
    }
    std::thread::sleep(std::time::Duration::from_secs(1));
    if let Err(e) = backend.shell_command(&[
        "wm", "size",
        &format!("{}x{}", wm_w, wm_h),
    ]) {
        eprintln!("wm size failed (non-fatal): {}", e);
    }
    if let Err(e) = backend.shell_command(&["wm", "density", "reset"]) {
        eprintln!("wm density reset failed (non-fatal): {}", e);
    }
    std::thread::sleep(std::time::Duration::from_secs(1));
    if let Err(e) = backend.shell_command(&["wm", "density", &config.density.to_string()]) {
        eprintln!("wm density failed (non-fatal): {}", e);
    }
    emit_progress("ApplyingDisplay", "Display resolution applied", true);
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Step 10: Apply rotation AFTER UI is available and display resolution is set.
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

    // Step 11: Verify
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
        // Container was stopped gracefully and restarted
        assert!(state.commands_executed.contains(&"container stop graceful".to_string()));
        assert!(state.commands_executed.contains(&"container start".to_string()));
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
        assert!(result.success, "Should restart container and apply: {}", result.message);
        let state = backend.state.lock().unwrap();
        // Both graceful stop and start should be called
        assert!(state.commands_executed.contains(&"container stop graceful".to_string()));
        assert!(state.commands_executed.contains(&"container start".to_string()));
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
    fn apply_sets_props_before_stop() {
        let (backend, repo, _tmp) = setup();
        let profile = Profile::builtin(
            "phone-sm", "Phone Small", "Small phone", 640, 360, 160,
            Orientation::Portrait,
        );
        let result = apply_profile(&backend, &repo, &profile, Orientation::Portrait, None);
        assert!(result.success);
        let state = backend.state.lock().unwrap();
        // Props must be set BEFORE container stop (prop set requires running session)
        let prop_idx = state.commands_executed.iter().position(|c| c.starts_with("prop set")).unwrap();
        let stop_idx = state.commands_executed.iter().position(|c| c == "container stop graceful").unwrap();
        assert!(prop_idx < stop_idx, "Props must be set before container stop");
    }

    #[test]
    fn apply_closes_ui_before_props() {
        let (backend, repo, _tmp) = setup();
        let profile = Profile::builtin(
            "phone-sm", "Phone Small", "Small phone", 640, 360, 160,
            Orientation::Portrait,
        );
        let result = apply_profile(&backend, &repo, &profile, Orientation::Portrait, None);
        assert!(result.success);
        let state = backend.state.lock().unwrap();
        // Order: close-ui → prop set → stop → start → show-full-ui
        let close_idx = state.commands_executed.iter().position(|c| c == "close-ui").unwrap();
        let prop_idx = state.commands_executed.iter().position(|c| c.starts_with("prop set")).unwrap();
        let stop_idx = state.commands_executed.iter().position(|c| c == "container stop graceful").unwrap();
        let start_idx = state.commands_executed.iter().position(|c| c == "container start").unwrap();
        let open_idx = state.commands_executed.iter().position(|c| c == "show-full-ui").unwrap();
        assert!(close_idx < prop_idx, "UI must close before setting props");
        assert!(prop_idx < stop_idx, "Props must be set before container stop");
        assert!(stop_idx < start_idx, "Container must stop before start");
        assert!(start_idx < open_idx, "Container must start before UI opens");
    }
}
