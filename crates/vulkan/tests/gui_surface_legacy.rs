//! Revision-bound legacy fault ordering for Oblikovati#3630; no native I/O.
use martensite_vulkan::*;
use rustycad_tkernel::standard::{Handle, SharedState, WeakHandle};
use rustycad_tkservice::{
    aspect::{AspectNeutralWindow, AspectWindowMembers},
    graphic3d::*,
};
use std::sync::Arc;

struct GuiFaultFixture {
    view: Handle<Graphic3dRenderView>,
    window: Handle<AspectNeutralWindow>,
}
impl GuiFaultFixture {
    fn new(driver: &Handle<Graphic3dRenderDriver>) -> Self {
        let manager = Handle::new(Graphic3dStructureManager::new(&driver.cast()).unwrap());
        let view = driver
            .create_view(&manager)
            .unwrap()
            .cast::<Graphic3dRenderView>();
        let window = Handle::new(AspectNeutralWindow::default());
        window.set_virtual(true);
        window.set_size(64, 64);
        view.set_window(&Handle::default(), &window.cast(), 0);
        Self { view, window }
    }
}
#[derive(Clone, Default)]
struct ResizeAndFailBackend {
    target: Arc<SharedState<WeakHandle<Graphic3dRenderView>>>,
    window: Arc<SharedState<Handle<AspectNeutralWindow>>>,
    resize: Arc<SharedState<Option<bool>>>,
}
impl RenderBackend for ResizeAndFailBackend {
    fn render(&mut self, _frame: &RenderFrame) -> Result<FrameCapture, RenderError> {
        self.deliver_resize();
        let fault = injected_primary_fault();
        Err(fault)
    }
}
impl ResizeAndFailBackend {
    fn deliver_resize(&self) {
        let Some(return_to_original) = *self.resize.read() else {
            return;
        };
        let window = self.window.read().clone();
        let target = self.target.read().observe();
        window.set_size(80, 72);
        target.resized();
        if return_to_original {
            window.set_size(64, 64);
            target.resized();
        }
    }
}
fn injected_primary_fault() -> RenderError {
    let operation = "owned device fault sentinel 3630";
    RenderError::rejected(operation, "successful device submission")
}
fn legacy_fixture(with_gui: bool) -> (GuiFaultFixture, ResizeAndFailBackend) {
    let backend = ResizeAndFailBackend::default();
    let driver = Graphic3dRenderDriver::with_backend(backend.clone(), 128).unwrap();
    let scene = GuiFaultFixture::new(&driver);
    *backend.target.write() = scene.view.downgrade();
    *backend.window.write() = scene.window.clone();
    if with_gui {
        let packet = RenderLayer::new(Vec::new(), false, false, false);
        scene.view.attach_gui_layer([64, 64], packet).unwrap();
    }
    (scene, backend)
}
fn assert_legacy_resize_masks_fault(return_to_original: bool) {
    let (scene, backend) = legacy_fixture(true);
    *backend.resize.write() = Some(return_to_original);
    let failure = scene.view.try_redraw().unwrap_err();
    assert_eq!(
        failure,
        RenderError::rejected(
            "composition or window binding changed during rendering",
            "a redraw with unchanged composition and window binding",
        )
    );
    assert_legacy_stale_capture(&scene.view, return_to_original);
}
fn assert_legacy_stale_capture(view: &Graphic3dRenderView, return_to_original: bool) {
    let extent = if return_to_original {
        [64, 64]
    } else {
        [80, 72]
    };
    let failure = RenderError::rejected(
        ([64, 64], extent, true),
        "a fresh GUI packet matching the current target extent",
    );
    assert_eq!(view.capture().unwrap_err(), failure);
}
#[test]
fn legacy_delivered_resize_masks_primary_device_fault() {
    let return_to_original = false;
    assert_legacy_resize_masks_fault(return_to_original);
}
#[test]
fn legacy_away_and_back_masks_fault_at_equal_extent() {
    let return_to_original = true;
    assert_legacy_resize_masks_fault(return_to_original);
}
#[test]
fn legacy_capture_extent_check_masks_stored_primary_fault() {
    let (scene, _) = legacy_fixture(true);
    assert_eq!(
        scene.view.try_redraw().unwrap_err(),
        injected_primary_fault()
    );
    assert_eq!(scene.view.capture().unwrap_err(), injected_primary_fault());
    scene.window.set_size(80, 72);
    assert_eq!(
        scene.view.capture().unwrap_err(),
        RenderError::rejected(
            ([64, 64], [80, 72], false),
            "a fresh GUI packet matching the current target extent",
        )
    );
}
#[test]
fn legacy_model_resize_preserves_primary_device_fault() {
    let (scene, backend) = legacy_fixture(false);
    *backend.resize.write() = Some(false);
    assert_eq!(
        scene.view.try_redraw().unwrap_err(),
        injected_primary_fault()
    );
    assert_eq!(scene.view.capture().unwrap_err(), injected_primary_fault());
}
