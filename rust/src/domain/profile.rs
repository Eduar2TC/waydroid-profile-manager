use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::orientation::Orientation;

/// A Waydroid device profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub base_width: u32,
    pub base_height: u32,
    pub density: u32,
    pub default_orientation: Orientation,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub builtin: bool,
}

impl Profile {
    /// Create a new user profile with generated ID and timestamps.
    pub fn new(
        name: String,
        description: Option<String>,
        base_width: u32,
        base_height: u32,
        density: u32,
        default_orientation: Orientation,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            base_width,
            base_height,
            density,
            default_orientation,
            created_at: now,
            updated_at: now,
            builtin: false,
        }
    }

    /// Create a built-in profile with a fixed ID.
    pub fn builtin(
        id: &str,
        name: &str,
        description: &str,
        base_width: u32,
        base_height: u32,
        density: u32,
        default_orientation: Orientation,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: Some(description.to_string()),
            base_width,
            base_height,
            density,
            default_orientation,
            created_at: now,
            updated_at: now,
            builtin: true,
        }
    }

    /// Duplicate this profile, generating a new ID and marking as non-builtin.
    pub fn duplicate(&self, new_name: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: new_name.unwrap_or_else(|| format!("{} (Copy)", self.name)),
            description: self.description.clone(),
            base_width: self.base_width,
            base_height: self.base_height,
            density: self.density,
            default_orientation: self.default_orientation,
            created_at: now,
            updated_at: now,
            builtin: false,
        }
    }

    /// Update mutable fields, preserving ID, created_at, and builtin status.
    pub fn update(
        &mut self,
        name: Option<String>,
        description: Option<Option<String>>,
        base_width: Option<u32>,
        base_height: Option<u32>,
        density: Option<u32>,
        default_orientation: Option<Orientation>,
    ) {
        if let Some(n) = name {
            self.name = n;
        }
        if let Some(d) = description {
            self.description = d;
        }
        if let Some(w) = base_width {
            self.base_width = w;
        }
        if let Some(h) = base_height {
            self.base_height = h;
        }
        if let Some(d) = density {
            self.density = d;
        }
        if let Some(o) = default_orientation {
            self.default_orientation = o;
        }
        self.updated_at = Utc::now();
    }

    /// Get all built-in profiles.
    pub fn builtins() -> Vec<Self> {
        vec![
            Self::builtin(
                "phone-sm",
                "Phone Small",
                "Small phone profile",
                640,
                360,
                160,
                Orientation::Portrait,
            ),
            Self::builtin(
                "phone-md",
                "Phone Medium",
                "Medium phone profile",
                800,
                450,
                180,
                Orientation::Portrait,
            ),
            Self::builtin(
                "phone-lg",
                "Phone Large",
                "Large phone profile",
                960,
                540,
                200,
                Orientation::Portrait,
            ),
            Self::builtin(
                "tablet-sm",
                "Tablet Small",
                "Small tablet profile",
                1024,
                640,
                160,
                Orientation::Landscape,
            ),
            Self::builtin(
                "tablet-md",
                "Tablet Medium",
                "Medium tablet profile",
                1280,
                800,
                160,
                Orientation::Landscape,
            ),
            Self::builtin(
                "tablet-lg",
                "Tablet Large",
                "Large tablet profile",
                1600,
                1000,
                160,
                Orientation::Landscape,
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_new_profile() {
        let p = Profile::new(
            "My Phone".into(),
            None,
            1080,
            2400,
            420,
            Orientation::Portrait,
        );
        assert!(!p.id.is_empty());
        assert_eq!(p.name, "My Phone");
        assert_eq!(p.base_width, 1080);
        assert_eq!(p.base_height, 2400);
        assert_eq!(p.density, 420);
        assert!(!p.builtin);
    }

    #[test]
    fn create_builtin_profile() {
        let p = Profile::builtin(
            "test",
            "Test",
            "Test profile",
            800,
            600,
            160,
            Orientation::Landscape,
        );
        assert!(p.builtin);
        assert_eq!(p.id, "test");
    }

    #[test]
    fn duplicate_profile() {
        let original = Profile::builtin(
            "orig",
            "Original",
            "Desc",
            800,
            600,
            160,
            Orientation::Portrait,
        );
        let dup = original.duplicate(None);
        assert_ne!(dup.id, original.id);
        assert_eq!(dup.name, "Original (Copy)");
        assert!(!dup.builtin);
        assert_eq!(dup.base_width, original.base_width);
    }

    #[test]
    fn duplicate_with_custom_name() {
        let original = Profile::builtin(
            "orig",
            "Original",
            "Desc",
            800,
            600,
            160,
            Orientation::Portrait,
        );
        let dup = original.duplicate(Some("My Copy".into()));
        assert_eq!(dup.name, "My Copy");
    }

    #[test]
    fn update_profile() {
        let mut p = Profile::new(
            "Old".into(),
            None,
            800,
            600,
            160,
            Orientation::Portrait,
        );
        let old_updated = p.updated_at;
        p.update(
            Some("New".into()),
            Some(Some("Updated".into())),
            Some(1920),
            Some(1080),
            Some(240),
            Some(Orientation::Landscape),
        );
        assert_eq!(p.name, "New");
        assert_eq!(p.description, Some("Updated".into()));
        assert_eq!(p.base_width, 1920);
        assert_eq!(p.base_height, 1080);
        assert_eq!(p.density, 240);
        assert_eq!(p.default_orientation, Orientation::Landscape);
        assert!(p.updated_at >= old_updated);
    }

    #[test]
    fn builtins_count() {
        assert_eq!(Profile::builtins().len(), 6);
    }

    #[test]
    fn builtins_are_marked() {
        for p in Profile::builtins() {
            assert!(p.builtin);
        }
    }

    #[test]
    fn serde_roundtrip() {
        let p = Profile::new(
            "Test".into(),
            Some("Desc".into()),
            1080,
            2400,
            420,
            Orientation::Portrait,
        );
        let json = serde_json::to_string(&p).unwrap();
        let parsed: Profile = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, p);
    }
}
