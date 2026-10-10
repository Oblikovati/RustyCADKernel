//! V3d viewer and view behaviour on the TKService null driver: view creation and activation,
//! window attachment, camera orientation, sizes, fits, conversions, lights, grids and dumps.
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};
use rustycad_tkservice::aspect::{
    AspectGridDrawMode, AspectGridType, AspectNeutralWindow, AspectWindow,
};
use rustycad_tkservice::null_driver::NullGraphicDriver;
use rustycad_tkv3d::v3d::{V3d, V3dTypeOfOrientation, V3dView, V3dViewer};

/// A viewer on the null driver with the default lights on.
fn new_viewer() -> Handle<V3dViewer> {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    viewer.set_default_lights().unwrap();
    viewer.set_light_on().unwrap();
    viewer
}

/// A view of `viewer` on a 400x300 virtual window.
fn new_view(viewer: &Handle<V3dViewer>) -> Handle<V3dView> {
    let view = viewer.create_view().unwrap();
    let window = AspectNeutralWindow::default();
    window.set_size(400, 300);
    let window: Handle<dyn AspectWindow> = Handle::new(window).cast();
    view.set_window(&window, 0).unwrap();
    view
}

#[test]
fn a_new_view_is_defined_and_activated_by_set_window() {
    let viewer = new_viewer();
    assert!(viewer.if_more_views());
    let view = new_view(&viewer);
    assert!(view.if_window());
    assert_eq!(viewer.defined_views().len(), 1);
    assert_eq!(viewer.active_views().len(), 1);
    viewer.set_view_off_one(&view);
    assert!(viewer.active_views().is_empty());
    viewer.set_view_on_one(&view).unwrap();
    assert_eq!(viewer.active_views().len(), 1);
}

#[test]
fn removing_a_view_forgets_it_in_the_viewer() {
    let viewer = new_viewer();
    let view = new_view(&viewer);
    view.remove();
    assert!(viewer.defined_views().is_empty());
    assert!(viewer.active_views().is_empty());
}

#[test]
fn set_proj_orientation_points_the_eye_along_the_axis() {
    let viewer = new_viewer();
    let view = new_view(&viewer);
    view.set_proj_orientation(V3dTypeOfOrientation::Zpos, false)
        .unwrap();
    let (vx, vy, vz) = view.proj();
    assert!(
        vx.abs() < 1e-12 && vy.abs() < 1e-12 && (vz - 1.0).abs() < 1e-12,
        "proj ({vx}, {vy}, {vz})"
    );
    let (ux, uy, uz) = view.up();
    assert!(
        ux.abs() < 1e-12 && (uy - 1.0).abs() < 1e-12 && uz.abs() < 1e-12,
        "up ({ux}, {uy}, {uz})"
    );
}

#[test]
fn set_size_and_zero_depth_follow_the_native_checks() {
    let viewer = new_viewer();
    let view = new_view(&viewer);
    view.set_size(250.0).unwrap();
    assert!((view.size().0 - 250.0).abs() < 1e-9);
    assert!(view.set_size(0.0).is_err());
    assert!(view.set_depth(0.0).is_err());
    assert!(view.set_proj(0.0, 0.0, 0.0).is_err());
}

#[test]
fn fit_all_on_an_empty_viewer_keeps_the_camera_valid() {
    let viewer = new_viewer();
    let view = new_view(&viewer);
    view.fit_all(0.01, true).unwrap();
    let (x, y, z) = view.eye();
    assert!(x.is_finite() && y.is_finite() && z.is_finite());
}

#[test]
fn pixel_and_view_conversions_are_inverse() {
    let viewer = new_viewer();
    let view = new_view(&viewer);
    view.set_size(400.0).unwrap();
    let (u, v) = view.convert_to_view(100, 50).unwrap();
    let (px, py) = view.convert_view_to_pixels(u, v).unwrap();
    // RealToInt truncates natively, so a round trip may land one pixel short.
    assert_eq!(px, 100);
    assert!(
        py == 50 || py == 49,
        "pixel y {py}; expected 50 or the truncated 49"
    );
}

#[test]
fn rectangular_grid_values_round_trip_through_the_viewer() {
    let viewer = new_viewer();
    viewer
        .activate_grid(AspectGridType::Rectangular, AspectGridDrawMode::Lines)
        .unwrap();
    assert!(viewer.is_grid_active().unwrap());
    viewer
        .set_rectangular_grid_values(1.0, 2.0, 5.0, 6.0, 0.0)
        .unwrap();
    let (x_origin, y_origin, x_step, y_step, rotation) = viewer.rectangular_grid_values().unwrap();
    assert_eq!(
        (x_origin, y_origin, x_step, y_step, rotation),
        (1.0, 2.0, 5.0, 6.0, 0.0)
    );
    viewer.deactivate_grid().unwrap();
    assert!(!viewer.is_grid_active().unwrap());
}

#[test]
fn privileged_plane_is_stored() {
    let viewer = new_viewer();
    let plane = Axis3::new(
        Point3::new(1.0, 2.0, 3.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    )
    .unwrap();
    viewer.set_privileged_plane(&plane).unwrap();
    assert_eq!(
        viewer.privileged_plane().location(),
        Point3::new(1.0, 2.0, 3.0)
    );
}

#[test]
fn view_orientation_names_round_trip_except_the_native_typo() {
    for orientation in V3dTypeOfOrientation::ALL {
        let name = V3d::type_of_orientation_to_string(orientation);
        // V3d_Table_PrintTypeOfOrientation prints YposZpos as "XPOSZPOS".
        let expected = if orientation == V3dTypeOfOrientation::YposZpos {
            V3dTypeOfOrientation::XposZpos
        } else {
            orientation
        };
        assert_eq!(V3d::type_of_orientation_from_string(name), expected);
    }
}

#[test]
fn viewer_and_view_dump_json() {
    let viewer = new_viewer();
    let view = new_view(&viewer);
    let mut stream = DumpStream::default();
    viewer.dump_json(&mut stream, 1).unwrap();
    view.dump_json(&mut stream, 1).unwrap();
    assert!(stream.text().contains("\"V3d_Viewer\"") && stream.text().contains("\"V3d_View\""));
}
