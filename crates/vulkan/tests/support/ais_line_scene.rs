#![allow(dead_code)]
//! Built-in line datums through AIS display, live aspects, geometry updates and selection.
use rustycad_tkv3d::v3d::V3dViewSource;
#[path = "../../../tkv3d/tests/support/ais_line_fixture.rs"]
pub(crate) mod fixture;
use super::view_setup;
use martensite_vulkan::{Graphic3dRenderDriver, Graphic3dRenderView};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::GeomCartesianPoint;
use rustycad_tkservice::aspect::AspectNeutralWindow;
use rustycad_tkv3d::{ais::*, prs_mgr::*, v3d::*};
pub const MODES: [&str; 11] = [
    "datum-line",
    "datum-infinite",
    "datum-color",
    "datum-width",
    "datum-unset-color",
    "datum-unset-width",
    "datum-moved",
    "datum-erased",
    "datum-redisplayed",
    "datum-selected",
    "datum-detected",
];
pub struct LineScene {
    pub view: Handle<dyn V3dViewSource>,
    pub window: Handle<AspectNeutralWindow>,
    pub context: Handle<AisInteractiveContext>,
    pub line: Handle<AisLine>,
}
impl LineScene {
    pub fn new(driver: &Handle<Graphic3dRenderDriver>, infinite: bool) -> Self {
        let viewer = V3dViewer::new(&driver.cast()).unwrap();
        let view = viewer.create_view().unwrap();
        let window = view_setup::configure_view(&view);
        let context = AisInteractiveContext::new(&viewer).unwrap();
        let line = fixture::object(!infinite, fixture::units());
        line.attributes().set_maximal_parameter_value(12.);
        context.display(&line.cast(), false).unwrap();
        Self {
            view,
            window,
            context,
            line,
        }
    }
    pub fn apply(&self, mode: &str) -> Result<(), Failure> {
        match mode {
            "datum-color" => self.line.set_color(Color::named(NamedColor::RED)),
            "datum-width" => self.line.set_width(3.),
            "datum-unset-color" => self.restore_color(),
            "datum-unset-width" => self.restore_width(),
            "datum-moved" => self.move_endpoints(),
            "datum-erased" => self.context.erase(&self.line.cast(), false),
            "datum-redisplayed" => self.redisplay(),
            "datum-detected" => self.detect(),
            "datum-selected" => self.context.set_selected_object(&self.line.cast(), false),
            "datum-line" | "datum-infinite" => Ok(()),
            _ => panic!("received {mode}; expected a built-in line scene"),
        }
    }
    fn restore_color(&self) -> Result<(), Failure> {
        self.line.set_width(3.)?;
        self.line.set_color(Color::named(NamedColor::RED))?;
        self.line.unset_color()
    }
    fn restore_width(&self) -> Result<(), Failure> {
        self.line.set_color(Color::named(NamedColor::RED))?;
        self.line.set_width(3.)?;
        self.line.unset_width()
    }
    fn move_endpoints(&self) -> Result<(), Failure> {
        self.line.set_points(
            &Handle::new(GeomCartesianPoint::new(-4., 2., 0.)).cast(),
            &Handle::new(GeomCartesianPoint::new(4., -1., 0.)).cast(),
        );
        self.context.redisplay(&self.line.cast(), false, false)
    }
    fn redisplay(&self) -> Result<(), Failure> {
        self.context.erase(&self.line.cast(), false)?;
        self.context.display(&self.line.cast(), false)
    }
    fn detect(&self) -> Result<(), Failure> {
        let origin = self.line.transformation().translation_part();
        let (x, y) = self
            .view
            .convert_world_to_pixels(origin.x(), origin.y(), origin.z())?;
        self.context.move_to(x, y, &self.view, false)?;
        assert_eq!(
            self.context.detected_interactive().object_address(),
            self.line.object_address()
        );
        Ok(())
    }
    pub fn render_view(&self) -> Handle<Graphic3dRenderView> {
        self.view.get().unwrap().view().cast()
    }
}
