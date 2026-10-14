#![allow(dead_code)]
//! Real-window AIS fixture shared by capture, lifecycle and timing probes.
use super::ais_scene;
use super::native_window::NativeWindow;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::{aspect::*, graphic3d::*};
pub const MODES: [&str; 6] = [
    "window-base",
    "window-resize",
    "window-rebind",
    "window-vsync",
    "window-remap",
    "window-erased",
];
pub struct WindowScene {
    pub native: NativeWindow,
    pub scene: ais_scene::AisViewerScene,
    width: i32,
    height: i32,
}
impl WindowScene {
    pub fn new(driver: &Handle<Graphic3dRenderDriver>, width: i32, height: i32) -> Self {
        let native = NativeWindow::new(width, height);
        let scene = ais_scene::AisViewerScene::new(driver);
        scene.view.set_window(&native.window.cast(), 0).unwrap();
        scene.view.must_be_resized();
        Self {
            native,
            scene,
            width,
            height,
        }
    }
    pub fn resize(&self, width: i32, height: i32) {
        self.native.resize(width, height);
        self.scene.view.must_be_resized();
    }
    pub fn redraw(&self) {
        self.scene.render_view().try_redraw().unwrap();
    }
    pub fn verify_pixels(&self) -> Vec<u8> {
        self.native
            .wait_for_pixels(&self.scene.render_view().capture().unwrap().rgba)
    }
    pub fn apply(&mut self, mode: &str, driver: &Graphic3dRenderDriver) -> Result<(), Failure> {
        match mode {
            "window-resize" => {
                self.resize(self.width + 16, self.height + 8);
                self.redraw();
                self.resize(self.width, self.height);
            }
            "window-rebind" => {
                self.native = NativeWindow::new(self.width, self.height);
                self.scene.view.set_window(&self.native.window.cast(), 0)?;
            }
            "window-vsync" => {
                driver.set_vertical_sync(true);
                self.redraw();
                driver.set_vertical_sync(false);
            }
            "window-remap" => {
                self.native.unmap();
                self.redraw();
                self.native.window.map();
                self.native.sync();
            }
            "window-erased" => self.scene.context.erase(&self.scene.object.cast(), false)?,
            _ => (),
        }
        Ok(())
    }
}
