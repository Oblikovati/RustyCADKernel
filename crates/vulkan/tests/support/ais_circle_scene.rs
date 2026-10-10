#![allow(dead_code)]
//! Real adaptive circle and arc datums through AIS presentation and selection.
use crate::view_setup;
use martensite_vulkan::{Graphic3dRenderDriver, Graphic3dRenderView};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::{GeomCircle, GeomConic};
use rustycad_tkmath::gp::{Axis2, Point3};
use rustycad_tkservice::aspect::{AspectNeutralWindow, AspectTypeOfDeflection};
use rustycad_tkv3d::{ais::*, prs_mgr::*, v3d::*};
pub const MODES: [&str; 19] = [
    "circle-full",
    "circle-arc",
    "circle-reverse-full",
    "circle-filled",
    "circle-arc-filled",
    "circle-color",
    "circle-width",
    "circle-unset-color",
    "circle-unset-width",
    "circle-replaced",
    "circle-radius",
    "circle-moved",
    "circle-erased",
    "circle-redisplayed",
    "circle-selected",
    "circle-detected",
    "circle-filled-detected",
    "circle-trim",
    "circle-absolute",
];
pub struct CircleScene {
    pub view: Handle<V3dView>,
    pub window: Handle<AspectNeutralWindow>,
    pub context: Handle<AisInteractiveContext>,
    pub circle: Handle<AisCircle>,
}
impl CircleScene {
    pub fn new(driver: &Handle<Graphic3dRenderDriver>, mode: &str) -> Self {
        let viewer = V3dViewer::new(&driver.cast()).unwrap();
        let view = viewer.create_view().unwrap();
        let window = view_setup::configure_view(&view);
        let context = AisInteractiveContext::new(&viewer).unwrap();
        let circle = circle_object(mode);
        context.display(&circle.cast(), false).unwrap();
        Self {
            view,
            window,
            context,
            circle,
        }
    }
    pub fn render_view(&self) -> Handle<Graphic3dRenderView> {
        self.view.get().unwrap().view().cast()
    }
    pub fn apply(&self, mode: &str) -> Result<(), Failure> {
        match mode {
            "circle-color" => self.circle.set_color(Color::named(NamedColor::RED)),
            "circle-width" => self.circle.set_width(3.),
            "circle-unset-color" => {
                self.circle.set_color(Color::named(NamedColor::RED))?;
                self.circle.unset_color()
            }
            "circle-unset-width" => {
                self.circle.set_width(3.)?;
                self.circle.unset_width()
            }
            "circle-replaced" => {
                self.circle.set_circle(&component(3.));
                self.redisplay()
            }
            "circle-radius" => {
                self.circle.circle().set_radius(3.)?;
                self.redisplay()
            }
            "circle-moved" => {
                self.circle.circle().set_location(Point3::new(-2., 1., 0.));
                self.redisplay()
            }
            "circle-trim" => {
                self.circle.set_first_param(0.5);
                self.circle.set_last_param(3.5);
                self.redisplay()
            }
            "circle-absolute" => {
                self.circle
                    .attributes()
                    .set_type_of_deflection(AspectTypeOfDeflection::Absolute);
                self.circle
                    .attributes()
                    .set_maximal_chordial_deviation(0.25);
                self.redisplay()
            }
            "circle-erased" => self.context.erase(&self.circle.cast(), false),
            "circle-redisplayed" => {
                self.context.erase(&self.circle.cast(), false)?;
                self.context.display(&self.circle.cast(), false)
            }
            "circle-selected" => self.context.set_selected_object(&self.circle.cast(), false),
            "circle-detected" | "circle-filled-detected" => {
                self.detect(mode == "circle-filled-detected")
            }
            _ => Ok(()),
        }
    }
    pub fn redisplay(&self) -> Result<(), Failure> {
        self.context.redisplay(&self.circle.cast(), false, false)
    }
    fn detect(&self, filled: bool) -> Result<(), Failure> {
        let offset = self.circle.transformation().translation_part();
        let (x, y) = self.view.convert_world_to_pixels(
            offset.x() + if filled { 0. } else { 5. },
            offset.y(),
            offset.z(),
        )?;
        self.context.move_to(x, y, &self.view, false)?;
        Ok(())
    }
}
fn component(radius: f64) -> Handle<GeomCircle> {
    Handle::new(GeomCircle::new(Axis2::default(), radius).unwrap())
}
fn circle_object(mode: &str) -> Handle<AisCircle> {
    match mode {
        "circle-arc" | "circle-arc-filled" => {
            AisCircle::with_parameters(&component(5.), 0.25, 4.5, mode == "circle-arc-filled")
        }
        "circle-reverse-full" => {
            AisCircle::with_parameters(&component(5.), std::f64::consts::TAU, 0., false)
        }
        "circle-filled" | "circle-filled-detected" => {
            AisCircle::with_parameters(&component(5.), 0., std::f64::consts::TAU, true)
        }
        _ => AisCircle::new(&component(5.)),
    }
}
