//! Replay real context/point selection without loading OCCT.
use super::*;
use crate::v3d::*;
use rustycad_tkernel::{ncollection::Vec2, standard::Handle};
use rustycad_tkg3d::geom::GeomCartesianPoint;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
use rustycad_tkservice::{
    aspect::AspectNeutralWindow, graphic3d::*, null_driver::NullGraphicDriver,
};

fn picking_scene(
    projection: i32,
) -> (
    Handle<V3dViewer>,
    Handle<V3dView>,
    Handle<AisInteractiveContext>,
) {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let view = viewer.create_view().unwrap();
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_size(80, 60);
    view.set_window(&window.cast(), 0).unwrap();
    configure_picking_camera(&view, projection);
    let context = AisInteractiveContext::new(&viewer).unwrap();
    let geometry = Handle::new(GeomCartesianPoint::new(0.2, 0.3, 2.));
    let point = AisPoint::new(&geometry.cast()).unwrap();
    context.display(&point.cast(), false).unwrap();
    (viewer, view, context)
}

fn configure_picking_camera(view: &V3dView, projection: i32) {
    let camera = view.camera();
    camera
        .set_eye_and_center(&Point3::new(0., 0., 20.), &Point3::default())
        .unwrap();
    camera.set_scale(20.);
    camera.set_aspect(4. / 3.);
    camera.set_projection_type(if projection == 0 {
        Graphic3dCameraProjection::Orthographic
    } else {
        Graphic3dCameraProjection::Perspective
    });
}

#[test]
fn controller_picking_replays_native_context_selection() {
    for projection in 0..2 {
        let (_viewer, view, context) = picking_scene(projection);
        let mut controller = AisViewController::new().unwrap();
        for line in include_str!("../../tests/fixtures/controller-picking-native.txt").lines() {
            let expected: Vec<f64> = line
                .split_whitespace()
                .map(|value| value.parse().unwrap())
                .collect();
            if expected[0] as i32 != projection {
                continue;
            }
            assert_picking_row(&mut controller, &context, &view, &expected);
        }
    }
}

fn assert_picking_row(
    controller: &mut AisViewController,
    context: &AisInteractiveContext,
    view: &V3dView,
    row: &[f64],
) {
    controller.previous_move_to = Vec2::new(14, 18);
    let mut point = Point3::new(97., 98., 99.);
    let hit = execute_picking_row(controller, context, view, row, &mut point);
    assert_eq!(hit, row[4] != 0., "{row:?}");
    for (actual, expected) in [point.x(), point.y(), point.z()]
        .into_iter()
        .zip(&row[5..8])
    {
        assert!(
            (actual - expected).abs() < 1e-10,
            "{row:?}: {actual} != {expected}"
        );
    }
    assert_eq!(controller.has_previous_move_to(), row[8] != 0.);
}

fn execute_picking_row(
    controller: &mut AisViewController,
    context: &AisInteractiveContext,
    view: &V3dView,
    row: &[f64],
    point: &mut Point3,
) -> bool {
    if row[1] == 2. {
        return execute_gravity_row(controller, context, view, row, point);
    }
    if row[1] == 0. {
        let pixel = Vec2::new(40 + row[2] as i32 * 5, 30);
        return controller
            .pick_point(point, context, view, &pixel, row[3] != 0.)
            .unwrap();
    }
    let axis = Axis1::new(
        Point3::new(row[2] * 0.2, 0.3, 20.),
        Direction3::new(0., 0., -1.).unwrap(),
    );
    controller.pick_axis(point, context, view, axis).unwrap()
}

#[test]
fn center_picking_requires_window_without_resetting_previous_pixel() {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let view = viewer.create_view().unwrap();
    let context = AisInteractiveContext::new(&viewer).unwrap();
    let mut controller = AisViewController::new().unwrap();
    controller.set_rotation_mode(AisRotationMode::PickCenter);
    controller.previous_move_to = Vec2::new(14, 18);
    assert!(
        controller
            .gravity_point(&context, &view)
            .unwrap_err()
            .to_string()
            .contains("window=null")
    );
    assert!(controller.has_previous_move_to());
    controller.set_rotation_mode(AisRotationMode::BndBoxScene);
    assert_eq!(
        controller.gravity_point(&context, &view).unwrap(),
        Point3::default()
    );
}

fn execute_gravity_row(
    controller: &mut AisViewController,
    context: &AisInteractiveContext,
    view: &V3dView,
    row: &[f64],
    point: &mut Point3,
) -> bool {
    let modes = [
        AisRotationMode::BndBoxActive,
        AisRotationMode::PickLast,
        AisRotationMode::PickCenter,
        AisRotationMode::CameraAt,
        AisRotationMode::BndBoxScene,
    ];
    controller.set_rotation_mode(modes[row[2] as usize]);
    controller.set_stick_to_ray_on_rotation(row[3] as i32 % 2 != 0);
    let start = if row[3] < 2. { 40.9 } else { 80.9 };
    controller.render_input.orbit_rotation.point_start = Vec2::new(start, 30.9);
    *point = controller.gravity_point(context, view).unwrap();
    true
}

#[test]
fn picking_failure_resets_highlight_before_volume_initialization() {
    let (viewer, _view, context) = picking_scene(0);
    let view = viewer.create_view().unwrap();
    let mut controller = AisViewController::new().unwrap();
    controller.previous_move_to = Vec2::new(1, 2);
    let mut point = Point3::new(97., 98., 99.);
    let error = controller
        .pick_point(&mut point, &context, &view, &Vec2::new(40, 30), true)
        .unwrap_err();
    assert!(error.to_string().contains("window=null"));
    assert_eq!(point, Point3::new(97., 98., 99.));
    assert!(!controller.has_previous_move_to());
}
