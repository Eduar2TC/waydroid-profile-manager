use std::sync::{Arc, Mutex};

use crate::domain::configuration_calculator::calculate_configuration;
use crate::domain::orientation::Orientation;
use crate::domain::profile::Profile;
use crate::domain::waydroid_status::OperationProgress;
use crate::infrastructure::storage::profiles::{AppState, ProfileRepository, WaydroidBackup};
use crate::infrastructure::waydroid::backend_trait::WaydroidBackend;

/// Result of an apply operation.
#[derive(Debug, Clone)]
pub struct ApplyResult {
    pub success: bool,
    pub message: String,
    pub error: Option<String>,
    /// Whether a session restart is needed for full effect.
    pub needs_restart: bool,
}

/// Apply a profile to Waydroid.
///
/// Phase 1 (no root): Close UI → Set props → Open UI → Rotation
/// Phase 2 (if needs_restart): User triggers session restart via elevation
pub fn apply_profile(
    backend: &dyn WaydroidBackend,
    repository: &ProfileRepository,
    profile: &Profile,
    orientation: Orientation,
    progress_callback: Option<Arc<Mutex<dyn FnMut(OperationProgress) + Send>>>,
) -> ApplyResult {
    let mut log_entries: Vec<String> = Vec::new();

    let mut emit_progress = |step: &str, message: &str, completed: bool| {
        log_entries.push(format!("[{}] {}", step, message));
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
            needs_restart: false,
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
    let backup = save_backup(backend, repository);
    if let Err(e) = backup {
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

    // Step 5: Set properties (persist for next session start)
    emit_progress("Configuring", "Setting width", false);
    if let Err(e) = backend.set_prop("persist.waydroid.width", &config.width.to_string()) {
        return ApplyResult {
            success: false,
            message: format!("Failed to set width: {}", e),
            error: Some("ConfigurationFailed".to_string()),
            needs_restart: false,
        };
    }
    emit_progress("Configuring", &format!("Width = {}", config.width), true);

    emit_progress("Configuring", "Setting height", false);
    if let Err(e) = backend.set_prop("persist.waydroid.height", &config.height.to_string()) {
        return ApplyResult {
            success: false,
            message: format!("Failed to set height: {}", e),
            error: Some("ConfigurationFailed".to_string()),
            needs_restart: false,
        };
    }
    emit_progress("Configuring", &format!("Height = {}", config.height), true);

    emit_progress("Configuring", "Setting density", false);
    if let Err(e) = backend.set_prop("persist.waydroid.density", &config.density.to_string()) {
        return ApplyResult {
            success: false,
            message: format!("Failed to set density: {}", e),
            error: Some("ConfigurationFailed".to_string()),
            needs_restart: false,
        };
    }
    emit_progress("Configuring", &format!("Density = {}", config.density), true);

    // Step 6: Ensure session is running (start if not)
    emit_progress("StartingSession", "Checking session", false);
    if backend.get_session_status() != crate::domain::waydroid_status::SessionStatus::Running {
        emit_progress("StartingSession", "Starting session", false);
        if let Err(e) = backend.session_start() {
            return ApplyResult {
                success: false,
                message: format!("Failed to start session: {}", e),
                error: Some("SessionStartFailed".to_string()),
                needs_restart: false,
            };
        }
        if !backend.wait_for_session(30) {
            return ApplyResult {
                success: false,
                message: "Session start timed out".to_string(),
                error: Some("Timeout".to_string()),
                needs_restart: false,
            };
        }
    }
    std::thread::sleep(std::time::Duration::from_secs(2));
    emit_progress("StartingSession", "Session running", true);

    // Step 6b: Apply resolution, density and rotation BEFORE opening UI
    // This way the window is created with the correct size and orientation
    let size_str = format!("{}x{}", config.width, config.height);
    emit_progress("ApplyingDisplay", &format!("wm size {}", size_str), false);
    if let Err(e) = backend.shell_command(&["wm", "size", &size_str]) {
        log::warn!("Failed to set wm size: {}", e);
    }

    let density_str = config.density.to_string();
    emit_progress("ApplyingDisplay", &format!("wm density {}", density_str), false);
    if let Err(e) = backend.shell_command(&["wm", "density", &density_str]) {
        log::warn!("Failed to set wm density: {}", e);
    }

    // Apply rotation before opening UI so window is created with correct orientation
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
        "cmd", "settings", "put", "system", "user_rotation", &config.rotation.to_string(),
    ]) {
        log::warn!("Failed to set rotation: {}", e);
    }
    emit_progress("ApplyingDisplay", "Display and rotation configured", true);

    // Wait for display changes to settle
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Step 7: Open UI (window will be created with new resolution and orientation)
    emit_progress("OpeningUi", "Opening Waydroid UI", false);
    if let Err(e) = backend.show_full_ui() {
        return ApplyResult {
            success: false,
            message: format!("Failed to open UI: {}", e),
            error: Some("UiOpenFailed".to_string()),
            needs_restart: false,
        };
    }
    emit_progress("WaitingForUi", "Waiting for UI to be ready", false);
    if !backend.wait_for_ui(15) {
        return ApplyResult {
            success: false,
            message: "UI open timed out".to_string(),
            error: Some("Timeout".to_string()),
            needs_restart: false,
        };
    }
    emit_progress("OpeningUi", "UI opened", true);

    // Step 9: Verify
    emit_progress("Verifying", "Verifying configuration", false);
    if let Err(e) = verify_configuration(backend) {
        emit_progress("Verifying", &format!("Warning: {}", e), true);
    } else {
        emit_progress("Verifying", "Configuration verified", true);
    }

    // Step 10: Update active profile in state
    let _ = repository.save_state(&AppState {
        active_profile_id: Some(profile.id.clone()),
        backup: None,
    });

    ApplyResult {
        success: true,
        message: format!("Profile '{}' applied successfully", profile.name),
        error: None,
        needs_restart: false,
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

/// Verify the applied configuration.
fn verify_configuration(backend: &dyn WaydroidBackend) -> anyhow::Result<()> {
    if backend.get_session_status() != crate::domain::waydroid_status::SessionStatus::Running {
        return Err(anyhow::anyhow!("Session not running after apply"));
    }
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
        assert!(!result.needs_restart); // Changes applied live via wm commands
    }

    #[test]
    fn apply_profile_landscape() {
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
        assert!(result.success, "Should start session and apply: {}", result.message);
        let state = backend.state.lock().unwrap();
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
}
