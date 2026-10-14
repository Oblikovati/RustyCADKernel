//! The same GUI acceptance assertions run on an actual GPU without a display dependency.
#[path = "support/ais_scene.rs"]
mod ais_scene;
#[path = "../examples/support/device_options.rs"]
mod device_options;
#[path = "../examples/support/gui_acceptance.rs"]
mod gui_acceptance;
#[path = "../examples/support/gui_acceptance_scene.rs"]
mod gui_acceptance_scene;
#[path = "support/gui_packets.rs"]
mod gui_packets;
#[path = "../examples/support/native_window.rs"]
mod native_window;
#[path = "support/view_setup.rs"]
mod view_setup;
#[path = "../examples/support/window_scene.rs"]
mod window_scene;
#[test]
fn gpu_gui_attachment_lifecycle() {
    let use_native = false;
    gui_acceptance::attachment_lifecycle(use_native);
}
#[test]
fn gpu_gui_ordered_alpha_and_font_uv() {
    let use_native = false;
    gui_acceptance::alpha_and_uv(use_native);
}
#[test]
fn gpu_gui_after_model_oit() {
    let use_native = false;
    gui_acceptance::model_transparency(use_native);
}
#[test]
fn gpu_gui_model_selection_identity() {
    let use_native = false;
    gui_acceptance::selection_identity(use_native);
}
#[test]
fn gpu_gui_font_producer_retirement_and_view_close() {
    let use_native = false;
    gui_acceptance::font_retirement(use_native);
}

#[path = "support/point_alpha_arrays.rs"]
mod point_alpha_arrays;
#[test]
fn gpu_gui_alpha_scope_keeps_retained_model_state_after_clear() {
    let use_native = false;
    gui_acceptance::model_alpha_scope(use_native);
}
