use std::sync::{Arc, Mutex};

use crate::application::apply_profile::{apply_profile, ApplyResult};
use crate::application::profile_manager::ProfileManager;
use crate::application::reset::{reset_waydroid, ResetResult};
use crate::application::status;
use crate::domain::orientation::Orientation;
use crate::domain::profile::Profile;
use crate::domain::waydroid_status::{OperationProgress, WaydroidStatus};
use crate::infrastructure::storage::profiles::ProfileRepository;
use crate::infrastructure::waydroid::backend_trait::WaydroidBackend;
use crate::infrastructure::waydroid::cli::RealWaydroidBackend;

/// Public API exposed to Flutter.
/// This is the ONLY interface Flutter should use.
pub struct WaydroidManager {
    backend: Box<dyn WaydroidBackend>,
    repository: ProfileRepository,
    profile_manager: ProfileManager,
}

impl WaydroidManager {
    /// Create a new WaydroidManager with real backend.
    pub fn new() -> Self {
        let backend = Box::new(RealWaydroidBackend::new());
        let repository = ProfileRepository::new();
        let profile_manager = ProfileManager::new(ProfileRepository::new());
        Self {
            backend,
            repository,
            profile_manager,
        }
    }

    /// Create a WaydroidManager with a custom backend (for testing).
    pub fn with_backend(
        backend: Box<dyn WaydroidBackend>,
        repository: ProfileRepository,
    ) -> Self {
        let profile_manager = ProfileManager::new(ProfileRepository::new());
        Self {
            backend,
            repository,
            profile_manager,
        }
    }

    /// Get the current Waydroid status.
    pub fn get_status(&self) -> WaydroidStatus {
        status::get_status(self.backend.as_ref())
    }

    /// Get all profiles.
    pub fn get_profiles(&self) -> Vec<Profile> {
        self.profile_manager.get_profiles().to_vec()
    }

    /// Get a profile by ID.
    pub fn get_profile(&self, id: &str) -> Option<Profile> {
        self.profile_manager.get_profile(id).cloned()
    }

    /// Get the currently active profile.
    pub fn get_active_profile(&self) -> Option<Profile> {
        status::get_active_profile(&self.repository, self.profile_manager.get_profiles())
    }

    /// Create a new profile.
    pub fn create_profile(
        &mut self,
        name: String,
        description: Option<String>,
        base_width: u32,
        base_height: u32,
        density: u32,
        default_orientation: Orientation,
    ) -> Result<Profile, String> {
        self.profile_manager
            .create_profile(
                name,
                description,
                base_width,
                base_height,
                density,
                default_orientation,
            )
            .map_err(|errors| {
                errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
    }

    /// Update an existing profile.
    pub fn update_profile(
        &mut self,
        id: String,
        name: Option<String>,
        description: Option<Option<String>>,
        base_width: Option<u32>,
        base_height: Option<u32>,
        density: Option<u32>,
        default_orientation: Option<Orientation>,
    ) -> Result<(), String> {
        self.profile_manager
            .update_profile(
                &id,
                name,
                description,
                base_width,
                base_height,
                density,
                default_orientation,
            )
            .map_err(|errors| {
                errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
    }

    /// Delete a profile by ID.
    pub fn delete_profile(&mut self, id: String) -> Result<(), String> {
        self.profile_manager
            .delete_profile(&id)
            .map_err(|e| e.to_string())
    }

    /// Duplicate a profile.
    pub fn duplicate_profile(
        &mut self,
        id: String,
        new_name: Option<String>,
    ) -> Result<Profile, String> {
        self.profile_manager
            .duplicate_profile(&id, new_name)
            .map_err(|e| e.to_string())
    }

    /// Apply a profile to Waydroid.
    pub fn apply_profile(
        &self,
        profile_id: String,
        orientation: Orientation,
    ) -> ApplyResult {
        let profile = match self.profile_manager.get_profile(&profile_id) {
            Some(p) => p,
            None => {
                return ApplyResult {
                    success: false,
                    message: "Profile not found".to_string(),
                    error: Some("ProfileNotFound".to_string()),
                }
            }
        };

        apply_profile(
            self.backend.as_ref(),
            &self.repository,
            &profile,
            orientation,
            None,
        )
    }

    /// Apply a profile with progress callback.
    pub fn apply_profile_with_progress(
        &self,
        profile_id: String,
        orientation: Orientation,
        progress_callback: Arc<Mutex<dyn FnMut(OperationProgress) + Send>>,
    ) -> ApplyResult {
        let profile = match self.profile_manager.get_profile(&profile_id) {
            Some(p) => p,
            None => {
                return ApplyResult {
                    success: false,
                    message: "Profile not found".to_string(),
                    error: Some("ProfileNotFound".to_string()),
                }
            }
        };

        apply_profile(
            self.backend.as_ref(),
            &self.repository,
            &profile,
            orientation,
            Some(progress_callback),
        )
    }

    /// Reset Waydroid configuration.
    pub fn reset(&self) -> ResetResult {
        reset_waydroid(self.backend.as_ref(), &self.repository)
    }

    /// Start the Waydroid session.
    pub fn start_session(&self) -> Result<(), String> {
        self.backend
            .session_start()
            .map_err(|e| e.to_string())?;
        if !self.backend.wait_for_session(30) {
            return Err("Session start timed out".to_string());
        }
        Ok(())
    }

    /// Stop the Waydroid session.
    pub fn stop_session(&self) -> Result<(), String> {
        self.backend
            .session_stop()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// Open the Waydroid UI.
    pub fn open_ui(&self) -> Result<(), String> {
        self.backend
            .show_full_ui()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// Close the Waydroid UI.
    pub fn close_ui(&self) -> Result<(), String> {
        self.backend
            .close_full_ui()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

}

/// Serializable version of the manager's state for Flutter consumption.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ManagerState {
    pub waydroid_status: WaydroidStatus,
    pub profiles: Vec<Profile>,
    pub active_profile: Option<Profile>,
    pub active_orientation: Orientation,
}

/// Get the full application state (convenience method for Flutter).
pub fn get_app_state(manager: &WaydroidManager) -> ManagerState {
    ManagerState {
        waydroid_status: manager.get_status(),
        profiles: manager.get_profiles(),
        active_profile: manager.get_active_profile(),
        active_orientation: Orientation::Portrait, // default, will be updated by Flutter
    }
}
