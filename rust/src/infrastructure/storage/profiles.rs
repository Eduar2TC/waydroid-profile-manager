use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::domain::profile::Profile;

/// Stored format for profiles JSON file.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProfilesStore {
    pub profiles: Vec<Profile>,
}

/// Backup of previous Waydroid configuration before applying a profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaydroidBackup {
    pub width: Option<String>,
    pub height: Option<String>,
    pub density: Option<String>,
    pub accelerometer_rotation: Option<String>,
    pub user_rotation: Option<String>,
    pub timestamp: String,
}

/// Application state file.
#[derive(Debug, Serialize, Deserialize)]
pub struct AppState {
    pub active_profile_id: Option<String>,
    pub backup: Option<WaydroidBackup>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_profile_id: None,
            backup: None,
        }
    }
}

/// Repository for profile persistence.
pub struct ProfileRepository {
    config_dir: PathBuf,
}

impl ProfileRepository {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("waydroid-profile-manager");
        Self { config_dir }
    }

    pub fn with_config_dir(config_dir: PathBuf) -> Self {
        Self { config_dir }
    }

    fn profiles_path(&self) -> PathBuf {
        self.config_dir.join("profiles.json")
    }

    fn state_path(&self) -> PathBuf {
        self.config_dir.join("state.json")
    }

    /// Ensure the config directory exists.
    pub fn ensure_dir(&self) -> anyhow::Result<()> {
        fs::create_dir_all(&self.config_dir)?;
        Ok(())
    }

    /// Load profiles from disk. Returns built-in profiles if file doesn't exist.
    pub fn load_profiles(&self) -> anyhow::Result<Vec<Profile>> {
        let path = self.profiles_path();
        if !path.exists() {
            return Ok(Profile::builtins());
        }

        let content = fs::read_to_string(&path)?;
        let store: ProfilesStore = serde_json::from_str(&content)?;
        Ok(store.profiles)
    }

    /// Save profiles to disk.
    pub fn save_profiles(&self, profiles: &[Profile]) -> anyhow::Result<()> {
        self.ensure_dir()?;
        let store = ProfilesStore {
            profiles: profiles.to_vec(),
        };
        let json = serde_json::to_string_pretty(&store)?;
        fs::write(self.profiles_path(), json)?;
        Ok(())
    }

    /// Load application state.
    pub fn load_state(&self) -> anyhow::Result<AppState> {
        let path = self.state_path();
        if !path.exists() {
            return Ok(AppState::default());
        }

        let content = fs::read_to_string(&path)?;
        let state: AppState = serde_json::from_str(&content)?;
        Ok(state)
    }

    /// Save application state.
    pub fn save_state(&self, state: &AppState) -> anyhow::Result<()> {
        self.ensure_dir()?;
        let json = serde_json::to_string_pretty(state)?;
        fs::write(self.state_path(), json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::orientation::Orientation;
    use tempfile::TempDir;

    fn make_repo() -> (ProfileRepository, TempDir) {
        let tmp = TempDir::new().unwrap();
        let repo = ProfileRepository::with_config_dir(tmp.path().to_path_buf());
        (repo, tmp)
    }

    #[test]
    fn save_and_load_profiles() {
        let (repo, _tmp) = make_repo();
        let profiles = vec![
            Profile::new("Test".into(), None, 800, 600, 160, Orientation::Portrait),
            Profile::builtin("phone-sm", "Phone Small", "Desc", 640, 360, 160, Orientation::Portrait),
        ];

        repo.save_profiles(&profiles).unwrap();
        let loaded = repo.load_profiles().unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].name, "Test");
        assert_eq!(loaded[1].id, "phone-sm");
    }

    #[test]
    fn load_defaults_when_no_file() {
        let (repo, _tmp) = make_repo();
        let profiles = repo.load_profiles().unwrap();
        assert_eq!(profiles.len(), 6); // built-in profiles
    }

    #[test]
    fn save_and_load_state() {
        let (repo, _tmp) = make_repo();
        let state = AppState {
            active_profile_id: Some("phone-sm".into()),
            backup: None,
        };
        repo.save_state(&state).unwrap();
        let loaded = repo.load_state().unwrap();
        assert_eq!(loaded.active_profile_id, Some("phone-sm".into()));
    }

    #[test]
    fn state_defaults() {
        let state = AppState::default();
        assert!(state.active_profile_id.is_none());
        assert!(state.backup.is_none());
    }
}
