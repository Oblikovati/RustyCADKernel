use super::face_face_task::BOPAlgoFaceFaceTask;
use crate::bop_algo::ParallelAlgo;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::{
    bnd::BoundingBox3,
    gp::{Axis3, Direction3, Point3},
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

#[test]
fn precision_translation_task_matches_native_plane_intersection() {
    let mut task = configured_task(true);
    task.set_indices(12, 21);
    task.set_parameters(true, true, true, 1e-7);
    task.set_tol_ff(0.125);
    task.set_fuzzy_value(1e-7);
    task.set_context(&Handle::new(crate::int_tools::Context::default()));
    task.intersector_mut().set_list(&[]);
    assert_eq!(task.indices(), [12, 21]);
    assert_eq!(task.tol_ff(), 0.125);
    assert!(!task.face1().is_null());
    assert!(!task.face2().is_null());
    task.perform_parallel().unwrap();
    assert!(task.intersector().is_done());
    task.apply_trsf().unwrap();
    assert_native_curve_row(&task, true);
}

#[test]
fn nearby_face_task_keeps_identity_transform() {
    let mut task = configured_task(false);
    task.perform_parallel().unwrap();
    assert!(task.intersector().is_done());
    assert_eq!(task.trsf().translation_part().x(), 0.0);
    assert_native_curve_row(&task, false);
}

fn configured_task(far: bool) -> BOPAlgoFaceFaceTask<'static> {
    let (faces, boxes) = if far {
        (far_plane_faces(), far_plane_boxes())
    } else {
        (near_plane_faces(), near_plane_boxes())
    };
    let mut task = BOPAlgoFaceFaceTask::default();
    task.set_faces(&faces[0], &faces[1]);
    task.set_boxes(&boxes[0], &boxes[1]);
    task
}

fn far_plane_faces() -> [Face; 2] {
    [
        bounded_plane(Point3::new(1e9, 0., 1e9), Direction3::Z, Direction3::X),
        bounded_plane(
            Point3::new(1e9 + 0.5, 0., 1e9),
            Direction3::X,
            Direction3::Y,
        ),
    ]
}

fn near_plane_faces() -> [Face; 2] {
    [
        bounded_plane(Point3::default(), Direction3::Z, Direction3::X),
        bounded_plane(Point3::new(0.5, 0., 0.), Direction3::X, Direction3::Y),
    ]
}

fn bounded_plane(origin: Point3, normal: Direction3, x_direction: Direction3) -> Face {
    let frame = Axis3::new(origin, normal, x_direction).unwrap();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(frame)).cast();
    BRepBuilderAPIMakeFace::from_surface_bounds(&surface, (-2., 2., -2., 2.), 1e-7)
        .unwrap()
        .face()
        .unwrap()
}

fn far_plane_boxes() -> [BoundingBox3; 2] {
    let minimum = Point3::new(1e9 - 2., -2., 1e9 - 2.);
    let maximum = Point3::new(1e9 + 2., 2., 1e9 + 2.);
    [
        BoundingBox3::new(minimum, maximum),
        BoundingBox3::new(minimum, maximum),
    ]
}

fn near_plane_boxes() -> [BoundingBox3; 2] {
    let minimum = Point3::new(-2., -2., -2.);
    let maximum = Point3::new(2., 2., 2.);
    [
        BoundingBox3::new(minimum, maximum),
        BoundingBox3::new(minimum, maximum),
    ]
}

fn assert_native_curve_row(task: &BOPAlgoFaceFaceTask<'_>, far: bool) {
    let curve = task.intersector().lines().unwrap()[0].curve();
    let first = curve
        .get()
        .unwrap()
        .eval_d0(curve.first_parameter())
        .unwrap();
    let last = curve
        .get()
        .unwrap()
        .eval_d0(curve.last_parameter())
        .unwrap();
    let expected = native_values(far);
    assert_task_status(task, far, &expected);
    assert_curve_range(curve.get().unwrap(), &expected);
    assert_point_close(first, [expected[5], expected[6], expected[7]]);
    assert_point_close(last, [expected[8], expected[9], expected[10]]);
}

fn assert_task_status(task: &BOPAlgoFaceFaceTask<'_>, far: bool, expected: &[f64]) {
    assert_eq!(expected[0], f64::from(far));
    assert_eq!(expected[1], 1.0);
    assert_eq!(
        expected[2],
        task.intersector().lines().unwrap().len() as f64
    );
    let translation = task.trsf().translation_part().x();
    assert_eq!(translation == 0.0, !far);
}

fn assert_curve_range(curve: &dyn rustycad_tkg3d::geom::GeomCurve, expected: &[f64]) {
    assert_close(curve.first_parameter(), expected[3], 1e-12);
    assert_close(curve.last_parameter(), expected[4], 1e-12);
}

fn native_values(far: bool) -> Vec<f64> {
    include_str!("../../../../../tools/oracle/tkbo_face_face_task.expected")
        .lines()
        .nth(usize::from(!far))
        .unwrap()
        .split_whitespace()
        .map(|value| value.parse().unwrap())
        .collect()
}

fn assert_point_close(point: rustycad_tkmath::gp::Point3, expected: [f64; 3]) {
    for (actual, wanted) in point.coordinates().iter().zip(expected) {
        assert_close(*actual, wanted, 1e-7);
    }
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected}"
    );
}

#[test]
fn void_boxes_follow_native_failure_alert_path() {
    let mut task = BOPAlgoFaceFaceTask::default();
    task.perform_parallel().unwrap();
    assert!(task.parallel_options().has_errors());
}
