use serde::{Deserialize, Serialize};

/// Screen orientation for Waydroid profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum Orientation {
    Landscape,
    #[default]
    Portrait,
}

impl Orientation {
    /// Convert base dimensions to actual HOST WINDOW dimensions based on orientation.
    ///
    /// Landscape: width = base_width, height = base_height, rotation = 0
    /// Portrait:  width = base_height, height = base_width, rotation = 0
    ///
    /// The host window (via persist.waydroid.width/height) uses orientation-specific
    /// dimensions so Waydroid creates the correct window shape.
    ///
    /// Rotation is always 0 because `wm size` now matches the host window dimensions,
    /// so no rotation is needed.
    pub fn to_actual(&self, base_width: u32, base_height: u32) -> (u32, u32, u32) {
        match self {
            Orientation::Landscape => (base_width, base_height, 0),
            Orientation::Portrait => (base_height, base_width, 0),
        }
    }

    /// Compute the internal Android display dimensions for `wm size`.
    ///
    /// Returns dimensions matching the host window orientation:
    /// - Landscape: (max, min) — matches the host window width × height
    /// - Portrait: (min, max) — matches the host window width × height
    pub fn to_wm_size(&self, base_width: u32, base_height: u32) -> (u32, u32) {
        match self {
            Orientation::Landscape => (base_width, base_height),
            Orientation::Portrait => (base_height, base_width),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn landscape_orientation() {
        let (w, h, r) = Orientation::Landscape.to_actual(800, 450);
        assert_eq!(w, 800);
        assert_eq!(h, 450);
        assert_eq!(r, 0); // rotation=0 since wm size matches window
    }

    #[test]
    fn portrait_orientation() {
        let (w, h, r) = Orientation::Portrait.to_actual(800, 450);
        assert_eq!(w, 450);
        assert_eq!(h, 800);
        assert_eq!(r, 0);
    }

    #[test]
    fn wm_size_matches_orientation() {
        // wm size matches host window dimensions (same as to_actual without rotation)
        let (w, h) = Orientation::Landscape.to_wm_size(640, 360);
        assert_eq!(w, 640);
        assert_eq!(h, 360);

        let (w, h) = Orientation::Portrait.to_wm_size(640, 360);
        assert_eq!(w, 360);
        assert_eq!(h, 640);

        // Reversed base (e.g. Pixel 7: 1080x2400)
        let (w, h) = Orientation::Landscape.to_wm_size(1080, 2400);
        assert_eq!(w, 1080);
        assert_eq!(h, 2400);

        let (w, h) = Orientation::Portrait.to_wm_size(1080, 2400);
        assert_eq!(w, 2400);
        assert_eq!(h, 1080);

        // Square stays the same
        let (w, h) = Orientation::Landscape.to_wm_size(800, 800);
        assert_eq!(w, 800);
        assert_eq!(h, 800);
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
