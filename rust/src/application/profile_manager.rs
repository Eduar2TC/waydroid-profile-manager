use crate::domain::orientation::Orientation;
use crate::domain::profile::Profile;
use crate::domain::validation::{validate_profile, ValidationLimits, ValidationError};
use crate::infrastructure::storage::profiles::ProfileRepository;

/// Manages profile CRUD operations with persistence.
pub struct ProfileManager {
    repository: ProfileRepository,
    profiles: Vec<Profile>,
}

impl ProfileManager {
    pub fn new(repository: ProfileRepository) -> Self {
        let profiles = repository.load_profiles().unwrap_or_else(|_| Profile::builtins());
        Self {
            repository,
            profiles,
        }
    }

    /// Get all profiles.
    pub fn get_profiles(&self) -> &[Profile] {
        &self.profiles
    }

    /// Get a profile by ID.
    pub fn get_profile(&self, id: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.id == id)
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
    ) -> Result<Profile, Vec<ValidationError>> {
        validate_profile(
            &name,
            base_width,
            base_height,
            density,
            &ValidationLimits::default(),
        )?;

        let profile = Profile::new(
            name,
            description,
            base_width,
            base_height,
            density,
            default_orientation,
        );

        self.profiles.push(profile.clone());
        self.save().map_err(|_| vec![ValidationError::NameRequired])?;
        Ok(profile)
    }

    /// Update an existing profile.
    #[allow(clippy::too_many_arguments)]
    pub fn update_profile(
        &mut self,
        id: &str,
        name: Option<String>,
        description: Option<Option<String>>,
        base_width: Option<u32>,
        base_height: Option<u32>,
        density: Option<u32>,
        default_orientation: Option<Orientation>,
    ) -> Result<(), Vec<ValidationError>> {
        let profile = self
            .profiles
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| vec![ValidationError::NameRequired])?; // TODO: better error

        // Validate with final values
        let final_name = name.as_deref().unwrap_or(&profile.name);
        let final_width = base_width.unwrap_or(profile.base_width);
        let final_height = base_height.unwrap_or(profile.base_height);
        let final_density = density.unwrap_or(profile.density);

        validate_profile(
            final_name,
            final_width,
            final_height,
            final_density,
            &ValidationLimits::default(),
        )?;

        profile.update(
            name,
            description,
            base_width,
            base_height,
            density,
            default_orientation,
        );

        self.save().map_err(|_| vec![ValidationError::NameRequired])?;
        Ok(())
    }

    /// Delete a profile by ID. Built-in profiles cannot be deleted.
    pub fn delete_profile(&mut self, id: &str) -> anyhow::Result<()> {
        let idx = self
            .profiles
            .iter()
            .position(|p| p.id == id)
            .ok_or_else(|| anyhow::anyhow!("Profile not found"))?;

        if self.profiles[idx].builtin {
            return Err(anyhow::anyhow!("Cannot delete built-in profile"));
        }

        self.profiles.remove(idx);
        self.save()?;
        Ok(())
    }

    /// Duplicate a profile.
    pub fn duplicate_profile(
        &mut self,
        id: &str,
        new_name: Option<String>,
    ) -> anyhow::Result<Profile> {
        let original = self
            .profiles
            .iter()
            .find(|p| p.id == id)
            .ok_or_else(|| anyhow::anyhow!("Profile not found"))?;

        let dup = original.duplicate(new_name);
        self.profiles.push(dup.clone());
        self.save()?;
        Ok(dup)
    }

    /// Save profiles to disk.
    fn save(&self) -> anyhow::Result<()> {
        self.repository.save_profiles(&self.profiles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_manager() -> (ProfileManager, TempDir) {
        let tmp = TempDir::new().unwrap();
        let repo = ProfileRepository::with_config_dir(tmp.path().to_path_buf());
        (ProfileManager::new(repo), tmp)
    }

    #[test]
    fn initial_profiles_are_builtins() {
        let (mgr, _tmp) = make_manager();
        assert_eq!(mgr.get_profiles().len(), 6);
    }

    #[test]
    fn create_custom_profile() {
        let (mut mgr, _tmp) = make_manager();
        let profile = mgr
            .create_profile(
                "Pixel 7".into(),
                Some("Custom profile".into()),
                1080,
                2400,
                420,
                Orientation::Portrait,
            )
            .unwrap();
        assert_eq!(mgr.get_profiles().len(), 7);
        assert_eq!(profile.name, "Pixel 7");
        assert!(!profile.builtin);
    }

    #[test]
    fn delete_custom_profile() {
        let (mut mgr, _tmp) = make_manager();
        let profile = mgr
            .create_profile("Delete Me".into(), None, 800, 600, 160, Orientation::Portrait)
            .unwrap();
        assert_eq!(mgr.get_profiles().len(), 7);
        mgr.delete_profile(&profile.id).unwrap();
        assert_eq!(mgr.get_profiles().len(), 6);
    }

    #[test]
    fn cannot_delete_builtin() {
        let (mut mgr, _tmp) = make_manager();
        let result = mgr.delete_profile("phone-sm");
        assert!(result.is_err());
    }

    #[test]
    fn duplicate_profile() {
        let (mut mgr, _tmp) = make_manager();
        let dup = mgr.duplicate_profile("phone-sm", Some("My Phone".into())).unwrap();
        assert_eq!(dup.name, "My Phone");
        assert!(!dup.builtin);
        assert_eq!(mgr.get_profiles().len(), 7);
    }

    #[test]
    fn create_invalid_profile_fails() {
        let (mut mgr, _tmp) = make_manager();
        let result = mgr.create_profile(
            "".into(),
            None,
            0,
            0,
            0,
            Orientation::Portrait,
        );
        assert!(result.is_err());
    }

    #[test]
    fn update_profile() {
        let (mut mgr, _tmp) = make_manager();
        let profile = mgr
            .create_profile("Original".into(), None, 800, 600, 160, Orientation::Portrait)
            .unwrap();
        mgr.update_profile(
            &profile.id,
            Some("Updated".into()),
            None,
            Some(1920),
            Some(1080),
            Some(240),
            Some(Orientation::Landscape),
        )
        .unwrap();
        let updated = mgr.get_profile(&profile.id).unwrap();
        assert_eq!(updated.name, "Updated");
        assert_eq!(updated.base_width, 1920);
    }
}
