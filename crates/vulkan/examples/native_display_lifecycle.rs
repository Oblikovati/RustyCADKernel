//! Native display v1: a refused reset preserves the accepted window/view until its consumers drop.
#[path = "../tests/support/ais_scene.rs"]
#[allow(dead_code)]
mod ais_scene;
#[path = "support/device_options.rs"]
mod device_options;
#[path = "../tests/support/gui_packets.rs"]
mod gui_packets;
#[path = "support/native_display_cases.rs"]
mod native_display_cases;
#[path = "support/native_window.rs"]
mod native_window;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
#[path = "support/window_scene.rs"]
mod window_scene;

fn main() {
    native_display_cases::window_and_wrapper_retention();
    native_display_cases::failed_window_creation_leaves_reset_available();
    let driver =
        martensite_vulkan::Graphic3dRenderDriver::new(device_options::validation_options())
            .unwrap();
    let displays = [false, true].map(|gui| native_display_cases::view_retention(&driver, gui));
    report_driver(&driver);
    drop(driver);
    for display in displays {
        display.init_checked(0).unwrap();
    }
    println!(
        "Martensite native display v1: refusal, unchanged capture, runtime retention and released reset passed"
    );
}

fn report_driver(driver: &martensite_vulkan::Graphic3dRenderDriver) {
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    println!("MARTENSITE_DEVICE_IDENTITY={:?}", driver.driver_identity());
}
