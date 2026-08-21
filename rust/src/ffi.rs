use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Mutex;

use crate::bridge::api::WaydroidManager;
use crate::domain::orientation::Orientation;

/// Global manager instance (singleton).
static MANAGER: Mutex<Option<WaydroidManager>> = Mutex::new(None);

/// Initialize the Waydroid manager.
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub extern "C" fn wpm_init() -> i32 {
    let manager = WaydroidManager::new();
    match MANAGER.lock() {
        Ok(mut m) => {
            *m = Some(manager);
            0
        }
        Err(_) => -1,
    }
}

/// Get Waydroid status as JSON string.
/// Caller must free the returned string with wpm_free_string.
#[no_mangle]
pub extern "C" fn wpm_get_status() -> *mut c_char {
    let manager = match MANAGER.lock() {
        Ok(m) => m,
        Err(_) => return c_string(""),
    };

    let mgr = match manager.as_ref() {
        Some(m) => m,
        None => return c_string(""),
    };

    let status = mgr.get_status();
    let json = serde_json::to_string(&status).unwrap_or_else(|_| "{}".to_string());
    c_string(&json)
}

/// Get all profiles as JSON array string.
/// Caller must free the returned string with wpm_free_string.
#[no_mangle]
pub extern "C" fn wpm_get_profiles() -> *mut c_char {
    let manager = match MANAGER.lock() {
        Ok(m) => m,
        Err(_) => return c_string("[]"),
    };

    let mgr = match manager.as_ref() {
        Some(m) => m,
        None => return c_string("[]"),
    };

    let profiles = mgr.get_profiles();
    let json = serde_json::to_string(&profiles).unwrap_or_else(|_| "[]".to_string());
    c_string(&json)
}

/// Get active profile as JSON string.
/// Caller must free the returned string with wpm_free_string.
#[no_mangle]
pub extern "C" fn wpm_get_active_profile() -> *mut c_char {
    let manager = match MANAGER.lock() {
        Ok(m) => m,
        Err(_) => return c_string("null"),
    };

    let mgr = match manager.as_ref() {
        Some(m) => m,
        None => return c_string("null"),
    };

    match mgr.get_active_profile() {
        Some(profile) => {
            let json = serde_json::to_string(&profile).unwrap_or_else(|_| "null".to_string());
            c_string(&json)
        }
        None => c_string("null"),
    }
}

/// Create a new profile.
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub extern "C" fn wpm_create_profile(
    name: *const c_char,
    description: *const c_char,
    base_width: u32,
    base_height: u32,
    density: u32,
    orientation: *const c_char,
) -> i32 {
    let name = unsafe { cstr_to_string(name) };
    let description = unsafe { optional_cstr(description) };
    let orientation = unsafe { cstr_to_string(orientation) };

    let orient = match orientation.as_str() {
        "landscape" => Orientation::Landscape,
        _ => Orientation::Portrait,
    };

    let mut manager = match MANAGER.lock() {
        Ok(m) => m,
        Err(_) => return -1,
    };

    let mgr = match manager.as_mut() {
        Some(m) => m,
        None => return -1,
    };

    match mgr.create_profile(name, description, base_width, base_height, density, orient) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// Delete a profile by ID.
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub extern "C" fn wpm_delete_profile(id: *const c_char) -> i32 {
    let id = unsafe { cstr_to_string(id) };

    let mut manager = match MANAGER.lock() {
        Ok(m) => m,
        Err(_) => return -1,
    };

    let mgr = match manager.as_mut() {
        Some(m) => m,
        None => return -1,
    };

    match mgr.delete_profile(id) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// Duplicate a profile.
/// Returns the new profile ID as JSON string.
/// Caller must free the returned string with wpm_free_string.
#[no_mangle]
pub extern "C" fn wpm_duplicate_profile(id: *const c_char, new_name: *const c_char) -> *mut c_char {
    let id = unsafe { cstr_to_string(id) };
    let new_name = unsafe { optional_cstr(new_name) };

    let mut manager = match MANAGER.lock() {
        Ok(m) => m,
        Err(_) => return c_string(""),
    };

    let mgr = match manager.as_mut() {
        Some(m) => m,
        None => return c_string(""),
    };

    match mgr.duplicate_profile(id, new_name) {
        Ok(profile) => {
            let json = serde_json::to_string(&profile).unwrap_or_else(|_| "{}".to_string());
            c_string(&json)
        }
        Err(_) => c_string(""),
    }
}

/// Apply a profile.
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub extern "C" fn wpm_apply_profile(profile_id: *const c_char, orientation: *const c_char) -> i32 {
    let profile_id = unsafe { cstr_to_string(profile_id) };
    let orientation = unsafe { cstr_to_string(orientation) };

    let orient = match orientation.as_str() {
        "landscape" => Orientation::Landscape,
        _ => Orientation::Portrait,
    };

    let manager = match MANAGER.lock() {
        Ok(m) => m,
        Err(_) => return -1,
    };

    let mgr = match manager.as_ref() {
        Some(m) => m,
        None => return -1,
    };

    let result = mgr.apply_profile(profile_id, orient);
    if result.success { 0 } else { -1 }
}

/// Reset Waydroid configuration.
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub extern "C" fn wpm_reset() -> i32 {
    let manager = match MANAGER.lock() {
        Ok(m) => m,
        Err(_) => return -1,
    };

    let mgr = match manager.as_ref() {
        Some(m) => m,
        None => return -1,
    };

    let result = mgr.reset();
    if result.success { 0 } else { -1 }
}

/// Free a string returned by this API.
#[no_mangle]
pub extern "C" fn wpm_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}

// Helper functions
fn c_string(s: &str) -> *mut c_char {
    match CString::new(s) {
        Ok(cstr) => cstr.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

unsafe fn cstr_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    CStr::from_ptr(ptr).to_string_lossy().to_string()
}

unsafe fn optional_cstr(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        let s = CStr::from_ptr(ptr).to_string_lossy().to_string();
        if s.is_empty() { None } else { Some(s) }
    }
}
