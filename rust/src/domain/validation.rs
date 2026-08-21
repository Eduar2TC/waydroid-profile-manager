use thiserror::Error;

/// Validation limits for profile parameters.
pub struct ValidationLimits {
    pub min_width: u32,
    pub max_width: u32,
    pub min_height: u32,
    pub max_height: u32,
    pub min_density: u32,
    pub max_density: u32,
    pub min_name_length: usize,
    pub max_name_length: usize,
}

impl Default for ValidationLimits {
    fn default() -> Self {
        Self {
            min_width: 100,
            max_width: 10000,
            min_height: 100,
            max_height: 10000,
            min_density: 40,
            max_density: 1000,
            min_name_length: 1,
            max_name_length: 100,
        }
    }
}

/// Errors that can occur during validation.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("Name is required")]
    NameRequired,

    #[error("Name must be between {min} and {max} characters")]
    NameLength { min: usize, max: usize },

    #[error("Width must be between {min} and {max}")]
    WidthOutOfRange { min: u32, max: u32 },

    #[error("Height must be between {min} and {max}")]
    HeightOutOfRange { min: u32, max: u32 },

    #[error("Density must be between {min} and {max}")]
    DensityOutOfRange { min: u32, max: u32 },
}

/// Validate a profile's fields against the given limits.
pub fn validate_profile(
    name: &str,
    width: u32,
    height: u32,
    density: u32,
    limits: &ValidationLimits,
) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    if name.trim().is_empty() {
        errors.push(ValidationError::NameRequired);
    } else if name.len() < limits.min_name_length || name.len() > limits.max_name_length {
        errors.push(ValidationError::NameLength {
            min: limits.min_name_length,
            max: limits.max_name_length,
        });
    }

    if width < limits.min_width || width > limits.max_width {
        errors.push(ValidationError::WidthOutOfRange {
            min: limits.min_width,
            max: limits.max_width,
        });
    }

    if height < limits.min_height || height > limits.max_height {
        errors.push(ValidationError::HeightOutOfRange {
            min: limits.min_height,
            max: limits.max_height,
        });
    }

    if density < limits.min_density || density > limits.max_density {
        errors.push(ValidationError::DensityOutOfRange {
            min: limits.min_density,
            max: limits.max_density,
        });
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_profile() {
        let result = validate_profile(
            "Phone Small",
            640,
            360,
            160,
            &ValidationLimits::default(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn empty_name_fails() {
        let result = validate_profile(
            "",
            640,
            360,
            160,
            &ValidationLimits::default(),
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.contains(&ValidationError::NameRequired));
    }

    #[test]
    fn zero_width_fails() {
        let result = validate_profile(
            "Test",
            0,
            360,
            160,
            &ValidationLimits::default(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn zero_height_fails() {
        let result = validate_profile(
            "Test",
            640,
            0,
            160,
            &ValidationLimits::default(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn zero_density_fails() {
        let result = validate_profile(
            "Test",
            640,
            360,
            0,
            &ValidationLimits::default(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn width_too_large_fails() {
        let result = validate_profile(
            "Test",
            10001,
            360,
            160,
            &ValidationLimits::default(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn density_too_large_fails() {
        let result = validate_profile(
            "Test",
            640,
            360,
            1001,
            &ValidationLimits::default(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn multiple_errors() {
        let result = validate_profile(
            "",
            0,
            0,
            0,
            &ValidationLimits::default(),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().len() >= 4);
    }

    #[test]
    fn boundary_values_valid() {
        let result = validate_profile(
            "Boundary",
            100,
            100,
            40,
            &ValidationLimits::default(),
        );
        assert!(result.is_ok());

        let result = validate_profile(
            "Boundary",
            10000,
            10000,
            1000,
            &ValidationLimits::default(),
        );
        assert!(result.is_ok());
    }
}
