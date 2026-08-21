use crate::domain::waydroid_status::WaydroidStatus;
use crate::infrastructure::storage::profiles::ProfileRepository;
use crate::infrastructure::waydroid::backend_trait::WaydroidBackend;

use crate::domain::profile::Profile;

/// Get the full Waydroid status.
pub fn get_status(backend: &dyn WaydroidBackend) -> WaydroidStatus {
    backend.get_full_status()
}

/// Get the currently active profile, if any.
pub fn get_active_profile(
    repository: &ProfileRepository,
    profiles: &[Profile],
) -> Option<Profile> {
    let state = repository.load_state().ok()?;
    let active_id = state.active_profile_id?;
    profiles.iter().find(|p| p.id == active_id).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::waydroid::service::FakeWaydroidBackend;
    use crate::infrastructure::storage::profiles::{AppState, ProfileRepository};
    use crate::domain::orientation::Orientation;
    use tempfile::TempDir;

    #[test]
    fn get_status_returns_running() {
        let backend = FakeWaydroidBackend::new();
        let status = get_status(&backend);
        assert!(status.installed);
        assert!(status.is_running());
    }

    #[test]
    fn get_active_profile_none() {
        let tmp = TempDir::new().unwrap();
        let repo = ProfileRepository::with_config_dir(tmp.path().to_path_buf());
        let profiles = Profile::builtins();
        let active = get_active_profile(&repo, &profiles);
        assert!(active.is_none());
    }

    #[test]
    fn get_active_profile_found() {
        let tmp = TempDir::new().unwrap();
        let repo = ProfileRepository::with_config_dir(tmp.path().to_path_buf());
        let state = AppState {
            active_profile_id: Some("phone-sm".into()),
            backup: None,
        };
        repo.save_state(&state).unwrap();

        let profiles = Profile::builtins();
        let active = get_active_profile(&repo, &profiles);
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, "phone-sm");
    }
}
