//! Run the native window lifecycle independently of offscreen acceptance.
#[path = "support/device_options.rs"]
mod device_options;
#[path = "support/native_window.rs"]
mod native_window;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
#[path = "support/window_runtime.rs"]
mod window_runtime;
#[path = "support/window_scene.rs"]
mod window_scene;
fn main() {
    window_runtime::check_windows();
}
