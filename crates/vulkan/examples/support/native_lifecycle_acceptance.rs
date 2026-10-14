//! Independent closed-target assertions run through the public native view path.
use super::{
    gui_packets, native_lifecycle_events as events, native_window::NativeWindow,
    window_scene::WindowScene,
};
use martensite_vulkan::{FrameCapture, Graphic3dRenderDriver, RenderError};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::aspect::AspectNativeWindowOperation as Operation;
use rustycad_tkservice::{aspect::*, graphic3d::Graphic3dCView};

/// Check closed errors and fresh rebind bytes; `delivered_destroy_and_rebind(&driver, true)`.
pub fn delivered_destroy_and_rebind(driver: &Handle<Graphic3dRenderDriver>, with_gui: bool) {
    let target = WindowScene::new(driver, 64, 64);
    prepare_capture(&target, with_gui);
    let baseline = target.scene.render_view().capture().unwrap();
    reject_foreign_destroy(&target);
    let original = target.native.window.native_handle();
    destroy_with_distinct_parent(&target, original);
    assert_closed_operations(&target, original);
    target.scene.view.must_be_resized();
    assert_closed_offscreen(&target, original);
    rebind_fresh_frame(&target, with_gui, &baseline);
    target.scene.view.remove();
    target.scene.view.remove();
    assert!(target.scene.render_view().capture().is_err());
}
/// Check removal after destruction remains idempotent; `delivered_destroy_and_remove(&driver)`.
pub fn delivered_destroy_and_remove(driver: &Handle<Graphic3dRenderDriver>) {
    let target = WindowScene::new(driver, 64, 64);
    prepare_capture(&target, false);
    let original = target.native.window.native_handle();
    let mut event = events::destroy_native_window(&target.native.window);
    assert!(events::deliver(&target.native.window, &mut event));
    target.scene.view.remove();
    target.scene.view.remove();
    assert_closed(
        target.scene.render_view().capture(),
        Operation::Capture,
        original,
    );
}
fn prepare_capture(target: &WindowScene, with_gui: bool) {
    if with_gui {
        install_gui(target);
    }
    target.redraw();
    assert_eq!(target.scene.render_view().capture().unwrap().width, 64);
}
fn install_gui(target: &WindowScene) {
    let layer = gui_packets::gui_quad([64, 64], [0., 1., 0., 1.], [12., 12., 52., 52.]);
    target
        .scene
        .render_view()
        .attach_gui_layer([64, 64], layer)
        .unwrap();
}
fn reject_foreign_destroy(target: &WindowScene) {
    let original = target.native.window.native_handle();
    let mut unrelated = events::synthetic_destroy_event(original, original + 1);
    assert!(!events::deliver(&target.native.window, &mut unrelated));
    assert_eq!(target.native.window.native_handle(), original);
    assert!(target.scene.render_view().capture().is_ok());
}
fn assert_closed_operations(target: &WindowScene, original: u64) {
    let render = target.scene.render_view();
    assert_closed(render.capture(), Operation::Capture, original);
    assert_closed(render.try_redraw(), Operation::Redraw, original);
    let layer = gui_packets::gui_quad([64, 64], [1.; 4], [0., 0., 64., 64.]);
    assert_closed(
        render.attach_gui_layer([64, 64], layer),
        Operation::AttachGui,
        original,
    );
    assert_closed(render.clear_gui_layer(), Operation::ClearGui, original);
}
fn assert_closed<T>(result: Result<T, RenderError>, operation: Operation, original: u64) {
    let failure = result
        .err()
        .expect("closed target must reject this operation");
    let closure = AspectNativeWindowClosure {
        original_handle: original,
        cause: AspectNativeWindowClosureCause::DestroyNotify,
        behavior_version: 1,
    };
    assert_eq!(
        failure.native_window_failure(),
        Some(&closure.failure(operation))
    );
}
fn assert_closed_offscreen(target: &WindowScene, original: u64) {
    let render = target.scene.render_view();
    let mut fbo = render.fbo_create(64, 64);
    render.set_fbo(&fbo);
    assert_closed(render.capture(), Operation::Capture, original);
    assert_closed(render.try_redraw(), Operation::Redraw, original);
    render.set_fbo(&Handle::default());
    render.fbo_release(&mut fbo);
}
fn rebind_fresh_frame(target: &WindowScene, with_gui: bool, baseline: &FrameCapture) {
    let replacement = NativeWindow::new(64, 64);
    target
        .scene
        .view
        .set_window(&replacement.window.cast(), 0)
        .unwrap();
    assert_fresh_rebind(target, with_gui, baseline);
}
fn assert_fresh_rebind(target: &WindowScene, with_gui: bool, baseline: &FrameCapture) {
    let render = target.scene.render_view();
    assert!(render.capture().is_err());
    if with_gui {
        assert!(render.try_redraw().is_err());
        install_gui(target);
    }
    render.try_redraw().unwrap();
    let fresh = render.capture().unwrap();
    assert_eq!(fresh.rgba, baseline.rgba);
    assert_eq!(fresh.depth, baseline.depth);
}

fn destroy_with_distinct_parent(target: &WindowScene, original: u64) {
    let mut event = events::destroy_native_window(&target.native.window);
    let mut different_parent = events::synthetic_destroy_event(original + 1, original);
    assert!(events::deliver(
        &target.native.window,
        &mut different_parent
    ));
    assert!(events::deliver(&target.native.window, &mut event));
    assert_eq!(target.native.window.native_handle(), 0);
}

/// Rebind always invalidates even an equal handle and extent; `explicit_same_handle_rebind(&driver, true)`.
pub fn explicit_same_handle_rebind(driver: &Handle<Graphic3dRenderDriver>, with_gui: bool) {
    let target = WindowScene::new(driver, 64, 64);
    prepare_capture(&target, with_gui);
    let baseline = target.scene.render_view().capture().unwrap();
    let original = target.native.window.native_handle();
    target
        .scene
        .view
        .set_window(&target.native.window.cast(), 0)
        .unwrap();
    assert_eq!(target.native.window.native_handle(), original);
    assert_fresh_rebind(&target, with_gui, &baseline);
}
