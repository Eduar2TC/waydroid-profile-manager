use serde::{Deserialize, Serialize};

/// Screen orientation for Waydroid profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Orientation {
    Landscape,
    Portrait,
}

impl Orientation {
    /// Convert base dimensions to actual dimensions based on orientation.
    ///
    /// Landscape: width = base_width, height = base_height, rotation = 1
    /// Portrait:  width = base_height, height = base_width, rotation = 0
    pub fn to_actual(&self, base_width: u32, base_height: u32) -> (u32, u32, u32) {
        match self {
            Orientation::Landscape => (base_width, base_height, 1),
            Orientation::Portrait => (base_height, base_width, 0),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Orientation::Landscape => "landscape",
            Orientation::Portrait => "portrait",
        }
    }
}

impl std::fmt::Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Orientation::Landscape => write!(f, "Landscape"),
            Orientation::Portrait => write!(f, "Portrait"),
        }
    }
}

impl Default for Orientation {
    fn default() -> Self {
        Orientation::Portrait
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn landscape_orientation() {
        let (w, h, r) = Orientation::Landscape.to_actual(800, 450);
        assert_eq!(w, 800);
        assert_eq!(h, 450);
        assert_eq!(r, 1);
    }

    #[test]
    fn portrait_orientation() {
        let (w, h, r) = Orientation::Portrait.to_actual(800, 450);
        assert_eq!(w, 450);
        assert_eq!(h, 800);
        assert_eq!(r, 0);
    }

    #[test]
    fn default_is_portrait() {
        assert_eq!(Orientation::default(), Orientation::Portrait);
    }

    #[test]
    fn orientation_display() {
        assert_eq!(Orientation::Landscape.to_string(), "Landscape");
        assert_eq!(Orientation::Portrait.to_string(), "Portrait");
    }

    #[test]
    fn orientation_as_str() {
        assert_eq!(Orientation::Landscape.as_str(), "landscape");
        assert_eq!(Orientation::Portrait.as_str(), "portrait");
    }

    #[test]
    fn serde_roundtrip() {
        let json = serde_json::to_string(&Orientation::Portrait).unwrap();
        assert_eq!(json, "\"portrait\"");
        let parsed: Orientation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Orientation::Portrait);
    }
}
