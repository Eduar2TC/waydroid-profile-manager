pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod bridge;
pub mod ffi;

// Re-export the public API
pub use bridge::api::WaydroidManager;
pub use domain::orientation::Orientation;
pub use domain::profile::Profile;
pub use domain::waydroid_status::WaydroidStatus;
pub use application::apply_profile::ApplyResult;
pub use application::reset::ResetResult;
