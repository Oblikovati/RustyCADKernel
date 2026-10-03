//! Focused edge-curve behavior tests for the internal simple-offset edge step.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Axis3, Direction3, Plane3, Point3, Transform3, Vector3};
use rustycad_tkmath::toploc::Location;
use rustycad_tkoffset::brep_offset::{OffsetFaceSurface, build_simple_offset_edge};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape};

#[test]
fn matches_native_simple_offset_curve_location_and_tolerance() {
    let face = bounded_plane_face();
    for edge in face_edges(&face) {
        assert_native_edge_result(&edge, &[offset_face(&face, 10.0)], 10.0, 1.0e-7);
    }
}

#[test]
fn uses_curve_on_face_for_located_and_reversed_faces() {
    let mut located = bounded_plane_face();
    located
        .set_location(
            &Location::new(Transform3::translation_vector(Vector3::new(0.0, 0.0, 3.0))),
            false,
        )
        .unwrap();
    for edge in face_edges(&located) {
        assert_native_edge_result(&edge, &[offset_face(&located, 13.0)], 13.0, 1.0e-7);
    }

    let mut reversed = bounded_plane_face();
    reversed.set_orientation(Orientation::Reversed);
    for edge in face_edges(&reversed) {
        assert_native_edge_result(&edge, &[offset_face(&reversed, -10.0)], -10.0, 1.0e-7);
    }
}

#[test]
fn measures_max_deviation_across_adjacent_offset_faces() {
    let face = bounded_plane_face();
    let mut reversed = face.clone();
    reversed.reverse();
    let adjacent = [offset_face(&face, 10.0), offset_face(&reversed, -10.0)];
    let edge = face_edges(&face).remove(0);
    assert_native_edge_result(&edge, &adjacent, 10.0, 20.0002);
}

#[test]
fn skips_edges_without_ancestor_face_offsets() {
    let edge = Edge::default();
    assert!(build_simple_offset_edge(&edge, &[]).unwrap().is_none());
}

fn bounded_plane_face() -> Face {
    let source: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let mut maker =
        BRepBuilderAPIMakeFace::from_surface_bounds(&source, (-1.0, 1.0, -1.0, 1.0), 0.0).unwrap();
    Face::unchecked(maker.shape().unwrap().clone())
}

fn offset_face(face: &Face, z_offset: f64) -> OffsetFaceSurface {
    OffsetFaceSurface {
        face: face.clone(),
        surface: plane_at(z_offset),
    }
}

fn plane_at(z: f64) -> Handle<dyn GeomSurface> {
    let frame = Plane3::new(
        Point3::new(0.0, 0.0, z),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap()
    .position();
    Handle::new(GeomPlane::new(frame)).cast()
}

fn face_edges(face: &Face) -> Vec<Edge> {
    let shape: Shape = face.clone().into();
    let mut explorer = Explorer::new(&shape, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    let mut edges: Vec<Edge> = Vec::new();
    while explorer.more() {
        edges.push(Edge::unchecked(explorer.current().clone()));
        explorer.next().unwrap();
    }
    edges
}

fn assert_native_edge_result(
    edge: &Edge,
    adjacent: &[OffsetFaceSurface],
    expected_z: f64,
    expected_tolerance: f64,
) {
    let result = build_simple_offset_edge(edge, adjacent).unwrap().unwrap();
    let curve: &dyn GeomCurve = result.curve.get().unwrap();
    assert!(result.location.is_identity());
    assert!((result.tolerance - expected_tolerance).abs() < 1.0e-9);
    let (first, last) = BRepTool::range(edge).unwrap();
    let start = curve.value(first).unwrap();
    let end = curve.value(last).unwrap();
    assert!((start.z() - expected_z).abs() < 1.0e-10);
    assert!((end.z() - expected_z).abs() < 1.0e-10);
    assert!((start.square_distance(&end) - 4.0).abs() < 1.0e-10);
}
