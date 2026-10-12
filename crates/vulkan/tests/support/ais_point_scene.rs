#![allow(dead_code)]
//! Built-in point datums and actual native screen-space marker styles.
use super::view_setup;
use martensite_vulkan::{Graphic3dRenderDriver, Graphic3dRenderView};
use rustycad_tkernel::{
    ncollection::HArray1,
    quantity::{Color, NamedColor},
    standard::{Failure, Handle, SharedState},
};
use rustycad_tkg3d::geom::GeomCartesianPoint;
use rustycad_tkservice::{
    aspect::{AspectNeutralWindow, AspectTypeOfMarker},
    graphic3d::*,
};
use rustycad_tkv3d::v3d::V3dViewSource;
use rustycad_tkv3d::{ais::*, prs_mgr::*, prs3d::Prs3dPointAspect, v3d::*};
pub const MODES: [&str; 24] = [
    "point-plus",
    "point-dot",
    "point-star",
    "point-x",
    "point-o",
    "point-o-dot",
    "point-o-plus",
    "point-o-star",
    "point-o-x",
    "point-ring1",
    "point-ring2",
    "point-ring3",
    "point-ball",
    "point-empty",
    "point-user",
    "point-color",
    "point-scale",
    "point-unset",
    "point-moved",
    "point-erased",
    "point-redisplayed",
    "point-selected",
    "point-detected",
    "point-batch",
];
pub struct PointScene {
    pub view: Handle<dyn V3dViewSource>,
    pub window: Handle<AspectNeutralWindow>,
    pub context: Handle<AisInteractiveContext>,
    pub point: Handle<AisPoint>,
    batch_structure: SharedState<Handle<Graphic3dStructure>>,
}
impl PointScene {
    pub fn new(driver: &Handle<Graphic3dRenderDriver>) -> Self {
        let viewer = V3dViewer::new(&driver.cast()).unwrap();
        let view = viewer.create_view().unwrap();
        let window = view_setup::configure_view(&view);
        let context = AisInteractiveContext::new(&viewer).unwrap();
        let point =
            AisPoint::new(&Handle::new(GeomCartesianPoint::new(0., 0., 0.)).cast()).unwrap();
        context.display(&point.cast(), false).unwrap();
        Self {
            view,
            window,
            context,
            point,
            batch_structure: SharedState::new(Handle::default()),
        }
    }
    pub fn apply(&self, mode: &str) -> Result<(), Failure> {
        if let Some(kind) = marker_kind(mode) {
            return self.point.set_marker(kind);
        }
        match mode {
            "point-plus" => Ok(()),
            "point-color" => self.point.set_color(Color::named(NamedColor::RED)),
            "point-scale" => self.change_scale(),
            "point-unset" => self.restore_marker(),
            "point-moved" => self.move_component(),
            "point-erased" => self.context.erase(&self.point.cast(), false),
            "point-redisplayed" => self.redisplay(),
            "point-selected" => self.context.set_selected_object(&self.point.cast(), false),
            "point-detected" => self.detect(),
            "point-user" => self.user_marker(),
            "point-batch" => self.batch(),
            _ => panic!("received {mode}; expected point capture mode"),
        }
    }
    fn change_scale(&self) -> Result<(), Failure> {
        self.point.set_width(3.)?;
        self.point.set_marker(AspectTypeOfMarker::Plus)
    }
    fn restore_marker(&self) -> Result<(), Failure> {
        self.point.set_marker(AspectTypeOfMarker::Star)?;
        self.point.unset_marker()
    }
    fn move_component(&self) -> Result<(), Failure> {
        self.point
            .set_component(&Handle::new(GeomCartesianPoint::new(4., -2., 1.)).cast());
        self.context.redisplay(&self.point.cast(), false, false)
    }
    fn redisplay(&self) -> Result<(), Failure> {
        self.context.erase(&self.point.cast(), false)?;
        self.context.display(&self.point.cast(), false)
    }
    fn detect(&self) -> Result<(), Failure> {
        let p = self.point.transformation().translation_part();
        let (x, y) = self.view.convert_world_to_pixels(p.x(), p.y(), p.z())?;
        self.context.move_to(x, y, &self.view, false)?;
        assert_eq!(
            self.context.detected_interactive().object_address(),
            self.point.object_address()
        );
        Ok(())
    }
    fn user_marker(&self) -> Result<(), Failure> {
        self.point
            .attributes()
            .set_point_aspect(Handle::new(Prs3dPointAspect::with_texture(
                Color::named(NamedColor::RED),
                8,
                8,
                asymmetric_bitmap(),
            )));
        self.context.redisplay(&self.point.cast(), false, false)
    }
    pub fn batch(&self) -> Result<(), Failure> {
        self.context.erase(&self.point.cast(), false)?;
        let structure =
            Graphic3dStructure::new(&self.view.viewer().structure_manager(), &Handle::default())?;
        populate_batch(&structure)?;
        structure.set_transformation(&Handle::new(rustycad_tkmath::toploc::Datum3::new(
            self.point.transformation(),
        )));
        structure.display();
        *self.batch_structure.write() = structure;
        Ok(())
    }
    pub fn render_view(&self) -> Handle<Graphic3dRenderView> {
        self.view.get().unwrap().view().cast()
    }
}
fn marker_kind(mode: &str) -> Option<AspectTypeOfMarker> {
    use AspectTypeOfMarker::*;
    match mode {
        "point-dot" => Some(Point),
        "point-star" => Some(Star),
        "point-x" => Some(X),
        "point-o" => Some(O),
        "point-o-dot" => Some(OPoint),
        "point-o-plus" => Some(OPlus),
        "point-o-star" => Some(OStar),
        "point-o-x" => Some(OX),
        "point-ring1" => Some(Ring1),
        "point-ring2" => Some(Ring2),
        "point-ring3" => Some(Ring3),
        "point-ball" => Some(Ball),
        "point-empty" => Some(Empty),
        _ => None,
    }
}

fn asymmetric_bitmap() -> Handle<HArray1<u8>> {
    let bits = Handle::new(HArray1::<u8>::with_bounds(1, 8).unwrap());
    for (i, v) in [0x80, 0xc0, 0xe0, 0xf0, 0xf8, 0xfc, 0xfe, 0xff]
        .into_iter()
        .enumerate()
    {
        bits.change_array1().set_value(i as i32 + 1, v).unwrap();
    }
    bits
}

fn populate_batch(structure: &Handle<Graphic3dStructure>) -> Result<(), Failure> {
    let group = structure.new_group()?;
    group.set_group_primitives_aspect(
        &Handle::new(Graphic3dAspectMarker3d::new(
            AspectTypeOfMarker::Plus,
            Color::named(NamedColor::YELLOW),
            1.,
        )?)
        .cast(),
    );
    group.add_primitive_array(&batch_vertices()?.cast(), true);
    Ok(())
}
fn batch_vertices() -> Result<Handle<Graphic3dArrayOfPoints>, Failure> {
    let vertices = Handle::new(Graphic3dArrayOfPoints::new(1024, 0)?);
    for i in 0..1024 {
        vertices.add_vertex_f64(
            f64::from(i % 32) * 0.5 - 8.,
            f64::from(i / 32) * 0.5 - 8.,
            0.,
        )?;
    }
    Ok(vertices)
}
