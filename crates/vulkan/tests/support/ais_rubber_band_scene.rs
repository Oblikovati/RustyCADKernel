#![allow(dead_code)]
//! Real AIS_RubberBand overlays exercise screen persistence and live polygon updates.
use super::view_setup;
use martensite_vulkan::{Graphic3dRenderDriver, Graphic3dRenderView};
use rustycad_tkernel::{
    ncollection::Vec2,
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::{Transform3, Vector3};
use rustycad_tkservice::aspect::{AspectNeutralWindow, AspectTypeOfLine};
use rustycad_tkv3d::{ais::*, prs_mgr::*, v3d::*};
pub const MODES: [&str; 8] = [
    "rubber-rectangle",
    "rubber-open",
    "rubber-filled",
    "rubber-transparent",
    "rubber-concave",
    "rubber-dash",
    "rubber-wide",
    "rubber-updated",
];
pub struct RubberBandScene {
    pub view: Handle<V3dView>,
    pub window: Handle<AspectNeutralWindow>,
    pub context: Handle<AisInteractiveContext>,
    pub object: Handle<AisRubberBand>,
}
impl RubberBandScene {
    pub fn new(driver: &Handle<Graphic3dRenderDriver>) -> Result<Self, Failure> {
        let viewer = V3dViewer::new(&driver.cast())?;
        let view = viewer.create_view()?;
        let window = view_setup::configure_view(&view);
        let context = AisInteractiveContext::new(&viewer)?;
        let object = AisRubberBand::new()?;
        object.set_rectangle(8, 8, 36, 36);
        context.display(&object.cast(), false)?;
        Ok(Self {
            view,
            window,
            context,
            object,
        })
    }
    pub fn render_view(&self) -> Handle<Graphic3dRenderView> {
        self.view.get().unwrap().view().cast()
    }
    pub fn apply(&self, mode: &str, offset: f64) -> Result<(), Failure> {
        self.object
            .set_local_transformation(&Transform3::translation_vector(Vector3::new(
                offset, 0., 0.,
            )))?;
        match mode {
            "rubber-open" => self.object.set_polygon_closed(false),
            "rubber-filled" | "rubber-concave" => self
                .object
                .set_filling_color(Color::named(NamedColor::RED), 0.)?,
            "rubber-transparent" => self
                .object
                .set_filling_color(Color::named(NamedColor::RED), 0.5)?,
            "rubber-dash" => self.object.set_line_type(AspectTypeOfLine::Dash),
            "rubber-wide" => self.object.set_line_width(3.)?,
            "rubber-updated" => self.object.set_rectangle(12, 10, 40, 34),
            "rubber-rectangle" => (),
            _ => {
                return Err(Failure::with_message(
                    "Standard_DomainError",
                    &format!("received mode {mode}; expected {MODES:?}"),
                ));
            }
        }
        if mode == "rubber-concave" {
            self.concave_points();
        }
        self.context.redisplay(&self.object.cast(), false, false)
    }
    fn concave_points(&self) {
        self.object.clear_points();
        for (x, y) in [(8, 8), (8, 36), (20, 20), (36, 36), (36, 8)] {
            self.object.add_point(Vec2::new(x, y));
        }
    }
}
