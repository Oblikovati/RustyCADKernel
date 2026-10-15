//! Named fake drives public view operations without native I/O.
use martensite_vulkan::*;
use rustycad_tkernel::standard::{Handle, SharedState, WeakHandle};
use rustycad_tkservice::{
    aspect::{AspectNeutralWindow, AspectWindow, AspectWindowMembers},
    graphic3d::*,
};
use std::sync::Arc;

pub struct GuiFaultFixture {
    pub view: Handle<Graphic3dRenderView>,
    pub window: Handle<AspectNeutralWindow>,
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
pub struct ResizeAndFailBackend {
    target: Arc<SharedState<WeakHandle<Graphic3dRenderView>>>,
    window: Arc<SharedState<Handle<AspectNeutralWindow>>>,
    pub resize: Arc<SharedState<Option<bool>>>,
    pub replace: Arc<std::sync::atomic::AtomicBool>,
    pub succeed: Arc<std::sync::atomic::AtomicBool>,
}
impl RenderBackend for ResizeAndFailBackend {
    fn render(&mut self, _frame: &RenderFrame) -> Result<FrameCapture, RenderError> {
        self.deliver_resize();
        self.replace_packet()?;
        if self.succeed.load(std::sync::atomic::Ordering::Relaxed) {
            return Ok(fixed_gui_capture(_frame.extent()));
        }
        let fault = injected_primary_fault();
        Err(fault)
    }
}
impl ResizeAndFailBackend {
    fn replace_packet(&self) -> Result<(), RenderError> {
        if !self
            .replace
            .swap(false, std::sync::atomic::Ordering::Relaxed)
        {
            return Ok(());
        }
        let target = self.target.read().observe();
        let (width, height) = self.window.read().size();
        target.attach_gui_layer([width as u32, height as u32], empty_gui_packet())
    }
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
pub fn injected_primary_fault() -> RenderError {
    let operation = "owned device fault sentinel 3630";
    RenderError::rejected(operation, "successful device submission")
}
pub fn gui_fault_fixture(
    with_gui: bool,
    policy: Option<RenderGuiSurfacePolicy>,
) -> (GuiFaultFixture, ResizeAndFailBackend) {
    let backend = ResizeAndFailBackend::default();
    let driver = match policy {
        Some(policy) => {
            Graphic3dRenderDriver::with_backend_and_gui_surface_policy(backend.clone(), 128, policy)
        }
        None => Graphic3dRenderDriver::with_backend(backend.clone(), 128),
    }
    .unwrap();
    let scene = GuiFaultFixture::new(&driver);
    *backend.target.write() = scene.view.downgrade();
    *backend.window.write() = scene.window.clone();
    if with_gui {
        scene
            .view
            .attach_gui_layer([64, 64], empty_gui_packet())
            .unwrap();
    }
    (scene, backend)
}
pub fn empty_gui_packet() -> RenderLayer {
    let draws = Vec::new();
    RenderLayer::new(draws, false, false, false)
}
fn fixed_gui_capture([width, height]: [u32; 2]) -> FrameCapture {
    FrameCapture {
        width,
        height,
        rgba: vec![91; (width * height * 4) as usize],
        depth: vec![1.; (width * height) as usize],
    }
}
