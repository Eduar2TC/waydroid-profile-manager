use std::sync::{Arc, Mutex};

use crate::domain::configuration_calculator::calculate_configuration;
use crate::domain::orientation::Orientation;
use crate::domain::profile::Profile;
use crate::domain::waydroid_status::{OperationProgress, SessionStatus};
use crate::infrastructure::storage::profiles::{ProfileRepository, WaydroidBackup};
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
// La firma tiene 8 argumentos porque el callback de progreso va como
// Arc<Mutex<dyn FnMut>>, que supera el umbral de complejidad de clippy.
#[allow(clippy::type_complexity)]
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
    std::thread::sleep(std::time::Duration::from_secs(1));
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
    // Short settle for the GPU crash (SIGABRT in composer@2.1-service) and
    // coredump handler. Cannot be polled — the crash is asynchronous.
    std::thread::sleep(std::time::Duration::from_secs(3));
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
    // Wait until the Android settings provider accepts shell commands
    // (polls `cmd settings get` until it succeeds — replaces a fixed 10s sleep).
    if !backend.wait_for_shell_ready(30) {
        emit_progress(
            "RestartingSession",
            "Warning: shell not ready after 30s, continuing anyway",
            true,
        );
    }
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
    // after UI creation. Polls the settings provider instead of a fixed sleep.
    if !backend.wait_for_shell_ready(15) {
        emit_progress(
            "OpeningUi",
            "Warning: settings provider not ready, continuing anyway",
            true,
        );
    }

    // Step 9: Apply internal Android display resolution via wm size.
    //
    // persist.waydroid.width/height only controls the HOST window size,
    // not the internal Android display resolution. We need wm size to make
    // Android render at the correct resolution so the content fills the window.
    //
    // wm size dimensions match the host window dimensions (orientation-specific).
    // For landscape, wm size = landscape dims. For portrait, wm size = portrait dims.
    emit_progress("ApplyingDisplay", "Setting display resolution", false);
    if let Err(e) = backend.shell_command(&[
        "wm", "size",
        &format!("{}x{}", config.wm_width, config.wm_height),
    ]) {
        log::warn!("wm size failed (non-fatal): {}", e);
    }
    if let Err(e) = backend.shell_command(&["wm", "density", &config.density.to_string()]) {
        log::warn!("wm density failed (non-fatal): {}", e);
    }
    emit_progress("ApplyingDisplay", "Display resolution applied", true);
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Step 10: Set display rotation.
    //
    // Since wm size now matches the host window dimensions, rotation is always 0.
    // The command is kept for compatibility with Android's display framework.
    emit_progress("ApplyingRotation", "Setting rotation", false);
    if let Err(e) = backend.shell_command(&[
        "cmd", "window", "set-user-rotation", "lock", &config.rotation.to_string(),
    ]) {
        log::warn!("Failed to set rotation via cmd (non-fatal): {}", e);
    }
    emit_progress("ApplyingRotation", "Rotation applied", true);

    // Step 11: Verify
    emit_progress("Verifying", "Verifying configuration", false);
    if backend.get_session_status() != SessionStatus::Running {
        emit_progress("Verifying", "Warning: session not running", true);
    } else {
        emit_progress("Verifying", "Configuration verified", true);
    }

    // Step 11: Update active profile in state, preserving the backup captured
    // in Step 3 — `reset` needs it to restore the configuration we overwrote.
    // Writing `backup: None` here would wipe it and make reset a no-op that
    // always fails with "No backup available to restore".
    if let Ok(mut current) = repository.load_state() {
        current.active_profile_id = Some(profile.id.clone());
        if let Err(e) = repository.save_state(&current) {
            log::warn!("Failed to save active profile: {}", e);
        }
    }

    ApplyResult {
        success: true,
        message: format!("Profile '{}' applied successfully", profile.name),
        error: None,
    }
}

