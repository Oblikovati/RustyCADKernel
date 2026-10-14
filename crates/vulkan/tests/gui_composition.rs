//! Owned UI modifies submission only, never the model's geometry or bounds.
#[path = "support/recording_device.rs"]
mod recording_device;
#[path = "support/viewer_scene.rs"]
mod scene;
#[path = "support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::*;
use recording_device::RecordingDevice;

fn fixture() -> (scene::ViewerScene, RecordingDevice) {
    let device = RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(device.clone(), 128).unwrap();
    (scene::ViewerScene::new(&driver), device)
}
fn gui_draw() -> PrimitiveDraw {
    let vertices = vec![RenderVertex::new([0., 0., 0.], [0.25, 0.5, 0.75, 0.4]); 3];
    let image = RasterImage::new([1, 1], vec![[1.; 4]]).unwrap();
    let style = TexturedStyle::new(image, vec![[0.125, 0.875]; 3], 0.).unwrap();
    PrimitiveDraw::textured(vertices, vec![0, 1, 2], identity_clip(), style).unwrap()
}
fn gui_layer() -> RenderLayer {
    let draw = gui_draw();
    RenderLayer::new(vec![draw], false, false, false)
}
#[test]
fn owned_gui_float_colors_uvs_and_model_order_survive_attachment() {
    let (scene, device) = fixture();
    scene
        .render_view()
        .attach_gui_layer([64, 64], gui_layer())
        .unwrap();
    assert!(scene.render_view().capture().is_err());
    scene.render_view().try_redraw().unwrap();
    assert_gui_frame(&device.last_frame());
}
fn assert_gui_frame(frame: &RenderFrame) {
    assert_eq!(frame.draws().len(), 2);
    assert_eq!(frame.draws()[1].vertices()[0].color, [0.25, 0.5, 0.75, 0.4]);
    assert_eq!(
        frame.draws()[1].textured_style().unwrap().uv()[0],
        [0.125, 0.875]
    );
    assert_eq!(
        frame.layers().last().unwrap().depth_policy(),
        (false, false, false)
    );
    assert_eq!(frame.clear_color(), [0., 0., 1., 1.]);
}
#[test]
fn rejected_replacement_is_atomic_and_clear_requires_fresh_capture() {
    let (scene, device) = fixture();
    scene
        .render_view()
        .attach_gui_layer([64, 64], gui_layer())
        .unwrap();
    scene.render_view().try_redraw().unwrap();
    assert!(
        scene
            .render_view()
            .attach_gui_layer([32, 64], gui_layer())
            .is_err()
    );
    assert!(scene.render_view().capture().is_ok());
    scene.render_view().try_redraw().unwrap();
    assert_eq!(device.last_frame().draws().len(), 2);
    assert_clear_model(&scene, &device);
}
fn assert_clear_model(scene: &scene::ViewerScene, device: &RecordingDevice) {
    scene.render_view().clear_gui_layer().unwrap();
    assert!(scene.render_view().capture().is_err());
    scene.render_view().try_redraw().unwrap();
    assert_eq!(device.last_frame().draws().len(), 1);
}
#[test]
fn resize_even_back_to_same_extent_requires_a_new_packet() {
    let (scene, _) = fixture();
    scene
        .render_view()
        .attach_gui_layer([64, 64], gui_layer())
        .unwrap();
    scene.render_view().try_redraw().unwrap();
    scene.window.set_size(80, 64);
    scene.view.must_be_resized();
    scene.window.set_size(64, 64);
    scene.view.must_be_resized();
    assert!(scene.render_view().capture().is_err());
    assert!(scene.render_view().try_redraw().is_err());
    scene
        .render_view()
        .attach_gui_layer([64, 64], gui_layer())
        .unwrap();
    scene.render_view().try_redraw().unwrap();
}
#[test]
fn depth_enabled_gui_and_removed_view_are_rejected() {
    let (scene, _) = fixture();
    let invalid = RenderLayer::new(vec![gui_draw()], true, false, false);
    assert!(
        scene
            .render_view()
            .attach_gui_layer([64, 64], invalid)
            .is_err()
    );
    scene.view.remove();
    assert!(
        scene
            .render_view()
            .attach_gui_layer([64, 64], gui_layer())
            .is_err()
    );
    assert!(scene.render_view().clear_gui_layer().is_err());
    assert!(scene.render_view().capture().is_err());
}

