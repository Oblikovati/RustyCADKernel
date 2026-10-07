//! Faces with internal edge paths, sewn and unified, shared by the UnifySameDomain targets.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Shape, Wire};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeUnifySameDomain;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeWire, BRepBuilderAPISewing,
};

use super::common::{face_count, rectangle_wire};

pub fn unify_faces_with_internal_edges(input: &Shape) -> Shape {
    let unifier = ShapeUpgradeUnifySameDomain::new(input, true, true, false);
    unifier.set_angular_tolerance(1e-6);
    unifier.set_linear_tolerance(1e-5);
    unifier.allow_internal_edges(true);
    unifier.build().unwrap();
    unifier.shape()
}

pub fn sew_coplanar_faces(first: Shape, second: Shape) -> Shape {
    let mut sewing = BRepBuilderAPISewing::default();
    sewing.add(&first).unwrap();
    sewing.add(&second).unwrap();
    sewing.perform(&ProgressRange::default()).unwrap();
    let input = sewing.sewed_shape();
    assert_eq!(face_count(input), 2);
    input.clone()
}

pub fn face_with_internal_path(
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

pub fn internal_path_wire(points: &[Point3]) -> Wire {
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

pub fn internal_wire_count(shape: &Shape) -> usize {
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

pub fn internal_edge_count(shape: &Shape) -> usize {
    rustycad_tkbrep::topexp::Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .filter(|edge| edge.orientation() == Orientation::Internal)
        .count()
}
