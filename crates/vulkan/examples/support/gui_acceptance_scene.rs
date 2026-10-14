//! One owner runs native and virtual render targets; acceptance captures their completed framebuffer.
use super::{ais_scene::AisViewerScene, window_scene::WindowScene};
use martensite_vulkan::{FrameCapture, Graphic3dRenderDriver};
use rustycad_tkernel::standard::Handle;
enum GuiAcceptanceTarget {
    Virtual(AisViewerScene),
    Native(Box<WindowScene>),
}
pub struct GuiAcceptanceScene {
    target: GuiAcceptanceTarget,
}
impl GuiAcceptanceScene {
    /// Own either real native or virtual GPU test setup; `GuiAcceptanceScene::new(driver,true)`.
    pub fn new(driver: &Handle<Graphic3dRenderDriver>, use_native: bool) -> Self {
        let target = if use_native {
            GuiAcceptanceTarget::Native(Box::new(WindowScene::new(driver, 64, 64)))
        } else {
            GuiAcceptanceTarget::Virtual(AisViewerScene::new(driver))
        };
        Self { target }
    }
    /// Borrow the actual AIS scene; `scene.scene().view.redraw()`.
    pub fn scene(&self) -> &AisViewerScene {
        match &self.target {
            GuiAcceptanceTarget::Virtual(scene) => scene,
            GuiAcceptanceTarget::Native(scene) => &scene.scene,
        }
    }
    /// Replace actual test model ownership; `scene.scene_mut().object = object`.
    pub fn scene_mut(&mut self) -> &mut AisViewerScene {
        match &mut self.target {
            GuiAcceptanceTarget::Virtual(scene) => scene,
            GuiAcceptanceTarget::Native(scene) => &mut scene.scene,
        }
    }
    /// Render and capture the completed composited framebuffer; `scene.frame()`.
    pub fn frame(&self) -> FrameCapture {
        self.scene().render_view().try_redraw().unwrap();
        self.scene().render_view().capture().unwrap()
    }
    /// Notify the actual target and view; `scene.resize(80,72)`.
    pub fn resize(&self, width: i32, height: i32) {
        if let GuiAcceptanceTarget::Native(scene) = &self.target {
            scene.resize(width, height);
            return;
        }
        self.scene().window.set_size(width, height);
        self.scene().view.must_be_resized();
    }
}
