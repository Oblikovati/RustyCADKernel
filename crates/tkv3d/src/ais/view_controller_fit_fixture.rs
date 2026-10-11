//! Actual context geometry, selections and view cameras mirror the local native oracle.
use super::*;
use crate::select3d::Select3DSensitiveEntityMembers as _;
use crate::{
    prs_mgr::{PrsMgrDisplayStatus, PrsMgrPresentableObjectMembers},
    select_mgr::*,
    v3d::*,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::GeomAxis2Placement;
use rustycad_tkmath::{
    bnd::BoundingBox3,
    gp::{Axis2, Direction3, Point3, Transform3, Vector3},
    toploc::Location,
};
use rustycad_tkservice::{
    aspect::AspectNeutralWindow, graphic3d::*, null_driver::NullGraphicDriver,
};
pub(super) struct NativeFitScene {
    _viewer: Handle<V3dViewer>,
    pub(super) view: Handle<V3dView>,
    pub(super) context: Handle<AisInteractiveContext>,
    pub(super) selected: Handle<AisPointCloud>,
    pub(super) outer: Handle<AisPointCloud>,
    pub(super) trihedron: Handle<AisTrihedron>,
}
pub(super) fn fit_scene(index: usize) -> NativeFitScene {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let view = viewer.create_view().unwrap();
    configure_fit_view(&view, index);
    let context = AisInteractiveContext::new(&viewer).unwrap();
    let mut scene = NativeFitScene {
        _viewer: viewer,
        view,
        context,
        selected: fit_cloud(false),
        outer: fit_cloud(true),
        trihedron: fit_trihedron(),
    };
    degenerate_fit_cloud(&scene, index);
    configure_fit_objects(&scene, index);
    transform_fit_objects(&scene, index);
    select_fit_scene(&mut scene, index);
    scene
}
fn fit_trihedron() -> Handle<AisTrihedron> {
    let frame = Axis2::new(
        Point3::new(5.0, 6.0, 7.0),
        Direction3::from_standard(rustycad_tkmath::gp::StandardDirection3::Z),
    )
    .unwrap();
    let trihedron = AisTrihedron::new(&Handle::new(GeomAxis2Placement::new(frame))).unwrap();
    trihedron.set_size(20.0).unwrap();
    trihedron
}

fn configure_fit_view(view: &V3dView, index: usize) {
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_size(640, 480);
    view.set_window(&window.cast(), 0).unwrap();
    view.set_immediate_update(false);
    let camera = view.camera();
    camera
        .set_eye_and_center(&Point3::new(0.0, 0.0, 200.0), &Point3::default())
        .unwrap();
    camera.set_scale(200.0);
    camera.set_aspect(4.0 / 3.0);
    configure_fit_projection(&camera, index);
}
fn configure_fit_projection(camera: &Graphic3dCamera, index: usize) {
    camera.set_projection_type(if index / 256 == 0 {
        Graphic3dCameraProjection::Orthographic
    } else {
        Graphic3dCameraProjection::Perspective
    });
}
fn fit_cloud(outer: bool) -> Handle<AisPointCloud> {
    let points = Handle::new(Graphic3dArrayOfPoints::new(2, 0).unwrap());
    let coordinates = if outer {
        [[-20.0, -30.0, -40.0], [40.0, 50.0, 60.0]]
    } else {
        [[-2.0, -3.0, -4.0], [1.0, 5.0, 7.0]]
    };
    for [x, y, z] in coordinates {
        points.add_vertex_f64(x, y, z).unwrap();
    }
    let object = AisPointCloud::new().unwrap();
    object.set_points(&points).unwrap();
    object
}
fn degenerate_fit_cloud(scene: &NativeFitScene, index: usize) {
    if index % 8 != 7 {
        return;
    }
    let points = Handle::new(Graphic3dArrayOfPoints::new(1, 0).unwrap());
    points.add_vertex_f64(1.0, 2.0, 3.0).unwrap();
    scene.selected.set_points(&points).unwrap();
}

fn configure_fit_objects(s: &NativeFitScene, index: usize) {
    let flags = (index / 16) % 16;
    s.selected.set_infinite_state(flags & 4 != 0);
    s.trihedron.set_infinite_state(flags & 8 != 0);
    s.context.display(&s.selected.cast(), false).unwrap();
    s.context.display(&s.outer.cast(), false).unwrap();
    s.context
        .display_with_modes(&s.trihedron.cast(), 0, 2, false, PrsMgrDisplayStatus::None)
        .unwrap();
    s.context
        .set_view_affinity(&s.selected.cast(), &s.view, flags & 1 == 0)
        .unwrap();
    s.context
        .set_view_affinity(&s.trihedron.cast(), &s.view, flags & 2 == 0)
        .unwrap();
}
fn transform_fit_objects(s: &NativeFitScene, index: usize) {
    if (index / 8).is_multiple_of(2) {
        return;
    }
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(10.0, -4.0, 3.0));
    let location = Location::new(transform);
    s.context
        .set_location(&s.selected.cast(), &location)
        .unwrap();
    s.context
        .set_location(&s.trihedron.cast(), &location)
        .unwrap();
}
fn fit_axis_owner(s: &NativeFitScene, rank: i32) -> AisSelectedOwner {
    s.trihedron
        .selection(2)
        .entities()
        .get(rank as usize)
        .unwrap()
        .base_sensitive()
        .owner_id()
}
fn select_fit_scene(s: &mut NativeFitScene, index: usize) {
    for owner in fit_selected_owners(s, index % 8) {
        s.context.selection().add_select(&owner).unwrap();
    }
}
fn fit_selected_owners(s: &NativeFitScene, selection: usize) -> Vec<AisSelectedOwner> {
    let selected = s.selected.global_sel_owner();
    let outer = s.outer.global_sel_owner();
    match selection {
        0 => vec![],
        1 | 7 => vec![selected],
        2 => vec![selected, outer],
        3 => vec![fit_axis_owner(s, 1)],
        4 => vec![fit_axis_owner(s, 1), fit_axis_owner(s, 2)],
        5 => vec![selected, fit_axis_owner(s, 1)],
        6 => vec![fit_axis_owner(s, 1), fit_axis_owner(s, 1)],
        other => panic!("selection {other}; expected 0..=7"),
    }
}

