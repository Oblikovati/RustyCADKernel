#![allow(dead_code)]
//! Actual Martensite point clouds exercise copied/aliased buffers and forced subsets.
use crate::view_setup;
use martensite_vulkan::{Graphic3dRenderDriver, Graphic3dRenderView};
use rustycad_tkernel::{
    quantity::{Color, ColorSpace, NamedColor},
    standard::Handle,
};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::{aspect::AspectNeutralWindow, graphic3d::*};
use rustycad_tkv3d::{ais::*, prs_mgr::*, v3d::*};
pub const POINT_CLOUD_MODES: [&str; 16] = [
    "cloud-points",
    "cloud-colors",
    "cloud-normals-unlit",
    "cloud-lit-phong",
    "cloud-lit-gouraud",
    "cloud-lit-colors",
    "cloud-lit-clipped",
    "cloud-lit-normal-updated",
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
        scene.configure_marker_lights(mode);
        scene.transition(mode);
        scene
    }
    fn transition(&self, mode: &str) {
        let object = self.cloud.cast();
        match mode {
            "cloud-lit-normal-updated" => {
                self.cloud
                    .get_points()
                    .set_vertex_normal(13, 1., 0., 0.)
                    .unwrap();
                self.context.redisplay(&object, false, false).unwrap();
            }
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
            .set_points(&cloud_points(
                32,
                (index % 2) as f64 * 0.25,
                true,
                self.cloud.get_points().has_vertex_normals(),
            ))
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
        .set_shading_model(if mode == "cloud-lit-gouraud" {
            Graphic3dTypeOfShadingModel::Gouraud
        } else if mode.starts_with("cloud-lit-") {
            Graphic3dTypeOfShadingModel::Phong
        } else {
            Graphic3dTypeOfShadingModel::Unlit
        });
    let points = cloud_points(
        5,
        offset,
        mode == "cloud-colors" || mode == "cloud-lit-colors",
        mode == "cloud-normals-unlit" || mode.starts_with("cloud-lit-"),
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
        points
            .set_vertex_normal(
                index,
                if col % 2 == 0 { 0. } else { 0.6 },
                0.,
                if col % 2 == 0 { 1. } else { 0.8 },
            )
            .unwrap();
    }
}

impl PointCloudScene {
    fn configure_marker_lights(&self, mode: &str) {
        if !mode.starts_with("cloud-lit-") {
            return;
        }
        self.view.change_rendering_params(&mut |params| {
            params.shading_model = Graphic3dTypeOfShadingModel::Phong
        });
        let ambient = Handle::new(Graphic3dCLight::new(Graphic3dTypeOfLightSource::Ambient));
        ambient.set_color(Color::new(0.125, 0.25, 0.5, ColorSpace::Rgb).unwrap());
        ambient.set_intensity(2.).unwrap();
        let directional = Handle::new(Graphic3dCLight::new(
            Graphic3dTypeOfLightSource::Directional,
        ));
        directional.set_direction_coords(0., 0., -1.).unwrap();
        directional.set_color(Color::new(0.75, 0.5, 0.25, ColorSpace::Rgb).unwrap());
        let lights = Handle::new(Graphic3dLightSet::default());
        lights.add(&ambient).unwrap();
        lights.add(&directional).unwrap();
        self.view.get().unwrap().view().set_lights(&lights);
        if mode != "cloud-lit-clipped" {
            return;
        }
        let planes = Handle::new(Graphic3dSequenceOfHClipPlane::default());
        planes.append(Handle::new(
            Graphic3dClipPlane::from_equation(rustycad_tkernel::ncollection::Vec4::new(
                1.,
                0.,
                0.,
                -self.offset,
            ))
            .unwrap(),
        ));
        self.view.set_clip_planes(&planes);
    }
    /// Time live nodal updates without rebuilding static sprite geometry; `scene.update_lit_normals(1)`.
    pub fn update_lit_normals(&self, index: usize) {
        let points = self.cloud.get_points();
        let center = cloud_grid_center_index(points.vertex_number());
        points
            .set_vertex_normal(
                center,
                if index.is_multiple_of(2) { 0. } else { 0.6 },
                0.,
                0.8,
            )
            .unwrap();
    }
}

fn cloud_grid_center_index(vertices: i32) -> i32 {
    let side = (f64::from(vertices)).sqrt() as i32;
    // Even grids have two middle rows; choose a visible center rather than a row endpoint.
    vertices / 2 + 1 + if side % 2 == 0 { side / 2 } else { 0 }
}
#[cfg(test)]
mod cloud_grid_center_tests {
    #[test]
    fn normal_update_targets_the_center_of_odd_and_even_grid_fixtures() {
        assert_eq!(super::cloud_grid_center_index(25), 13);
        assert_eq!(super::cloud_grid_center_index(1024), 529);
    }
}
