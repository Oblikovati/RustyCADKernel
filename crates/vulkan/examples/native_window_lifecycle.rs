//! Native lifecycle v1: delivered destruction cannot publish stale captures or kill the process.
#[path = "../tests/support/ais_scene.rs"]
#[allow(dead_code)] // Shared fixture retains ownership fields used by other native cases.
mod ais_scene;
#[path = "support/device_options.rs"]
mod device_options;
#[path = "../tests/support/gui_packets.rs"]
mod gui_packets;
#[path = "support/native_lifecycle_acceptance.rs"]
mod native_lifecycle_acceptance;
#[path = "support/native_lifecycle_events.rs"]
mod native_lifecycle_events;
#[path = "support/native_window.rs"]
mod native_window;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
#[path = "support/window_scene.rs"]
mod window_scene;
fn main() {
    let driver =
        martensite_vulkan::Graphic3dRenderDriver::new(device_options::validation_options())
            .unwrap();
    for with_gui in [false, true] {
        native_lifecycle_acceptance::delivered_destroy_and_rebind(&driver, with_gui);
        native_lifecycle_acceptance::explicit_same_handle_rebind(&driver, with_gui);
    }
    native_lifecycle_acceptance::delivered_destroy_and_remove(&driver);
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    report_acceptance(&driver);
}
fn report_acceptance(driver: &martensite_vulkan::Graphic3dRenderDriver) {
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    println!("MARTENSITE_DEVICE_IDENTITY={:?}", driver.driver_identity());
    println!(
        "Martensite native lifecycle v1: destruction, duplicate event, resize, capture refusal, rebind and double remove passed"
    );
}