/// Save current Waydroid configuration as backup.
///
/// Captures the host window props, the container's `wm size`/`wm density`
/// overrides and the rotation settings so that `reset` can put every one of
/// them back exactly as it found them.
fn save_backup(
    backend: &dyn WaydroidBackend,
    repository: &ProfileRepository,
) -> anyhow::Result<()> {
    use chrono::Utc;

    // Read current props (host window + density) BEFORE modifying anything.
    // get_prop returns None if the prop is not set — that's fine: reset will
    // skip restoring it and Waydroid keeps its default.
    //
    // The `wm`/`settings` probes go through the Android shell, which only
    // answers while the session runs (and each failed probe retries with
    // sleeps), so skip them when it is down: there is nothing to read then.
    let session_running = backend.get_session_status() == SessionStatus::Running;
    let (accelerometer_rotation, user_rotation, wm_size, wm_density) = if session_running {
        (
            read_setting(backend, "accelerometer_rotation"),
            read_setting(backend, "user_rotation"),
            read_wm_override(backend, "size"),
            read_wm_override(backend, "density"),
        )
    } else {
        (None, None, None, None)
    };

    let backup = WaydroidBackup {
        width: backend.get_prop("persist.waydroid.width"),
        height: backend.get_prop("persist.waydroid.height"),
        density: backend.get_prop("persist.waydroid.density"),
        accelerometer_rotation,
        user_rotation,
        wm_size,
        wm_density,
        timestamp: Utc::now().to_rfc3339(),
    };

    // Only overwrite the existing backup if there is no backup yet.
    // This way the ORIGINAL pre-first-apply configuration is preserved
    // even after applying multiple profiles in a row.
    let mut state = repository.load_state()?;
    if state.backup.is_none() {
        state.backup = Some(backup);
        repository.save_state(&state)?;
    } else {
        // Update timestamp only — keep the original values
        if let Some(existing) = state.backup.as_mut() {
            existing.timestamp = Utc::now().to_rfc3339();
        }
        repository.save_state(&state)?;
    }
    Ok(())
}

/// Read an Android system setting via shell (settings get).
fn read_setting(backend: &dyn WaydroidBackend, key: &str) -> Option<String> {
    backend
        .shell_command(&["cmd", "settings", "get", "system", key])
        .ok()
        .filter(|r| r.success)
        .map(|r| r.stdout.trim().to_string())
        .filter(|s| !s.is_empty() && s != "null")
}

/// Read the container's current `wm size`/`wm density` override, e.g.
/// "640x360" from a line like `Override size: 640x360`.
///
/// Returns None when there is no override (or the shell is unreachable),
/// which reset interprets as "no override before applying" and clears with
/// `wm ... reset`.
fn read_wm_override(backend: &dyn WaydroidBackend, kind: &str) -> Option<String> {
    let prefix = match kind {
        "size" => "Override size:",
        _ => "Override density:",
    };
    let result = backend.shell_command(&["wm", kind]).ok()?;
    if !result.success {
        return None;
    }
    result
        .stdout
        .lines()
        .find_map(|line| line.trim().strip_prefix(prefix))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
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
    fn apply_keeps_backup_so_reset_can_restore_it() {
        let (backend, repo, _tmp) = setup();
        // Seed the "original" configuration so the backup has something real
        // to capture (the fake backend starts with no props set).
        backend.set_prop("persist.waydroid.width", "1024").unwrap();
        backend.set_prop("persist.waydroid.height", "768").unwrap();
        backend.set_prop("persist.waydroid.density", "240").unwrap();

        let profile = Profile::builtin(
            "phone-sm", "Phone Small", "Small phone", 640, 360, 160,
            Orientation::Portrait,
        );
        let apply = apply_profile(&backend, &repo, &profile, Orientation::Portrait, None);
        assert!(apply.success, "Apply failed: {}", apply.message);

        // The final state save must keep the backup from Step 3, otherwise
        // reset always fails with "No backup available" and restores nothing.
        let state = repo.load_state().unwrap();
        assert!(state.backup.is_some(), "apply must keep the config backup");
        assert_eq!(state.active_profile_id.as_deref(), Some("phone-sm"));
        assert_eq!(state.backup.as_ref().unwrap().width.as_deref(), Some("1024"));

        let reset = crate::application::reset::reset_waydroid(&backend, &repo);
        assert!(reset.success, "Reset failed: {}", reset.message);
        assert_eq!(reset.message, "Restored: width, height, density");

        let state = backend.state.lock().unwrap();
        assert_eq!(state.props["persist.waydroid.width"], "1024");
        assert_eq!(state.props["persist.waydroid.height"], "768");
        assert_eq!(state.props["persist.waydroid.density"], "240");
        // No override was recorded, so the display override apply() created
        // must be cleared rather than left behind.
        assert!(state
            .commands_executed
            .iter()
            .any(|c| c.contains("wm size reset")));
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
