//! One fixture interface preserves actual native pixels and display-independent GPU tests.
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
    /// Render and verify actual native pixels when configured; `scene.frame()`.
    pub fn frame(&self) -> FrameCapture {
        self.scene().render_view().try_redraw().unwrap();
        if let GuiAcceptanceTarget::Native(scene) = &self.target {
            scene.verify_pixels();
        }
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
impl GuiAcceptanceScene {
    /// Read actual drawable pixels for proof; example: `scene.presented_pixels().unwrap()`.
    pub fn presented_pixels(&self) -> Option<Vec<u8>> {
        match &self.target {
            GuiAcceptanceTarget::Native(scene) => Some(scene.native.snapshot()),
            GuiAcceptanceTarget::Virtual(_) => None,
        }
    }
}
