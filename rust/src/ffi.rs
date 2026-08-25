use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::{Arc, Mutex};

use crate::bridge::api::WaydroidManager;
use crate::domain::orientation::Orientation;
use crate::domain::waydroid_status::OperationProgress;

/// Global manager instance (singleton).
static MANAGER: Mutex<Option<WaydroidManager>> = Mutex::new(None);

/// Global apply state: latest progress + whether apply is still running.
static APPLY_PROGRESS: Mutex<Option<String>> = Mutex::new(None);
static APPLY_RUNNING: Mutex<bool> = Mutex::new(false);

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

/// Update an existing profile by ID.
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub extern "C" fn wpm_update_profile(
    id: *const c_char,
    name: *const c_char,
    description: *const c_char,
    base_width: u32,
    base_height: u32,
    density: u32,
    orientation: *const c_char,
) -> i32 {
    let id = unsafe { cstr_to_string(id) };
    let name = unsafe { optional_cstr(name) };
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

    match mgr.update_profile(
        id,
        name,
        Some(description),
        Some(base_width),
        Some(base_height),
        Some(density),
        Some(orient),
    ) {
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
/// Returns JSON: {"success": bool, "message": string}
/// Caller must free the returned string with wpm_free_string.
#[no_mangle]
pub extern "C" fn wpm_apply_profile(profile_id: *const c_char, orientation: *const c_char) -> *mut c_char {
    let profile_id = unsafe { cstr_to_string(profile_id) };
    let orientation = unsafe { cstr_to_string(orientation) };

    let orient = match orientation.as_str() {
        "landscape" => Orientation::Landscape,
        _ => Orientation::Portrait,
    };

    let manager = match MANAGER.lock() {
        Ok(m) => m,
        Err(_) => return c_string("{\"success\":false,\"message\":\"Manager not initialized\"}"),
    };

    let mgr = match manager.as_ref() {
        Some(m) => m,
        None => return c_string("{\"success\":false,\"message\":\"Manager not initialized\"}"),
    };

    let result = mgr.apply_profile(profile_id, orient);
    let json = serde_json::json!({
        "success": result.success,
        "message": result.message,
    });
    c_string(&json.to_string())
}

/// Apply a profile asynchronously (non-blocking).
/// Progress can be polled via wpm_get_apply_progress().
/// Check wpm_is_apply_done() to know when it's finished.
/// Returns 0 on success (started), -1 on error.
#[no_mangle]
pub extern "C" fn wpm_apply_profile_async(
    profile_id: *const c_char,
    orientation: *const c_char,
) -> i32 {
    let profile_id = unsafe { cstr_to_string(profile_id) };
    let orientation = unsafe { cstr_to_string(orientation) };

    let orient = match orientation.as_str() {
        "landscape" => Orientation::Landscape,
        _ => Orientation::Portrait,
    };

    // Check if already running
    {
        let mut running = match APPLY_RUNNING.lock() {
            Ok(r) => r,
            Err(_) => return -1,
        };
        if *running {
            return -1; // Already applying
        }
        *running = true;
    }

    // Clear previous progress
    {
        if let Ok(mut p) = APPLY_PROGRESS.lock() {
            *p = Some(
                serde_json::json!({
                    "step": "Starting",
                    "message": "Starting apply operation",
                    "completed": false,
                    "done": false,
                    "success": false,
                })
                .to_string(),
            );
        }
    }

    // Spawn a thread to do the work
    std::thread::spawn(move || {
        let progress_store: Arc<Mutex<Option<OperationProgress>>> =
            Arc::new(Mutex::new(None));
        let progress_clone = progress_store.clone();

        let callback = Arc::new(Mutex::new(move |progress: OperationProgress| {
            *progress_clone.lock().unwrap() = Some(progress);
        })) as Arc<Mutex<dyn FnMut(OperationProgress) + Send>>;

        let result = {
            let manager = match MANAGER.lock() {
                Ok(m) => m,
                Err(_) => {
                    set_apply_result(false, "Manager lock failed", true);
                    return;
                }
            };

            let mgr = match manager.as_ref() {
                Some(m) => m,
                None => {
                    set_apply_result(false, "Manager not initialized", true);
                    return;
                }
            };

            mgr.apply_profile_with_progress(profile_id, orient, callback)
        };

        // Drain any final progress
        if let Some(p) = progress_store.lock().unwrap().take() {
            set_progress(&p);
        }

        set_apply_result(result.success, &result.message, true);
    });

    0
}

/// Get the latest apply progress as JSON.
/// Returns null if no apply has been started.
/// JSON shape: {"step": string, "message": string, "completed": bool, "done": bool, "success": bool}
#[no_mangle]
pub extern "C" fn wpm_get_apply_progress() -> *mut c_char {
    let p = match APPLY_PROGRESS.lock() {
        Ok(p) => p,
        Err(_) => return c_string(""),
    };
    match p.as_ref() {
        Some(json_str) => c_string(json_str),
        None => c_string(""),
    }
}

/// Check if the async apply operation is done.
/// Returns 1 if done, 0 if still running, -1 on error.
#[no_mangle]
pub extern "C" fn wpm_is_apply_done() -> i32 {
    let running = match APPLY_RUNNING.lock() {
        Ok(r) => r,
        Err(_) => return -1,
    };
    if *running { 0 } else { 1 }
}

fn set_progress(progress: &OperationProgress) {
    if let Ok(mut p) = APPLY_PROGRESS.lock() {
        let running = APPLY_RUNNING
            .lock()
            .map(|r| *r)
            .unwrap_or(false);
        let json = serde_json::json!({
            "step": progress.step,
            "message": progress.message,
            "completed": progress.completed,
            "done": !running,
            "success": false,
        });
        *p = Some(json.to_string());
    }
}

fn set_apply_result(success: bool, message: &str, done: bool) {
    if let Ok(mut p) = APPLY_PROGRESS.lock() {
        let json = serde_json::json!({
            "step": if success { "Completed" } else { "Error" },
            "message": message,
            "completed": true,
            "done": done,
            "success": success,
        });
        *p = Some(json.to_string());
    }
    if let Ok(mut r) = APPLY_RUNNING.lock() {
        *r = false;
    }
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

/// Export all profiles as a JSON string.
/// Caller must free the returned string with wpm_free_string.
#[no_mangle]
pub extern "C" fn wpm_export_profiles() -> *mut c_char {
    let manager = match MANAGER.lock() {
        Ok(m) => m,
        Err(_) => return c_string("[]"),
    };

    let mgr = match manager.as_ref() {
        Some(m) => m,
        None => return c_string("[]"),
    };

    let profiles = mgr.get_profiles();
    // Export only non-builtin profiles
    let custom: Vec<_> = profiles.into_iter().filter(|p| !p.builtin).collect();
    let json = serde_json::to_string_pretty(&custom).unwrap_or_else(|_| "[]".to_string());
    c_string(&json)
}

/// Import profiles from a JSON string.
/// Returns the number of profiles imported, or -1 on error.
#[no_mangle]
pub extern "C" fn wpm_import_profiles(json: *const c_char) -> i32 {
    let json_str = unsafe { cstr_to_string(json) };
    if json_str.is_empty() {
        return -1;
    }

    let profiles: Vec<crate::domain::profile::Profile> = match serde_json::from_str(&json_str) {
        Ok(p) => p,
        Err(_) => return -1,
    };

    let mut manager = match MANAGER.lock() {
        Ok(m) => m,
        Err(_) => return -1,
    };

    let mgr = match manager.as_mut() {
        Some(m) => m,
        None => return -1,
    };

    let mut imported = 0;
    for profile in profiles {
        // Skip built-in profiles from import
        if profile.builtin {
            continue;
        }
        // Create as new profile (new ID, preserve name/dimensions)
        let result = mgr.create_profile(
            profile.name,
            profile.description,
            profile.base_width,
            profile.base_height,
            profile.density,
            profile.default_orientation,
        );
        if result.is_ok() {
            imported += 1;
        }
    }

    imported
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
