//! Run the native window lifecycle independently of offscreen acceptance.
#[cfg(target_os = "linux")]
#[path = "support/device_options.rs"]
mod device_options;
#[cfg(target_os = "linux")]
#[path = "support/native_window.rs"]
mod native_window;
#[cfg(target_os = "linux")]
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
#[cfg(target_os = "linux")]
#[path = "support/window_runtime.rs"]
mod window_runtime;
#[cfg(target_os = "linux")]
#[path = "support/window_scene.rs"]
mod window_scene;
#[cfg(target_os = "linux")]
fn main() {
    window_runtime::check_windows();
}

#[allow(dead_code)]
#[cfg(target_os = "linux")]
#[path = "../tests/support/ais_scene.rs"]
mod ais_scene;

#[cfg(not(target_os = "linux"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Err("native X11 window example requires Linux".into())
}
