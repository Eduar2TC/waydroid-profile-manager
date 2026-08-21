use super::orientation::Orientation;

/// The resolved configuration to apply to Waydroid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceConfiguration {
    pub width: u32,
    pub height: u32,
    pub density: u32,
    pub rotation: u32,
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
    DeviceConfiguration {
        width,
        height,
        density,
        rotation,
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
    }

    #[test]
    fn landscape_phone_small() {
        let config = calculate_configuration(640, 360, 160, Orientation::Landscape);
        assert_eq!(config.width, 640);
        assert_eq!(config.height, 360);
        assert_eq!(config.density, 160);
        assert_eq!(config.rotation, 1);
    }

    #[test]
    fn portrait_tablet_medium() {
        let config = calculate_configuration(1280, 800, 160, Orientation::Portrait);
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 1280);
        assert_eq!(config.density, 160);
        assert_eq!(config.rotation, 0);
    }

    #[test]
    fn landscape_tablet_medium() {
        let config = calculate_configuration(1280, 800, 160, Orientation::Landscape);
        assert_eq!(config.width, 1280);
        assert_eq!(config.height, 800);
        assert_eq!(config.density, 160);
        assert_eq!(config.rotation, 1);
    }

    #[test]
    fn custom_pixel_7_portrait() {
        // Pixel 7-like: 1080x2400, 420 DPI
        let config = calculate_configuration(1080, 2400, 420, Orientation::Portrait);
        assert_eq!(config.width, 2400);
        assert_eq!(config.height, 1080);
        assert_eq!(config.density, 420);
        assert_eq!(config.rotation, 0);
    }

    #[test]
    fn custom_pixel_7_landscape() {
        let config = calculate_configuration(1080, 2400, 420, Orientation::Landscape);
        assert_eq!(config.width, 1080);
        assert_eq!(config.height, 2400);
        assert_eq!(config.density, 420);
        assert_eq!(config.rotation, 1);
    }

    #[test]
    fn square_resolution() {
        let config = calculate_configuration(800, 800, 160, Orientation::Portrait);
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 800);
        assert_eq!(config.rotation, 0);
    }
}
