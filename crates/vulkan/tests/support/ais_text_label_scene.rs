#![allow(dead_code)]
//! AIS annotation fixtures retain live context, font measurement and presentation state.
use crate::view_setup;
use martensite_vulkan::{Graphic3dRenderDriver, Graphic3dRenderView};
use rustycad_tkernel::{standard::Handle, tcollection::ExtendedString};
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkservice::{aspect::AspectNeutralWindow, graphic3d::*};
use rustycad_tkv3d::{ais::*, v3d::*};
pub const MODES: [&str; 6] = [
    "label-screen",
    "label-zoom",
    "label-plane",
    "label-flip",
    "label-angle",
    "label-update",
];
pub struct TextLabelScene {
    pub view: Handle<V3dView>,
    pub window: Handle<AspectNeutralWindow>,
    pub context: Handle<AisInteractiveContext>,
    pub label: Handle<AisTextLabel>,
}
impl TextLabelScene {
    pub fn new(driver: &Handle<Graphic3dRenderDriver>, mode: &str, offset: f64) -> Self {
        let viewer = V3dViewer::new(&driver.cast()).unwrap();
        let view = viewer.create_view().unwrap();
        let window = view_setup::configure_view(&view);
        let context = AisInteractiveContext::new(&viewer).unwrap();
        let label = AisTextLabel::new();
        configure_label(&label, mode, offset);
        context.display(&label.cast(), false).unwrap();
        let scene = Self {
            view,
            window,
            context,
            label,
        };
        scene.apply(mode);
        scene
    }
    pub fn render_view(&self) -> Handle<Graphic3dRenderView> {
        self.view.get().unwrap().view().cast()
    }
    pub fn apply(&self, mode: &str) {
        if mode == "label-update" {
            self.label.set_text(&ExtendedString::new("VA"));
            self.label.set_position(Point3::new(1., -1., 0.));
            self.context
                .redisplay(&self.label.cast(), false, false)
                .unwrap();
        }
        if mode == "label-flip" {
            self.view
                .camera()
                .set_up(&Direction3::new(0., -1., 0.).unwrap());
        }
    }
}
fn configure_label(label: &AisTextLabel, mode: &str, offset: f64) {
    label.set_text(&ExtendedString::new("AV"));
    label.set_font("DejaVu Sans");
    label.set_height(16.);
    label.set_position(Point3::new(offset, 0., 0.));
    label.set_h_justification(Graphic3dHorizontalTextAlignment::Center);
    label.set_v_justification(Graphic3dVerticalTextAlignment::Center);
    if mode == "label-zoom" {
        label.set_zoomable(true);
        label.set_height(2.);
    }
    if matches!(mode, "label-plane" | "label-flip") {
        label.set_orientation_3d(Axis2::new(label.position(), Direction3::Z).unwrap());
        label.set_flipping(mode == "label-flip");
    }
    if mode == "label-angle" {
        label.set_angle(0.37);
    }
}
