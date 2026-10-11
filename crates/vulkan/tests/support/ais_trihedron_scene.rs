#![allow(dead_code)]
//! Real AIS trihedrons exercise shaded datums and their part-owner highlights.
use crate::view_setup;
use martensite_vulkan::{Graphic3dRenderDriver, Graphic3dRenderView};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
    tcollection::ExtendedString,
};
use rustycad_tkg3d::geom::GeomAxis2Placement;
use rustycad_tkmath::gp::{Axis2, Point3};
use rustycad_tkservice::aspect::AspectNeutralWindow;
use rustycad_tkv3d::{ais::*, prs_mgr::*, prs3d::*, select_mgr::*, v3d::*};
pub const MODES: [&str; 16] = [
    "tri-wire",
    "tri-shaded",
    "tri-wire-noarrows",
    "tri-shaded-noarrows",
    "tri-labels",
    "tri-color",
    "tri-arrow-color",
    "tri-text-color",
    "tri-size",
    "tri-moved",
    "tri-selected",
    "tri-axis",
    "tri-origin",
    "tri-plane",
    "tri-shaded-axis",
    "tri-shaded-origin",
];
pub struct TrihedronScene {
    pub view: Handle<V3dView>,
    pub window: Handle<AspectNeutralWindow>,
    pub context: Handle<AisInteractiveContext>,
    pub object: Handle<AisTrihedron>,
}
impl TrihedronScene {
    pub fn new(driver: &Handle<Graphic3dRenderDriver>, mode: &str) -> Result<Self, Failure> {
        let viewer = V3dViewer::new(&driver.cast())?;
        let view = viewer.create_view()?;
        let window = view_setup::configure_view(&view);
        let context = AisInteractiveContext::new(&viewer)?;
        let mut frame = Axis2::default();
        frame.set_location(Point3::new(-3., -3., 0.));
        let object = AisTrihedron::new(&Handle::new(GeomAxis2Placement::new(frame)))?;
        object.set_size(6.)?;
        if mode.contains("shaded") {
            object.set_datum_display_mode(Prs3dDatumMode::Shaded);
        }
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
    pub fn apply(&self, mode: &str) -> Result<(), Failure> {
        match mode {
            "tri-wire-noarrows" | "tri-shaded-noarrows" => self.object.set_draw_arrows(false)?,
            "tri-labels" => {
                for (part, label) in [
                    (Prs3dDatumParts::XAxis, "U"),
                    (Prs3dDatumParts::YAxis, "V"),
                    (Prs3dDatumParts::ZAxis, "W"),
                ] {
                    self.object.set_label(part, &ExtendedString::new(label));
                }
            }
            "tri-color" => self.object.set_color(Color::named(NamedColor::RED))?,
            "tri-arrow-color" => self
                .object
                .set_arrow_color(Color::named(NamedColor::GREEN))?,
            "tri-text-color" => self.object.set_text_color(Color::named(NamedColor::CYAN))?,
            "tri-size" => self.object.set_size(4.)?,
            "tri-moved" => {
                let mut frame = Axis2::default();
                frame.set_location(Point3::new(-2., 1., 0.));
                self.object
                    .set_component(&Handle::new(GeomAxis2Placement::new(frame)));
            }
            "tri-selected" => return self.context.set_selected_object(&self.object.cast(), false),
            "tri-axis" | "tri-shaded-axis" => return self.highlight(Prs3dDatumParts::XAxis),
            "tri-origin" | "tri-shaded-origin" => return self.highlight(Prs3dDatumParts::Origin),
            "tri-plane" => return self.highlight(Prs3dDatumParts::XOYAxis),
            _ => (),
        }
        self.context.redisplay(&self.object.cast(), false, false)
    }
    fn highlight(&self, part: Prs3dDatumParts) -> Result<(), Failure> {
        let owner = AisTrihedronOwner::new(
            &self.object.cast(),
            part,
            self.object.selection_priority(part),
        );
        self.object
            .hilight_selected(&self.context.main_prs_mgr(), &[owner.cast()])
    }
}