#[test]
fn gui_geometry_is_excluded_from_real_model_bounds_and_fit() {
    let (scene, _) = fixture();
    let bounds = scene.cview().min_max_values(false).unwrap().get().unwrap();
    scene.view.fit_all(0.01, false).unwrap();
    let scale = scene.view.camera().scale();
    let center = scene.view.camera().center();
    scene
        .render_view()
        .attach_gui_layer([64, 64], gui_layer())
        .unwrap();
    scene.view.fit_all(0.01, false).unwrap();
    assert_eq!(
        scene.cview().min_max_values(false).unwrap().get().unwrap(),
        bounds
    );
    assert_eq!(scene.view.camera().scale(), scale);
    assert_eq!(scene.view.camera().center(), center);
    assert_eq!(scene.viewer.active_views().len(), 1);
    assert!(!scene.structure.is_null() && !scene.group.is_null() && !scene.vertices.is_null());
}
#[derive(Clone, Default)]
struct GuiReplacementBackend {
    target: std::sync::Arc<
        rustycad_tkernel::standard::SharedState<
            rustycad_tkernel::standard::WeakHandle<Graphic3dRenderView>,
        >,
    >,
    replace: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
impl RenderBackend for GuiReplacementBackend {
    fn render(&mut self, frame: &RenderFrame) -> Result<FrameCapture, RenderError> {
        if self
            .replace
            .swap(false, std::sync::atomic::Ordering::Relaxed)
        {
            self.target
                .read()
                .observe()
                .attach_gui_layer([64, 64], gui_layer())?;
        }
        let [width, height] = frame.extent();
        Ok(FrameCapture {
            width,
            height,
            rgba: vec![255; (width * height * 4) as usize],
            depth: vec![1.; (width * height) as usize],
        })
    }
}
#[test]
fn replacement_during_submission_rejects_old_capture_then_retries() {
    let backend = GuiReplacementBackend::default();
    let driver = Graphic3dRenderDriver::with_backend(backend.clone(), 128).unwrap();
    let scene = scene::ViewerScene::new(&driver);
    *backend.target.write() = scene.render_view().downgrade();
    backend
        .replace
        .store(true, std::sync::atomic::Ordering::Relaxed);
    assert!(scene.render_view().try_redraw().is_err());
    assert!(scene.render_view().capture().is_err());
    scene.render_view().try_redraw().unwrap();
    assert!(scene.render_view().capture().is_ok());
}
#[test]
fn failed_device_draw_preserves_packet_and_requires_successful_retry() {
    let (scene, device) = fixture();
    scene
        .render_view()
        .attach_gui_layer([64, 64], gui_layer())
        .unwrap();
    device
        .fail
        .store(true, std::sync::atomic::Ordering::Relaxed);
    assert!(scene.render_view().try_redraw().is_err());
    assert!(scene.render_view().capture().is_err());
    device
        .fail
        .store(false, std::sync::atomic::Ordering::Relaxed);
    scene.render_view().try_redraw().unwrap();
    assert_eq!(device.last_frame().draws().len(), 2);
}

#[test]
fn target_extent_change_cannot_return_the_previous_gui_capture() {
    let (scene, _) = fixture();
    scene
        .render_view()
        .attach_gui_layer([64, 64], gui_layer())
        .unwrap();
    scene.render_view().try_redraw().unwrap();
    scene.window.set_size(80, 64);
    assert!(scene.render_view().capture().is_err());
    assert!(scene.render_view().try_redraw().is_err());
    scene.render_view().clear_gui_layer().unwrap();
    scene.render_view().try_redraw().unwrap();
    assert_eq!(scene.render_view().capture().unwrap().width, 80);
}
