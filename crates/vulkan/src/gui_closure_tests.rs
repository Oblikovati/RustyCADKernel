//! Named fake closure never permits an older successful checked GUI result to escape.
use super::*;
use crate::{
    Graphic3dRenderDriver, RenderBackend, RenderFrame, RenderGuiSurfacePolicy, RenderLayer,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering},
};
struct ClosingGuiWindow {
    transient: StandardTransient,
    neutral: AspectNeutralWindow,
    closed: Arc<AtomicBool>,
    calls: AtomicUsize,
    close_at: AtomicUsize,
}
rustycad_tkernel::impl_transient!(ClosingGuiWindow, "Martensite_ClosingGuiWindow", dyn AspectWindow, transient, interfaces: [dyn AspectWindow]);
impl ClosingGuiWindow {
    fn new(closed: Arc<AtomicBool>) -> Self {
        let neutral = AspectNeutralWindow::default();
        neutral.set_virtual(true);
        neutral.set_size(64, 64);
        neutral.set_native_handle(91);
        Self {
            transient: Default::default(),
            neutral,
            closed,
            calls: AtomicUsize::new(0),
            close_at: AtomicUsize::new(usize::MAX),
        }
    }
}
impl AspectWindow for ClosingGuiWindow {
    fn window_base(&self) -> &AspectWindowBase {
        let native = &self.neutral;
        native.window_base()
    }
    fn is_mapped(&self) -> bool {
        let native = &self.neutral;
        native.is_mapped()
    }
    fn map(&self) {
        let native = &self.neutral;
        native.map();
    }
    fn unmap(&self) {
        let native = &self.neutral;
        native.unmap();
    }
    fn do_resize(&self) -> AspectTypeOfResize {
        let native = &self.neutral;
        native.do_resize()
    }
    fn do_mapping(&self) -> bool {
        let native = &self.neutral;
        native.do_mapping()
    }
    fn ratio(&self) -> f64 {
        let native = &self.neutral;
        native.ratio()
    }
    fn position(&self) -> (i32, i32, i32, i32) {
        let native = &self.neutral;
        native.position()
    }
    fn size(&self) -> (i32, i32) {
        let native = &self.neutral;
        native.size()
    }
    fn native_handle(&self) -> AspectDrawable {
        let native = &self.neutral;
        native.native_handle()
    }
    fn native_parent_handle(&self) -> AspectDrawable {
        let native = &self.neutral;
        native.native_parent_handle()
    }
    fn native_fb_config(&self) -> AspectFbConfig {
        let native = &self.neutral;
        native.native_fb_config()
    }
    fn native_window_closure(&self) -> Option<AspectNativeWindowClosure> {
        let call = self.calls.fetch_add(1, Ordering::Relaxed);
        let closed =
            self.closed.load(Ordering::Relaxed) || call >= self.close_at.load(Ordering::Relaxed);
        closed.then_some(AspectNativeWindowClosure {
            original_handle: 91,
            cause: AspectNativeWindowClosureCause::DestroyNotify,
            behavior_version: 1,
        })
    }
}
#[derive(Clone, Default)]
struct CloseDuringRenderBackend {
    closed: Arc<AtomicBool>,
    action: Arc<AtomicU8>,
}
impl RenderBackend for CloseDuringRenderBackend {
    fn render(&mut self, frame: &RenderFrame) -> Result<FrameCapture, RenderError> {
        let action = self.action.load(Ordering::Relaxed);
        self.closed
            .store(action == 1 || action == 2, Ordering::Relaxed);
        if action >= 2 {
            return Err(closing_primary_fault());
        }
        let [width, height] = frame.extent();
        Ok(FrameCapture {
            width,
            height,
            rgba: vec![17; (width * height * 4) as usize],
            depth: vec![1.; (width * height) as usize],
        })
    }
}
fn closing_primary_fault() -> RenderError {
    let rejected = "owned failing submission concurrent with delivered closure";
    RenderError::rejected(rejected, "successful device submission")
}
fn closing_fixture() -> (
    Handle<Graphic3dRenderView>,
    Handle<ClosingGuiWindow>,
    CloseDuringRenderBackend,
) {
    let backend = CloseDuringRenderBackend::default();
    let driver = Graphic3dRenderDriver::with_backend_and_gui_surface_policy(
        backend.clone(),
        128,
        RenderGuiSurfacePolicy::CheckedV2,
    )
    .unwrap();
    let view = closing_view(&driver);
    let window = Handle::new(ClosingGuiWindow::new(backend.closed.clone()));
    view.set_window(&Handle::default(), &window.cast(), 0);
    view.attach_gui_layer([64, 64], RenderLayer::new(Vec::new(), false, false, false))
        .unwrap();
    verify_initial_closing_frame(&view);
    (view, window, backend)
}
fn assert_stored_closure(view: &Graphic3dRenderView, with_fault: bool) {
    let stored = view.outcome.read().clone().unwrap().unwrap_err();
    let AspectNativeWindowFailure::Closed { closure, .. } = stored.native_window_failure().unwrap();
    assert_eq!(closure.original_handle, 91);
    assert!(stored.gui_surface_outcome().is_none());
    if with_fault {
        assert_eq!(stored.primary_failure(), Some(&closing_primary_fault()));
    }
}
#[test]
fn successful_draw_followed_by_closure_replaces_previous_success() {
    let (view, _, backend) = closing_fixture();
    backend.action.store(1, Ordering::Relaxed);
    assert!(
        view.try_redraw()
            .unwrap_err()
            .native_window_failure()
            .is_some()
    );
    assert_stored_closure(&view, false);
    assert!(
        view.capture()
            .unwrap_err()
            .native_window_failure()
            .is_some()
    );
}
#[test]
fn failed_draw_and_closure_preserve_original_primary_on_redraw_and_capture() {
    let (view, _, backend) = closing_fixture();
    backend.action.store(2, Ordering::Relaxed);
    let failure = view.try_redraw().unwrap_err();
    assert_eq!(failure.primary_failure(), Some(&closing_primary_fault()));
    assert_stored_closure(&view, true);
    let captured = view.capture().unwrap_err();
    assert_eq!(captured.primary_failure(), Some(&closing_primary_fault()));
    assert_stored_closure(&view, true);
}
#[test]
fn closure_at_capture_start_is_stored_without_returning_previous_pixels() {
    let (view, _, backend) = closing_fixture();
    backend.closed.store(true, Ordering::Relaxed);
    assert!(
        view.capture()
            .unwrap_err()
            .native_window_failure()
            .is_some()
    );
    assert_stored_closure(&view, false);
}
#[test]
fn closure_at_final_capture_check_is_stored_without_returning_previous_pixels() {
    let (view, window, _) = closing_fixture();
    window
        .close_at
        .store(window.calls.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
    assert!(
        view.capture()
            .unwrap_err()
            .native_window_failure()
            .is_some()
    );
    assert_stored_closure(&view, false);
}

fn closing_view(driver: &Handle<Graphic3dRenderDriver>) -> Handle<Graphic3dRenderView> {
    let manager = Handle::new(Graphic3dStructureManager::new(&driver.cast()).unwrap());
    driver
        .create_view(&manager)
        .unwrap()
        .cast::<Graphic3dRenderView>()
}

fn verify_initial_closing_frame(view: &Graphic3dRenderView) {
    view.try_redraw().unwrap();
    assert!(view.capture().is_ok());
}
