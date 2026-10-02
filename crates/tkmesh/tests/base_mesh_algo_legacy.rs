//! Direct Rust mirrors of OCCT's internal-face-vertex meshing regressions.
#![allow(non_snake_case)]
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Face, Shape},
};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::{
    gp::{Plane3, Point3},
    poly::mesh_purpose,
};
use rustycad_tkmesh::{brep_mesh::BRepMeshIncrementalMesh, imesh_tools::MeshParameters};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeVertex};

#[test]
fn BRepMesh_BaseMeshAlgoTest__InternalVerticesAreBound() {
    let points = [
        Point3::new(40., 20., 0.),
        Point3::new(60., 30., 0.),
        Point3::new(77., 33., 0.),
    ];
    let (shape, face) = planar_face_with_internal_vertices(&points).unwrap();
    mesh_with_internal_vertices(&shape, true).unwrap();
    let triangulation = BRepTool::triangulation(&face, mesh_purpose::NONE)
        .unwrap()
        .0;
    assert!(!triangulation.is_null());
    assert_valid_mesh(&triangulation);
    for point in points {
        assert!(closest_node_distance(&triangulation, point) < 1e-6);
    }
}

#[test]
fn BRepMesh_BaseMeshAlgoTest__MultipleInternalVertices() {
    let points = (1..5)
        .flat_map(|x| (1..5).map(move |y| Point3::new((20 * x) as f64, (20 * y) as f64, 0.)))
        .collect::<Vec<_>>();
    let (shape, face) = planar_face_with_bounds_and_vertices(100., 100., &points).unwrap();
    mesh_with_internal_vertices(&shape, true).unwrap();
    let triangulation = BRepTool::triangulation(&face, mesh_purpose::NONE)
        .unwrap()
        .0;
    assert!(!triangulation.is_null());
    assert_valid_mesh(&triangulation);
    assert_eq!(
        points
            .iter()
            .filter(|point| closest_node_distance(&triangulation, **point) < 1e-6)
            .count(),
        points.len()
    );
}

#[test]
fn BRepMesh_BaseMeshAlgoTest__InternalVerticesModeDisabled() {
    let points = [Point3::new(50., 25., 0.)];
    let (shape, face) = planar_face_with_bounds_and_vertices(100., 50., &points).unwrap();
    mesh_with_internal_vertices(&shape, false).unwrap();
    let triangulation = BRepTool::triangulation(&face, mesh_purpose::NONE)
        .unwrap()
        .0;
    assert!(!triangulation.is_null());
    assert_valid_mesh(&triangulation);
}

fn planar_face_with_internal_vertices(
    points: &[Point3],
) -> Result<(Shape, Face), rustycad_tkernel::standard::Failure> {
    planar_face_with_bounds_and_vertices(100., 50., points)
}

fn planar_face_with_bounds_and_vertices(
    u_max: f64,
    v_max: f64,
    points: &[Point3],
) -> Result<(Shape, Face), rustycad_tkernel::standard::Failure> {
    let maker =
        BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (0., u_max, 0., v_max))?;
    let face = maker.face()?;
    let mut shape = face.clone().into_shape();
    for point in points {
        add_internal_vertex(&mut shape, &face, *point)?;
    }
    Ok((shape, face))
}

fn add_internal_vertex(
    shape: &mut Shape,
    face: &Face,
    point: Point3,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let vertex = BRepBuilderAPIMakeVertex::new(point)?.vertex()?;
    BRepBuilder::default().update_vertex_on_face(&vertex, point.x(), point.y(), face, 1e-7)?;
    let component = vertex.into_shape().oriented(Orientation::Internal);
    BRepBuilder::default().add(shape, &component)
}

fn mesh_with_internal_vertices(
    shape: &Shape,
    enabled: bool,
) -> Result<BRepMeshIncrementalMesh, rustycad_tkernel::standard::Failure> {
    BRepMeshIncrementalMesh::with_parameters(
        shape,
        &MeshParameters {
            deflection: 1.,
            angle: 0.5,
            internal_vertices_mode: enabled,
            ..Default::default()
        },
        &ProgressRange::default(),
    )
}

fn assert_valid_mesh(triangulation: &rustycad_tkmath::poly::Triangulation) {
    for index in 1..=triangulation.nb_nodes() {
        let point = triangulation.node(index).unwrap();
        assert!(point.x().abs() < 1e10 && point.y().abs() < 1e10 && point.z().abs() < 1e10);
    }
    for index in 1..=triangulation.nb_triangles() {
        let (a, b, c) = triangulation.triangle(index).unwrap().get();
        assert!(a != b && b != c && a != c);
    }
}

fn closest_node_distance(
    triangulation: &rustycad_tkmath::poly::Triangulation,
    point: Point3,
) -> f64 {
    (1..=triangulation.nb_nodes())
        .map(|index| triangulation.node(index).unwrap().distance(&point))
        .fold(f64::INFINITY, f64::min)
}
