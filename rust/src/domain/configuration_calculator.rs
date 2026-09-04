use super::orientation::Orientation;

/// The resolved configuration to apply to Waydroid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceConfiguration {
    /// Host window width (persist.waydroid.width) — orientation-specific.
    pub width: u32,
    /// Host window height (persist.waydroid.height) — orientation-specific.
    pub height: u32,
    /// Display density (persist.waydroid.density / wm density).
    pub density: u32,
    /// Rotation for `cmd window set-user-rotation lock`. 0=portrait, 1=landscape.
    pub rotation: u32,
    /// Internal Android display width for `wm size` — always portrait (min).
    pub wm_width: u32,
    /// Internal Android display height for `wm size` — always portrait (max).
    pub wm_height: u32,
}

/// Calculate the actual Waydroid configuration from profile base values and orientation.
///
/// This is a pure function with no side effects — easily testable.
pub fn calculate_configuration(
    base_width: u32,
    base_height: u32,
    density: u32,
    orientation: Orientation,
) -> DeviceConfiguration {
    let (width, height, rotation) = orientation.to_actual(base_width, base_height);
    let (wm_width, wm_height) = orientation.to_wm_size(base_width, base_height);
    DeviceConfiguration {
        width,
        height,
        density,
        rotation,
        wm_width,
        wm_height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portrait_phone_small() {
        // Profile: 640x360 base, 160 DPI, Portrait
        let config = calculate_configuration(640, 360, 160, Orientation::Portrait);
        assert_eq!(config.width, 360);
        assert_eq!(config.height, 640);
        assert_eq!(config.density, 160);
        assert_eq!(config.rotation, 0);
        assert_eq!(config.wm_width, 360);
        assert_eq!(config.wm_height, 640);
    }

    #[test]
    fn landscape_phone_small() {
        let config = calculate_configuration(640, 360, 160, Orientation::Landscape);
        // Props use landscape dims for the host window
        assert_eq!(config.width, 640);
        assert_eq!(config.height, 360);
        assert_eq!(config.density, 160);
        assert_eq!(config.rotation, 0); // rotation=0 since wm size matches window
        // wm size matches host window dimensions
        assert_eq!(config.wm_width, 640);
        assert_eq!(config.wm_height, 360);
    }

    #[test]
    fn portrait_tablet_medium() {
        let config = calculate_configuration(1280, 800, 160, Orientation::Portrait);
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 1280);
        assert_eq!(config.density, 160);
        assert_eq!(config.rotation, 0);
        assert_eq!(config.wm_width, 800);
        assert_eq!(config.wm_height, 1280);
    }

    #[test]
    fn landscape_tablet_medium() {
        let config = calculate_configuration(1280, 800, 160, Orientation::Landscape);
        assert_eq!(config.width, 1280);
        assert_eq!(config.height, 800);
        assert_eq!(config.density, 160);
        assert_eq!(config.rotation, 0); // rotation=0 since wm size matches window
        // wm size matches host window dimensions
        assert_eq!(config.wm_width, 1280);
        assert_eq!(config.wm_height, 800);
    }

    #[test]
    fn custom_pixel_7_portrait() {
        // Pixel 7-like: 1080x2400, 420 DPI
        let config = calculate_configuration(1080, 2400, 420, Orientation::Portrait);
        // Portrait swaps: width=base_height, height=base_width
        assert_eq!(config.width, 2400);
        assert_eq!(config.height, 1080);
        assert_eq!(config.density, 420);
        assert_eq!(config.rotation, 0);
        // wm size matches host window dimensions
        assert_eq!(config.wm_width, 2400);
        assert_eq!(config.wm_height, 1080);
    }

    #[test]
    fn custom_pixel_7_landscape() {
        let config = calculate_configuration(1080, 2400, 420, Orientation::Landscape);
        // Landscape keeps base dims: width=base_width, height=base_height
        assert_eq!(config.width, 1080);
        assert_eq!(config.height, 2400);
        assert_eq!(config.density, 420);
        assert_eq!(config.rotation, 0);
        // wm size matches host window dimensions
        assert_eq!(config.wm_width, 1080);
        assert_eq!(config.wm_height, 2400);
    }

    #[test]
    fn square_resolution() {
        let config = calculate_configuration(800, 800, 160, Orientation::Portrait);
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 800);
        assert_eq!(config.rotation, 0);
        assert_eq!(config.wm_width, 800);
        assert_eq!(config.wm_height, 800);
    }
}
