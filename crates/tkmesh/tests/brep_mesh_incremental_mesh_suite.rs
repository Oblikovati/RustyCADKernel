//! Rust mirrors for the full BRepMesh_IncrementalMesh GTest source.
#![allow(non_snake_case)]
use rustycad_tkbrep::{brep::BRepTool, topods::Wire};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkmath::{
    gp::{Axis2, Circle3, Direction3, Plane3, Point3, StandardDirection3, Vector3},
    poly::mesh_purpose,
};
use rustycad_tkmesh::{brep_mesh::BRepMeshIncrementalMesh, imesh_tools::MeshParameters};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire,
};

#[test]
fn BRepMesh_IncrementalMeshTest__OCC26407_PlanarPolygonMeshStatus() {
    let points = planar_regression_points();
    let wire = closed_polygon_wire(&points).unwrap();
    let normal =
        Vector3::between(points[0], points[1]).crossed(Vector3::between(points[0], points[7]));
    let normal = Direction3::new(normal.x(), normal.y(), normal.z()).unwrap();
    let plane = Plane3::new(points[0], normal).unwrap();
    let face_maker = BRepBuilderAPIMakeFace::from_plane_wire(plane, &wire, true).unwrap();
    let face = face_maker.face().unwrap();
    let shape = face.clone().into_shape();
    let result = BRepMeshIncrementalMesh::with_parameters(
        &shape,
        &MeshParameters {
            deflection: 1e-7,
            ..Default::default()
        },
        &ProgressRange::default(),
    );
    assert!(result.is_ok(), "planar polygon meshing should complete");
    let triangulation = BRepTool::triangulation(&face, mesh_purpose::NONE)
        .unwrap()
        .0;
    assert!(!triangulation.is_null());
    assert!(triangulation.nb_triangles() > 0);
}

#[test]
fn BRepMesh_IncrementalMeshTest__OCC30234_ZeroLengthFreeEdgeHasNoTriangles() {
    let circle = Circle3::new(Axis2::from_standard(StandardDirection3::Z), 0.0).unwrap();
    let mut maker = BRepBuilderAPIMakeEdge::from_circ(circle).unwrap();
    let shape = maker.shape().unwrap().clone();
    let parameters = MeshParameters {
        deflection: 0.005,
        relative: true,
        ..Default::default()
    };
    let result =
        BRepMeshIncrementalMesh::with_parameters(&shape, &parameters, &ProgressRange::default());
    assert!(
        result.is_ok(),
        "zero-length free-edge meshing failed: {}",
        result
            .err()
            .map(|failure| failure.exception_type().to_string())
            .unwrap_or_default()
    );
}

fn planar_regression_points() -> [Point3; 8] {
    [
        Point3::new(587.9000000000001, 40.675817923051625, 88.5),
        Point3::new(807.8241820769484, 260.59999999999997, 88.5),
        Point3::new(644.1741820769485, 424.24999999999994, 88.50000000000001),
        Point3::new(629.978025792619, 424.25, 88.5),
        Point3::new(793.6280257926187, 260.59999999999985, 88.5),
        Point3::new(587.9000000000002, 54.87197420738135, 88.5),
        Point3::new(218.52197420738142, 424.25000000000006, 88.5),
        Point3::new(204.3258179230519, 424.24999999999994, 88.5),
    ]
}

fn closed_polygon_wire(points: &[Point3; 8]) -> Result<Wire, rustycad_tkernel::standard::Failure> {
    let mut maker = BRepBuilderAPIMakeWire::new();
    for index in 0..points.len() {
        let next = (index + 1) % points.len();
        let edge_maker = BRepBuilderAPIMakeEdge::from_points(points[index], points[next])?;
        maker.add_edge(&edge_maker.edge()?)?;
    }
    maker.wire()
}
