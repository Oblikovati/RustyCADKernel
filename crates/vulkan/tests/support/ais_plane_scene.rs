#![allow(dead_code)]
//! Actual AIS planes retain native geometry, appearance, selection and transparency.
use super::view_setup;
use martensite_vulkan::{Graphic3dRenderDriver, Graphic3dRenderView};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
};
use rustycad_tkg3d::geom::{GeomAxis2Placement, GeomPlane};
use rustycad_tkmath::gp::{Axis2, Axis3, Direction3, Point3};
use rustycad_tkservice::{
    aspect::{AspectNeutralWindow, AspectTypeOfFacingModel},
    graphic3d::*,
};
use rustycad_tkv3d::{ais::*, prs_mgr::*, v3d::*};
pub const AIS_PLANE_MODES: [&str; 17] = [
    "aisplane-wire",
    "aisplane-current",
    "aisplane-manual",
    "aisplane-xy",
    "aisplane-xz",
    "aisplane-yz",
    "aisplane-shaded",
    "aisplane-shaded-alpha",
    "aisplane-shaded-opaque",
    "aisplane-shaded-xyz",
    "aisplane-minimum",
    "aisplane-sized",
    "aisplane-selected",
    "aisplane-detected",
    "aisplane-erased",
    "aisplane-redisplayed",
    "aisplane-unset-color",
];
pub struct AisPlaneScene {
    pub viewer: Handle<V3dViewer>,
    pub view: Handle<V3dView>,
    pub window: Handle<AspectNeutralWindow>,
    pub context: Handle<AisInteractiveContext>,
    pub object: Handle<AisPlane>,
    pub shaded: Handle<Graphic3dStructure>,
}
impl AisPlaneScene {
    /// Retain a real datum and its viewer; `AisPlaneScene::new(&driver,"aisplane-wire",0.)`.
    pub fn new(driver: &Handle<Graphic3dRenderDriver>, mode: &str, offset: f64) -> Self {
        let viewer = V3dViewer::new(&driver.cast()).unwrap();
        let view = viewer.create_view().unwrap();
        let window = view_setup::configure_view(&view);
        let context = AisInteractiveContext::new(&viewer).unwrap();
        let object = capture_object(&context, mode, offset);
        let shaded = shaded_plane_structure(&viewer, mode);
        let scene = Self {
            viewer,
            view,
            window,
            context,
            object,
            shaded,
        };
        scene.display(mode);
        scene
    }
    /// Access the production readback path; `scene.render_view().try_redraw()?`.
    pub fn render_view(&self) -> Handle<Graphic3dRenderView> {
        self.view.get().unwrap().view().cast()
    }
    fn display(&self, mode: &str) {
        if !self.shaded.is_null() {
            self.refresh_shaded();
            self.shaded.display();
            return;
        }
        self.context.display(&self.object.cast(), false).unwrap();
        self.transition(mode);
    }
    /// Recompute shaded geometry using live AIS aspects; `scene.refresh_shaded()`.
    pub fn refresh_shaded(&self) {
        self.shaded.clear(false);
        self.object
            .compute(&Handle::default(), &self.shaded.cast(), 1)
            .unwrap();
    }
    fn transition(&self, mode: &str) {
        let object = self.object.cast();
        match mode {
            "aisplane-erased" => self.context.erase(&object, false).unwrap(),
            "aisplane-redisplayed" => {
                self.context.erase(&object, false).unwrap();
                self.context.display(&object, false).unwrap();
            }
            "aisplane-selected" => self.context.set_selected_object(&object, false).unwrap(),
            "aisplane-sized" => self.object.set_size_xy(6., 8.).unwrap(),
            "aisplane-unset-color" => {
                self.object
                    .set_color(Color::named(NamedColor::RED))
                    .unwrap();
                self.object.unset_color().unwrap();
                self.context.redisplay(&object, false, false).unwrap();
            }
            _ => {}
        }
    }
    /// Pick a boundary after final viewport configuration; `scene.detect()`.
    pub fn detect(&self) {
        let c = self.object.center();
        let (x, y) = self
            .view
            .convert_world_to_pixels(c.x() + 6., c.y(), c.z())
            .unwrap();
        self.context.move_to(x, y, &self.view, false).unwrap();
        assert_eq!(
            self.context.detected_interactive(),
            self.object.cast::<dyn AisInteractiveObject>()
        );
    }
}
fn shaded_plane_structure(viewer: &V3dViewer, mode: &str) -> Handle<Graphic3dStructure> {
    if !mode.contains("shaded") {
        return Handle::default();
    }
    Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap()
}
fn capture_object(
    ctx: &Handle<AisInteractiveContext>,
    mode: &str,
    offset: f64,
) -> Handle<AisPlane> {
    let mut frame = Axis3::default();
    frame.set_location(Point3::new(offset, 0., 0.));
    let geometry = Handle::new(GeomPlane::new(frame));
    let object = construct_plane_object(&geometry, mode, offset);
    object.set_context(ctx);
    object.set_size_xy(12., 10.).unwrap();
    configure_plane_shading(&object, mode);
    configure_xyz(&object, mode, offset);
    if mode == "aisplane-minimum" {
        object.set_minimum_size(30.).unwrap();
    }
    object
}
fn construct_plane_object(
    geometry: &Handle<GeomPlane>,
    mode: &str,
    offset: f64,
) -> Handle<AisPlane> {
    if mode == "aisplane-manual" {
        AisPlane::with_bounds(
            geometry,
            Point3::new(offset, 0., 0.),
            Point3::new(offset - 3., -2., 0.),
            Point3::new(99., 99., 99.),
            true,
        )
        .unwrap()
    } else {
        AisPlane::with_center(
            geometry,
            Point3::new(offset, 0., 0.),
            mode == "aisplane-current",
        )
        .unwrap()
    }
}
fn configure_plane_shading(object: &AisPlane, mode: &str) {
    let aspect = object.attributes().shading_aspect().aspect();
    aspect.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
    aspect.set_polygon_offsets(0, 0., 0.);
    if mode == "aisplane-shaded-alpha" {
        object
            .attributes()
            .shading_aspect()
            .set_transparency(0.8, AspectTypeOfFacingModel::BothSide)
            .unwrap();
    }
    if mode == "aisplane-shaded-opaque" {
        object
            .attributes()
            .shading_aspect()
            .set_transparency(0., AspectTypeOfFacingModel::BothSide)
            .unwrap();
    }
}
fn configure_xyz(object: &AisPlane, mode: &str, offset: f64) {
    let kind = match mode {
        "aisplane-xy" | "aisplane-shaded-xyz" | "aisplane-sized" => AisTypeOfPlane::XYPlane,
        "aisplane-xz" => AisTypeOfPlane::XZPlane,
        "aisplane-yz" => AisTypeOfPlane::YZPlane,
        _ => return,
    };
    let frame = Axis2::with_x_direction(
        Point3::new(offset - 3., -3., 0.),
        Direction3::new(0., 0.6, 0.8).unwrap(),
        Direction3::X,
    )
    .unwrap();
    object
        .set_axis2_placement(&Handle::new(GeomAxis2Placement::new(frame)), kind)
        .unwrap();
}