pub(super) fn fit_snapshot(s: &NativeFitScene) -> Vec<f64> {
    let mut values = vec![f64::from(s.context.nb_selected())];
    append_fit_bounds(
        &mut values,
        &s.context.bounding_box_of_selection(Some(&s.view)).unwrap(),
    );
    append_fit_bounds(
        &mut values,
        &s.context.bounding_box_of_selection(None).unwrap(),
    );
    append_fit_bounds(
        &mut values,
        &s.view.get().unwrap().view().min_max_values(false).unwrap(),
    );
    append_fit_camera(&mut values, &s.view.camera());
    values
}
fn append_fit_bounds(values: &mut Vec<f64>, bounds: &BoundingBox3) {
    values.push(f64::from(bounds.is_void()));
    if bounds.is_void() {
        values.extend([0.0; 6]);
        return;
    }
    values.extend(bounds.get().unwrap());
}
fn append_fit_camera(values: &mut Vec<f64>, c: &Graphic3dCamera) {
    for point in [c.center(), c.eye()] {
        values.extend([point.x(), point.y(), point.z()]);
    }
    for direction in [c.up(), c.direction()] {
        values.extend([direction.x(), direction.y(), direction.z()]);
    }
    values.extend([
        c.scale(),
        c.distance(),
        c.aspect(),
        f64::from(c.projection_type() as i32),
        c.z_near(),
        c.z_far(),
        c.fov_y(),
    ]);
}
