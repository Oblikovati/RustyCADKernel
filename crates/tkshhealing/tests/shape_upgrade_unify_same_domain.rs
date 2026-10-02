//! Native parity for focused `ShapeUpgrade_UnifySameDomain::Build` paths.
mod common;
use common::{face_count, rectangle_wire};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Shape, Shell, Wire};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeUnifySameDomain;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape,
    BRepBuilderAPIMakeWire, BRepBuilderAPISewing,
};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;

#[test]
fn basic_box_unification_matches_native_no_merge_result() {
    let mut box_maker =
        BRepPrimAPIMakeBox::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 10.0, 10.0))
            .unwrap();
    box_maker.build(&ProgressRange::default()).unwrap();
    assert!(box_maker.is_done());
    let input = box_maker.shape().unwrap().clone();
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, true, false);

    unifier.build().unwrap();

    let result = unifier.shape();
    assert!(!result.is_null());
    assert_eq!(face_count(&result), 6);
    assert!(
        BRepCheckAnalyzer::new(&result, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
}

#[test]
fn internal_open_path_terminates_and_keeps_its_face() {
    let input = shell_with_internal_path(
        &[
            Point3::new(2.0, 2.0, 0.0),
            Point3::new(5.0, 5.0, 0.0),
            Point3::new(8.0, 2.0, 0.0),
        ],
        false,
    );
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, true, false);
    unifier.set_angular_tolerance(1e-6);
    unifier.set_linear_tolerance(1e-5);
    unifier.allow_internal_edges(true);

    unifier.build().unwrap();

    let result = unifier.shape();
    assert!(!result.is_null());
    assert!(face_count(&result) >= 1);
}

#[test]
fn closed_internal_wire_keeps_nonnull_result() {
    let input = shell_with_internal_path(
        &[
            Point3::new(5.0, 5.0, 0.0),
            Point3::new(15.0, 5.0, 0.0),
            Point3::new(10.0, 15.0, 0.0),
        ],
        true,
    );
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, true, false);
    unifier.set_angular_tolerance(1e-6);
    unifier.set_linear_tolerance(1e-5);
    unifier.allow_internal_edges(true);

    unifier.build().unwrap();

    assert!(!unifier.shape().is_null());
}

fn shell_with_internal_path(points: &[Point3], closed: bool) -> Shape {
    let extent = if closed { 20.0 } else { 10.0 };
    let outer = rectangle_wire(0.0, extent, 0.0, extent);
    let mut face = BRepBuilderAPIMakeFace::from_plane_wire(Plane3::default(), &outer, true)
        .unwrap()
        .face()
        .unwrap();
    let mut internal = BRepBuilderAPIMakeWire::new();
    let edge_count = if closed {
        points.len()
    } else {
        points.len() - 1
    };
    for index in 0..edge_count {
        let edge =
            BRepBuilderAPIMakeEdge::from_points(points[index], points[(index + 1) % points.len()])
                .unwrap()
                .edge()
                .unwrap();
        internal.add_edge(&edge).unwrap();
    }
    let mut internal = internal.wire().unwrap();
    internal.set_orientation(Orientation::Internal);
    BRepBuilder::default().add(&mut face, &internal).unwrap();

    let mut shell = Shell::default();
    BRepBuilder::default().make_shell(&mut shell);
    BRepBuilder::default()
        .add(&mut shell, &face.into_shape())
        .unwrap();
    shell.into_shape()
}

#[test]
fn multiple_coplanar_faces_with_internal_wires_merge_to_one_face() {
    let first = face_with_internal_path(
        0.0,
        5.0,
        0.0,
        10.0,
        &[Point3::new(1.0, 5.0, 0.0), Point3::new(4.0, 5.0, 0.0)],
    );
    let second = face_with_internal_path(
        5.0,
        10.0,
        0.0,
        10.0,
        &[Point3::new(6.0, 5.0, 0.0), Point3::new(9.0, 5.0, 0.0)],
    );
    let input = sew_coplanar_faces(first, second);
    let result = unify_faces_with_internal_edges(&input);

    assert!(!result.is_null());
    assert_eq!(face_count(&result), 1);
    assert_eq!(internal_wire_count(&result), 2);
    assert_eq!(internal_edge_count(&result), 2);
}

