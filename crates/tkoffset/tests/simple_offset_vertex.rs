//! Native differential tests for simple-offset vertex aggregation and ancestor maps.
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Builder, Compound, Edge, Face, Shape};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Axis3, Direction3, Plane3, Point3, Transform3, Vector3};
use rustycad_tkmath::toploc::Location;
use rustycad_tkoffset::brep_offset::{
    OffsetEdgeGeometry, OffsetFaceSurface, build_simple_offset_edge, build_simple_offset_vertices,
};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape};

#[test]
fn matches_native_face_vertex_points_and_tolerances() {
    let face = bounded_plane_face();
    assert_vertex_results(&face, &[offset_face(&face, 10.0)], 10.0, 1.001e-7);
}

#[test]
fn matches_native_located_and_reversed_face_vertex_points() {
    let mut located = bounded_plane_face();
    located
        .set_location(
            &Location::new(Transform3::translation_vector(Vector3::new(0.0, 0.0, 3.0))),
            false,
        )
        .unwrap();
    assert_vertex_results(&located, &[offset_face(&located, 13.0)], 13.0, 1.001e-7);

    let mut reversed = bounded_plane_face();
    reversed.set_orientation(Orientation::Reversed);
    assert_vertex_results(&reversed, &[offset_face(&reversed, -10.0)], -10.0, 1.001e-7);
}

#[test]
fn includes_adjacent_edge_tolerance_in_vertex_result() {
    let face = bounded_plane_face();
    let mut reversed = face.clone();
    reversed.reverse();
    let mut compound = Compound::default();
    let builder = Builder;
    builder.make_compound(&mut compound);
    builder
        .add(&mut compound, &Shape::from(face.clone()))
        .unwrap();
    builder
        .add(&mut compound, &Shape::from(reversed.clone()))
        .unwrap();
    let adjacent = [offset_face(&face, 10.0), offset_face(&reversed, -10.0)];
    let offsets = compute_edge_offsets(&compound, &adjacent);
    let results = build_simple_offset_vertices(&compound, &offsets).unwrap();
    assert!(!results.is_empty());
    assert!(results.iter().all(|result| {
        (result.geometry.point.z() - 10.0).abs() < 1.0e-10
            && (result.geometry.tolerance - 20.0202).abs() < 1.0e-6
    }));
}

fn assert_vertex_results(
    face: &Face,
    adjacent: &[OffsetFaceSurface],
    expected_z: f64,
    expected_tolerance: f64,
) {
    let shape: Shape = face.clone().into();
    let offsets = compute_edge_offsets(&shape, adjacent);
    let results = build_simple_offset_vertices(&shape, &offsets).unwrap();
    assert_eq!(results.len(), 4);
    assert!(results.iter().all(|result| {
        (result.geometry.point.z() - expected_z).abs() < 1.0e-10
            && (result.geometry.tolerance - expected_tolerance).abs() < 1.0e-12
    }));
}

fn compute_edge_offsets(
    shape: &Shape,
    adjacent: &[OffsetFaceSurface],
) -> Vec<(Edge, Option<OffsetEdgeGeometry>)> {
    let mut explorer = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    let mut offsets: Vec<(Edge, Option<OffsetEdgeGeometry>)> = Vec::new();
    while explorer.more() {
        let edge = Edge::unchecked(explorer.current().clone());
        let geometry = build_simple_offset_edge(&edge, adjacent).unwrap();
        offsets.push((edge, geometry));
        explorer.next().unwrap();
    }
    offsets
}

fn bounded_plane_face() -> Face {
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let mut maker =
        BRepBuilderAPIMakeFace::from_surface_bounds(&plane, (-1.0, 1.0, -1.0, 1.0), 0.0).unwrap();
    Face::unchecked(maker.shape().unwrap().clone())
}

fn offset_face(face: &Face, z: f64) -> OffsetFaceSurface {
    OffsetFaceSurface {
        face: face.clone(),
        surface: plane_at(z),
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
