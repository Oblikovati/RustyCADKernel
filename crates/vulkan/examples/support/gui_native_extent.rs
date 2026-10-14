//! An application-owned neutral window may lag behind the actual native drawable.
use super::{ais_scene::AisViewerScene, device_options, gui_packets, native_window::NativeWindow};
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::aspect::{AspectNeutralWindow, AspectWindow, AspectWindowMembers};
use rustycad_tkservice::graphic3d::Graphic3dCView;
/// Reject stale surface dimensions and prove recovery; example: `verify_native_gui_extent()`.
pub fn verify_native_gui_extent() {
    let driver = Graphic3dRenderDriver::new(device_options::validation_options()).unwrap();
    let native = NativeWindow::new(64, 64);
    let window = application_window(&native);
    let scene = AisViewerScene::new(&driver);
    scene.view.set_window(&window.cast(), 0).unwrap();
    install_packet(&scene, [64, 64]);
    scene.render_view().try_redraw().unwrap();
    native.resize(80, 72);
    assert!(scene.render_view().try_redraw().is_err());
    assert!(scene.render_view().capture().is_err());
    verify_extent_retry(&scene, &native, &window);
    scene.render_view().remove();
    assert!(driver.validation_errors().is_empty());
}
fn application_window(native: &NativeWindow) -> Handle<AspectNeutralWindow> {
    let window = Handle::new(AspectNeutralWindow::default());
    window.window_base().fields_mut().display = native.window.display_connection();
    window.set_native_handle(native.window.native_handle());
    window.set_size(64, 64);
    window
}
fn install_packet(scene: &AisViewerScene, extent: [u32; 2]) {
    let layer = gui_packets::gui_quad(extent, [0., 1., 0., 1.], [0., 0., 80., 72.]);
    scene.render_view().attach_gui_layer(extent, layer).unwrap();
}
fn verify_extent_retry(
    scene: &AisViewerScene,
    native: &NativeWindow,
    window: &AspectNeutralWindow,
) {
    window.set_size(80, 72);
    scene.view.must_be_resized();
    install_packet(scene, [80, 72]);
    scene.render_view().try_redraw().unwrap();
    let capture = scene.render_view().capture().unwrap();
    assert_eq!((capture.width, capture.height), (80, 72));
    assert_eq!(capture.pixel(32, 32), Some([0, 255, 0, 255]));
    // Capture is the completed framebuffer; rootless XWayland client pixmaps can remain stale.
    assert_eq!(native.window.size(), (80, 72));
}

/// Prove capture refusal survives observed away/back resize; `verify_checked_native_gui_extent()`.
pub fn verify_checked_native_gui_extent() {
    let driver = Graphic3dRenderDriver::with_gui_surface_policy(
        device_options::validation_options(),
        martensite_vulkan::RenderGuiSurfacePolicy::CheckedV2,
    )
    .unwrap();
    let native = NativeWindow::new(64, 64);
    let window = application_window(&native);
    let scene = AisViewerScene::new(&driver);
    scene.view.set_window(&window.cast(), 0).unwrap();
    install_packet(&scene, [64, 64]);
    scene.render_view().try_redraw().unwrap();
    assert_checked_capture_refusals(&scene, &native);
    verify_checked_mapping(&scene, &window);
    scene.render_view().remove();
    assert!(driver.validation_errors().is_empty());
}
fn assert_checked_capture_refusals(scene: &AisViewerScene, native: &NativeWindow) {
    native.resize(80, 72);
    let error = scene.render_view().capture().unwrap_err();
    assert!(matches!(
        error.gui_surface_outcome(),
        Some(martensite_vulkan::RenderGuiSurfaceOutcome::SurfaceChanged(
            _
        ))
    ));
    native.resize(64, 64);
    assert!(scene.render_view().capture().is_err());
    assert!(scene.render_view().try_redraw().is_err());
    install_packet(scene, [64, 64]);
    scene.render_view().try_redraw().unwrap();
    assert_eq!(
        scene.render_view().capture().unwrap().pixel(32, 32),
        Some([0, 255, 0, 255])
    );
}
fn verify_checked_mapping(scene: &AisViewerScene, window: &AspectNeutralWindow) {
    window.unmap();
    scene.render_view().try_redraw().unwrap();
    assert_eq!(
        scene.render_view().capture().unwrap().pixel(32, 32),
        Some([0, 255, 0, 255])
    );
    window.map();
    scene.render_view().try_redraw().unwrap();
    assert_eq!(
        scene.render_view().capture().unwrap().pixel(32, 32),
        Some([0, 255, 0, 255])
    );
}