#[test]
fn shared_vertex_internal_path_rebuild_terminates_in_face_merge() {
    let input = sewn_internal_path_pair();
    let result = unify_faces_with_internal_edges(&input);

    assert_internal_path_merge(&result);
}

fn unify_faces_with_internal_edges(input: &Shape) -> Shape {
    let unifier = ShapeUpgradeUnifySameDomain::new(input, true, true, false);
    unifier.set_angular_tolerance(1e-6);
    unifier.set_linear_tolerance(1e-5);
    unifier.allow_internal_edges(true);
    unifier.build().unwrap();
    unifier.shape()
}

fn sewn_internal_path_pair() -> Shape {
    let first = face_with_internal_path(
        0.0,
        10.0,
        0.0,
        10.0,
        &[
            Point3::new(2.0, 2.0, 0.0),
            Point3::new(5.0, 5.0, 0.0),
            Point3::new(8.0, 2.0, 0.0),
        ],
    );
    let second = face_with_internal_path(0.0, 10.0, 10.0, 20.0, &[]);
    sew_coplanar_faces(first, second)
}

fn sew_coplanar_faces(first: Shape, second: Shape) -> Shape {
    let mut sewing = BRepBuilderAPISewing::default();
    sewing.add(&first).unwrap();
    sewing.add(&second).unwrap();
    sewing.perform(&ProgressRange::default()).unwrap();
    let input = sewing.sewed_shape();
    assert_eq!(face_count(input), 2);
    input.clone()
}

fn assert_internal_path_merge(result: &Shape) {
    assert_eq!(face_count(result), 1);
    assert_eq!(internal_wire_count(result), 1);
    assert_eq!(internal_edge_count(result), 2);
}

fn face_with_internal_path(
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    points: &[Point3],
) -> Shape {
    let outer = rectangle_wire(x_min, x_max, y_min, y_max);
    let plane = Plane3::default();
    let mut face = BRepBuilderAPIMakeFace::from_plane_wire(plane, &outer, true)
        .unwrap()
        .face()
        .unwrap();
    if points.is_empty() {
        return face.into_shape();
    }
    let mut inner = internal_path_wire(points);
    inner.set_orientation(Orientation::Internal);
    BRepBuilder::default().add(&mut face, &inner).unwrap();
    face.into_shape()
}

fn internal_path_wire(points: &[Point3]) -> Wire {
    let mut inner_builder = BRepBuilderAPIMakeWire::new();
    for pair in points.windows(2) {
        let edge = BRepBuilderAPIMakeEdge::from_points(pair[0], pair[1])
            .unwrap()
            .edge()
            .unwrap();
        inner_builder.add_edge(&edge).unwrap();
    }
    inner_builder.wire().unwrap()
}

fn internal_wire_count(shape: &Shape) -> usize {
    rustycad_tkbrep::topexp::Explorer::new(shape, ShapeEnum::Wire, ShapeEnum::Shape)
        .unwrap()
        .filter(|wire| {
            wire.orientation() == Orientation::Internal
                || rustycad_tkbrep::topexp::Explorer::new(wire, ShapeEnum::Edge, ShapeEnum::Shape)
                    .is_ok_and(|edges| {
                        edges
                            .into_iter()
                            .any(|edge| edge.orientation() == Orientation::Internal)
                    })
        })
        .count()
}

fn internal_edge_count(shape: &Shape) -> usize {
    rustycad_tkbrep::topexp::Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .filter(|edge| edge.orientation() == Orientation::Internal)
        .count()
}
