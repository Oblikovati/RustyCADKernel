//! Native GUI composition uses the existing Graphic3d view, presentation and capture owner.
#[path = "../tests/support/ais_scene.rs"]
mod ais_scene;
#[path = "support/device_options.rs"]
mod device_options;
#[path = "support/gui_acceptance.rs"]
mod gui_acceptance;
#[path = "support/gui_acceptance_scene.rs"]
mod gui_acceptance_scene;
#[path = "support/gui_native_extent.rs"]
mod gui_native_extent;
#[path = "../tests/support/gui_packets.rs"]
mod gui_packets;
#[path = "support/native_window.rs"]
mod native_window;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
#[path = "support/window_scene.rs"]
mod window_scene;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    gui_acceptance::attachment_lifecycle(true);
    gui_acceptance::alpha_and_uv(true);
    gui_acceptance::model_transparency(true);
    gui_acceptance::selection_identity(true);
    gui_acceptance::font_retirement(true);
    gui_acceptance::model_alpha_scope(true);
    gui_native_extent::verify_native_gui_extent();
    if let Some(prefix) = std::env::args().nth(1) {
        gui_acceptance::capture_native_overlay(&prefix)?;
    }
    println!(
        "Martensite native GUI: lifecycle, alpha/UV, three model blend modes, selection all passed"
    );
    Ok(())
}

#[path = "../tests/support/point_alpha_arrays.rs"]
mod point_alpha_arrays;
