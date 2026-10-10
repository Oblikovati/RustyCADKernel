//! Run the analytic capping and raster state acceptance on the selected Vulkan device.
#[path = "support/capping_runtime.rs"]
mod capping_runtime;
#[path = "support/device_options.rs"]
mod device_options;
fn main() {
    capping_runtime::check_capping();
}
