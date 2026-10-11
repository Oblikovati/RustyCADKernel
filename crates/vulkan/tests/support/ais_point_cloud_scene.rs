#![allow(dead_code)]
//! Actual Martensite point clouds exercise copied/aliased buffers and forced subsets.
use crate::view_setup;
use martensite_vulkan::{Graphic3dRenderDriver, Graphic3dRenderView};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::{aspect::AspectNeutralWindow, graphic3d::*};
use rustycad_tkv3d::{ais::*, prs_mgr::*, v3d::*};
pub const POINT_CLOUD_MODES: [&str; 11] = [
    "cloud-points",
    "cloud-colors",
    "cloud-normals-unlit",
    "cloud-box",
    "cloud-selected",
    "cloud-subset-selected",
    "cloud-subset-detected",
    "cloud-moved",
    "cloud-erased",
    "cloud-redisplayed",
    "cloud-unset-color",
];
pub struct PointCloudScene {
    pub viewer: Handle<V3dViewer>,
    pub view: Handle<V3dView>,
    pub window: Handle<AspectNeutralWindow>,
    pub context: Handle<AisInteractiveContext>,
    pub cloud: Handle<AisPointCloud>,
    pub offset: f64,
}
impl PointCloudScene {
    /// Retain a production AIS cloud and viewer; `PointCloudScene::new(&driver,"cloud-points",0.)`.
    pub fn new(driver: &Handle<Graphic3dRenderDriver>, mode: &str, offset: f64) -> Self {
        let viewer = V3dViewer::new(&driver.cast()).unwrap();
        let view = viewer.create_view().unwrap();
        let window = view_setup::configure_view(&view);
        let context = AisInteractiveContext::new(&viewer).unwrap();
        let cloud = capture_cloud(&context, mode, offset);
        let scene = Self {
            viewer,
            view,
            window,
            context,
            cloud,
            offset,
        };
        scene.transition(mode);
        scene
    }
    fn transition(&self, mode: &str) {
        let object = self.cloud.cast();
        match mode {
            "cloud-selected" => self.context.set_selected_object(&object, false).unwrap(),
            "cloud-erased" => self.context.erase(&object, false).unwrap(),
            "cloud-redisplayed" => {
                self.context.erase(&object, false).unwrap();
                self.context.display(&object, false).unwrap();
            }
            "cloud-unset-color" => {
                self.cloud.set_color(Color::named(NamedColor::RED)).unwrap();
                self.cloud.unset_color().unwrap();
            }
            "cloud-moved" => {
                self.cloud
                    .set_points(&cloud_points(7, self.offset + 1., false, false))
                    .unwrap();
                self.context.redisplay(&object, false, false).unwrap();
            }
            _ => {}
        }
    }
    /// Detect the central point and verify its native subset owner; `scene.detect_subset(true)`.
    pub fn detect_subset(&self, selected: bool) {
        let (x, y) = self
            .view
            .convert_world_to_pixels(self.offset, 0., 0.)
            .unwrap();
        self.context.move_to(x, y, &self.view, false).unwrap();
        let owner = self.context.detected_owner().cast::<AisPointCloudOwner>();
        assert!(
            !owner.is_null(),
            "expected AIS_PointCloudOwner at the central point"
        );
        if selected {
            self.context
                .select_detected(AisSelectionScheme::Replace)
                .unwrap();
        }
        let subset = if selected {
            owner.selected_points()
        } else {
            owner.detected_points()
        };
        assert!(
            !subset.map().is_empty(),
            "expected highlighted zero-based subset"
        );
    }
    /// Access the production redraw and readback path; `scene.render_view().try_redraw()?`.
    pub fn render_view(&self) -> Handle<Graphic3dRenderView> {
        self.view.get().unwrap().view().cast()
    }
    /// Rebuild a thousand points and recompute active selection; `scene.update_points(1)`.
    pub fn update_points(&self, index: usize) {
        self.cloud
            .set_points(&cloud_points(32, (index % 2) as f64 * 0.25, true, false))
            .unwrap();
        self.context
            .redisplay(&self.cloud.cast(), false, false)
            .unwrap();
    }
}
fn cloud_points(
    side: i32,
    offset: f64,
    colors: bool,
    normals: bool,
) -> Handle<Graphic3dArrayOfPoints> {
    let points =
        Handle::new(Graphic3dArrayOfPoints::with_options(side * side, colors, normals).unwrap());
    for position in 0..side * side {
        append_cloud_grid_point(&points, side, position, offset, colors, normals);
    }
    points
}

fn capture_cloud(
    context: &AisInteractiveContext,
    mode: &str,
    offset: f64,
) -> Handle<AisPointCloud> {
    let cloud = AisPointCloud::new().unwrap();
    cloud
        .attributes()
        .shading_aspect()
        .aspect()
        .set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
    let points = cloud_points(
        5,
        offset,
        mode == "cloud-colors",
        mode == "cloud-normals-unlit",
    );
    cloud.set_points(&points).unwrap();
    let display = if mode == "cloud-box" { 2 } else { 0 };
    let selection = if mode.contains("subset") { 1 } else { 0 };
    context
        .display_with_modes(
            &cloud.cast(),
            display,
            selection,
            false,
            PrsMgrDisplayStatus::None,
        )
        .unwrap();
    cloud
}

fn append_cloud_grid_point(
    points: &Graphic3dArrayOfPoints,
    side: i32,
    position: i32,
    offset: f64,
    colors: bool,
    normals: bool,
) {
    let (row, col) = (position / side, position % side);
    let half = f64::from(side - 1) / 2.;
    let point = Point3::new(
        (f64::from(col) - half) * 1.25 + offset,
        (f64::from(row) - half) * 1.25,
        0.,
    );
    let index = points.add_vertex_pnt(&point).unwrap();
    if colors {
        let color = if (row + col) % 2 == 0 {
            NamedColor::RED
        } else {
            NamedColor::GREEN
        };
        points.set_vertex_color(index, Color::named(color)).unwrap();
    }
    if normals {
        points.set_vertex_normal(index, 0., 0., 1.).unwrap();
    }
}
