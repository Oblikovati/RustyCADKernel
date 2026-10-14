//! Owned lifetime and exact unchanged-result invariants through public native operations.
use super::{gui_packets, native_window::NativeWindow, window_scene::WindowScene};
use martensite_vulkan::{FrameCapture, Graphic3dRenderDriver};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::{aspect::*, xw::XwWindow};

/// Both create and wrap retain a consumer; `window_and_wrapper_retention()`.
pub fn window_and_wrapper_retention() {
    let native = NativeWindow::new(64, 64);
    let display = native.window.display_connection();
    assert_retained(&display, 1, 0);
    let wrapped = XwWindow::wrap(&display, native.window.native_handle(), 0).unwrap();
    assert_retained(&display, 2, 0);
    assert_eq!(wrapped.size(), (64, 64));
    drop(wrapped);
    assert_retained(&display, 1, 0);
    drop(native);
    display.init_checked(0).unwrap();
}
/// Invalid constructor inputs acquire no lasting consumer; `failed_window_creation_leaves_reset_available()`.
pub fn failed_window_creation_leaves_reset_available() {
    let display = Handle::new(AspectDisplayConnection::new().unwrap());
    assert!(XwWindow::create(&display, "invalid extent", 0, 0, 0, 64).is_err());
    assert!(XwWindow::wrap(&display, 0, 0).is_err());
    display.init_checked(0).unwrap();
}
/// Runtime retention outlives removed views; `view_retention(&driver, true)`.
pub fn view_retention(
    driver: &Handle<Graphic3dRenderDriver>,
    gui: bool,
) -> Handle<AspectDisplayConnection> {
    let display = retained_view_case(driver, gui);
    assert_retained(&display, 0, 1);
    display
}
fn retained_view_case(
    driver: &Handle<Graphic3dRenderDriver>,
    gui: bool,
) -> Handle<AspectDisplayConnection> {
    let target = WindowScene::new(driver, 64, 64);
    submit_gui(&target, gui);
    target.redraw();
    let baseline = target.scene.render_view().capture().unwrap();
    let display = target.native.window.display_connection();
    assert_retained(&display, 1, 1);
    assert_same_capture(&target, &baseline);
    target.scene.view.must_be_resized();
    submit_gui(&target, gui);
    target.redraw();
    assert_same_capture(&target, &baseline);
    substituted_metadata_keeps_native_binding(&target, gui, &baseline);
    assert_retained(&display, 1, 1);
    target.scene.view.remove();
    display
}
fn substituted_metadata_keeps_native_binding(
    target: &WindowScene,
    gui: bool,
    baseline: &FrameCapture,
) {
    let original = target.native.window.native_display_connection();
    let metadata = Handle::new(AspectDisplayConnection::new().unwrap());
    target.native.window.window_base().fields_mut().display = metadata.clone();
    let pinned = target.native.window.native_display_connection();
    assert_eq!(pinned.object_address(), original.object_address());
    assert_eq!(target.native.window.size(), (64, 64));
    rebind_same_display(target, gui, baseline);
    metadata.init_checked(0).unwrap();
    target.native.window.invalidate_content(&Handle::default());
    assert_retained(&original, 1, 1);
}
fn rebind_same_display(target: &WindowScene, gui: bool, baseline: &FrameCapture) {
    target
        .scene
        .view
        .set_window(&target.native.window.cast(), 0)
        .unwrap();
    submit_gui(target, gui);
    target.redraw();
    assert_same_capture(target, baseline);
}
fn submit_gui(target: &WindowScene, gui: bool) {
    if !gui {
        return;
    }
    let packet = gui_packets::gui_quad([64, 64], [0., 1., 0., 1.], [12., 12., 52., 52.]);
    target
        .scene
        .render_view()
        .attach_gui_layer([64, 64], packet)
        .unwrap();
}
fn assert_same_capture(target: &WindowScene, baseline: &FrameCapture) {
    let capture = target.scene.render_view().capture().unwrap();
    assert_eq!((capture.width, capture.height), (64, 64));
    assert_eq!(capture.rgba, baseline.rgba);
    assert_eq!(capture.depth, baseline.depth);
}
fn assert_retained(display: &AspectDisplayConnection, windows: usize, runtimes: usize) {
    let original = display.get_display_aspect();
    let failure = display.init_checked(0).unwrap_err();
    assert_eq!(failure.operation, AspectDisplayOperation::Initialize);
    assert_eq!(failure.failure_class(), AspectDisplayFailureClass::InUse);
    assert_eq!(failure.cause, AspectDisplayLifecycleCause::Retained);
    assert_eq!(
        (failure.leases.windows, failure.leases.vulkan_instances),
        (windows, runtimes)
    );
    assert_eq!(failure.current_display, original);
    assert_eq!(failure.requested_display, 0);
    assert_eq!(failure.behavior_version, 1);
    assert_eq!(display.get_display_aspect(), original);
}
