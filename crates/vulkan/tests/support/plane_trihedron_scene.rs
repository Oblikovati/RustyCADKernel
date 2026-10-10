#![allow(dead_code)]
//! Plane trihedrons through the real AIS context, including separate arrow/text styles.
use crate::{line_scene::fixture, view_setup};
use martensite_vulkan::{Graphic3dRenderDriver, Graphic3dRenderView};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
    tcollection::AsciiString,
};
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkservice::aspect::AspectNeutralWindow;
use rustycad_tkv3d::{ais::*, prs_mgr::*, prs3d::Prs3dDatumParts, v3d::*};
pub const MODES: [&str; 14] = [
    "plane-base",
    "plane-indirect",
    "plane-length",
    "plane-anisotropic",
    "plane-color",
    "plane-styles",
    "plane-labels",
    "plane-empty-labels",
    "plane-moved",
    "plane-mutated",
    "plane-erased",
    "plane-redisplayed",
    "plane-selected",
    "plane-detected",
];
pub struct PlaneScene {
    pub view: Handle<V3dView>,
    pub window: Handle<AspectNeutralWindow>,
    pub context: Handle<AisInteractiveContext>,
    pub object: Handle<AisPlaneTrihedron>,
}
impl PlaneScene {
    pub fn new(driver: &Handle<Graphic3dRenderDriver>, mode: &str) -> Self {
        let viewer = V3dViewer::new(&driver.cast()).unwrap();
        let view = viewer.create_view().unwrap();
        let window = view_setup::configure_view(&view);
        let context = AisInteractiveContext::new(&viewer).unwrap();
        let mut frame = Axis3::default();
        frame.set_location(Point3::new(-3., -3., 0.));
        if mode == "plane-indirect" {
            frame.y_reverse();
        }
        let object =
            AisPlaneTrihedron::new(&Handle::new(GeomPlane::new(frame)), fixture::units()).unwrap();
        object.set_length(6.);
        context.display(&object.cast(), false).unwrap();
        Self {
            view,
            window,
            context,
            object,
        }
    }
    pub fn render_view(&self) -> Handle<Graphic3dRenderView> {
        self.view.get().unwrap().view().cast()
    }
    pub fn apply(&self, mode: &str) -> Result<(), Failure> {
        match mode {
            "plane-color" => self.object.set_color(Color::named(NamedColor::RED)),
            "plane-erased" => self.context.erase(&self.object.cast(), false),
            "plane-redisplayed" => {
                self.context.erase(&self.object.cast(), false)?;
                self.context.display(&self.object.cast(), false)
            }
            "plane-selected" => self.context.set_selected_object(&self.object.cast(), false),
            "plane-detected" => self.detect(),
            _ => {
                self.mutate(mode);
                self.context.redisplay(&self.object.cast(), false, false)
            }
        }
    }
    fn mutate(&self, mode: &str) {
        match mode {
            "plane-length" => self.object.set_length(4.),
            "plane-anisotropic" => self
                .object
                .attributes()
                .datum_aspect()
                .set_axis_length(5., 3., 8.),
            "plane-labels" => self.labels("U", "V"),
            "plane-empty-labels" => self.labels("", ""),
            "plane-styles" => self.styles(),
            "plane-moved" => {
                let mut frame = Axis3::default();
                frame.set_location(Point3::new(-2., 1., 0.));
                self.object
                    .set_component(&Handle::new(GeomPlane::new(frame)));
            }
            "plane-mutated" => {
                let mut p = self.object.component().pln();
                p.set_location(Point3::new(-2., 1., 0.));
                self.object.component().set_pln(p);
            }
            _ => (),
        }
    }
    fn labels(&self, x: &str, y: &str) {
        self.object.set_x_label(&AsciiString::from(x));
        self.object.set_y_label(&AsciiString::from(y));
    }
    fn styles(&self) {
        let drawer = self.object.attributes();
        drawer
            .datum_aspect()
            .line_aspect(Prs3dDatumParts::YAxis)
            .set_color(Color::named(NamedColor::GREEN));
        drawer
            .arrow_aspect()
            .set_color(Color::named(NamedColor::RED));
        drawer
            .text_aspect()
            .set_color(Color::named(NamedColor::CYAN));
    }
    fn detect(&self) -> Result<(), Failure> {
        let offset = self.object.transformation().translation_part();
        let (x, y) = self
            .view
            .convert_world_to_pixels(offset.x(), offset.y() - 3., offset.z())?;
        self.context.move_to(x, y, &self.view, false)?;
        Ok(())
    }
}
