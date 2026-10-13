use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Direction3, Point2, Point3};
use rustycad_tkmath::poly::{
    Polygon3D, PolygonOnTriangulation, Triangle, Triangulation, mesh_purpose,
};
use rustycad_tkmath::toploc::Location;

pub(super) fn attach_meshes(edge: &Edge, face: &Face) {
    let mesh = Handle::new(Triangulation::with_sizes(3, 1, true, true).unwrap());
    fill_triangle(mesh.get().unwrap());
    mesh.get().unwrap().update_cached_min_max();
    BRepBuilder::default()
        .update_face_triangulation(face, &mesh, true)
        .unwrap();
    attach_edge_polygon(edge);
    attach_edge_indices(edge, &mesh);
}

fn fill_triangle(mesh: &Triangulation) {
    let vertices = [
        Point3::default(),
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    for (index, point) in vertices.into_iter().enumerate() {
        fill_mesh_vertex(mesh, index as i32 + 1, point);
    }
    mesh.set_triangle(1, Triangle::new(1, 2, 3)).unwrap();
}

fn fill_mesh_vertex(mesh: &Triangulation, index: i32, point: Point3) {
    mesh.set_node(index, point).unwrap();
    mesh.set_uv_node(index, Point2::new(point.x(), point.y()))
        .unwrap();
    mesh.set_normal(index, Direction3::new(0.0, 0.0, 1.0).unwrap())
        .unwrap();
}

fn attach_edge_polygon(edge: &Edge) {
    let mut points = Array1::with_bounds(1, 2).unwrap();
    points.set_value(1, Point3::default()).unwrap();
    points.set_value(2, Point3::new(2.0, 0.0, 0.0)).unwrap();
    let polygon =
        Handle::new(Polygon3D::from_nodes_and_parameters(&points, &edge_parameters()).unwrap());
    BRepBuilder::default()
        .update_edge_polygon3d(edge, &polygon, &Location::default())
        .unwrap();
}

fn attach_edge_indices(edge: &Edge, mesh: &Handle<Triangulation>) {
    let mut nodes = Array1::with_bounds(1, 2).unwrap();
    nodes.set_value(1, 1).unwrap();
    nodes.set_value(2, 2).unwrap();
    let indices = Handle::new(PolygonOnTriangulation::from_nodes_and_parameters(
        &nodes,
        &edge_parameters(),
    ));
    BRepBuilder::default()
        .update_edge_polygon_on_triangulation(edge, &indices, mesh, &Location::default())
        .unwrap();
}

pub(super) fn retained_mesh(face: &Face) -> Handle<Triangulation> {
    let (mesh, _) = BRepTool::triangulation(face, mesh_purpose::NONE).unwrap();
    assert!(!mesh.is_null());
    mesh
}

pub(super) fn mutate_meshes(edge: &Edge, face: &Face, coordinate: f64) {
    let mesh_handle = retained_mesh(face);
    let mesh = mesh_handle.get().unwrap();
    mesh.set_node(2, Point3::new(coordinate, 0.0, 0.0)).unwrap();
    mesh.set_uv_node(2, Point2::new(coordinate, 1.0)).unwrap();
    mesh.set_normal(2, Direction3::new(1.0, 0.0, 0.0).unwrap())
        .unwrap();
    mesh.set_triangle(1, Triangle::new(3, 2, 1)).unwrap();
    mesh.unset_cached_min_max();
    mesh.set_deflection(coordinate);
    mutate_edge_meshes(edge, &mesh_handle, coordinate);
}

fn mutate_edge_meshes(edge: &Edge, mesh: &Handle<Triangulation>, coordinate: f64) {
    let (polygon, _) = BRepTool::polygon3d(edge).unwrap();
    mutate_polygon(polygon.get().unwrap(), coordinate);
    let indices = BRepTool::polygon_on_triangulation(edge, mesh, &Location::default()).unwrap();
    indices.get().unwrap().set_node(2, 3).unwrap();
    indices.get().unwrap().set_deflection(coordinate);
    indices.get().unwrap().set_parameter(2, coordinate).unwrap();
}

#[derive(Debug, PartialEq)]
pub(super) struct MeshSnapshot {
    triangle: TriangleSnapshot,
    polygon: PolygonSnapshot,
}

#[derive(Debug, PartialEq)]
struct TriangleSnapshot {
    node: Point3,
    uv: Point2,
    normal: [f64; 3],
    triangle: (i32, i32, i32),
    cached_bounds: Option<[f64; 6]>,
    deflection: f64,
}

#[derive(Debug, PartialEq)]
struct PolygonSnapshot {
    node: Point3,
    index: i32,
    deflection: f64,
    indices_deflection: f64,
    polygon_parameter: f64,
    index_parameter: f64,
}

pub(super) fn mesh_snapshot(edge: &Edge, face: &Face) -> MeshSnapshot {
    let mesh = retained_mesh(face);
    let location = BRepTool::triangulation(face, mesh_purpose::NONE).unwrap().1;
    MeshSnapshot {
        triangle: triangle_snapshot(mesh.get().unwrap()),
        polygon: polygon_snapshot(edge, &mesh, &location),
    }
}

fn triangle_snapshot(mesh: &Triangulation) -> TriangleSnapshot {
    let normal = mesh.normal(2).unwrap();
    TriangleSnapshot {
        node: mesh.node(2).unwrap(),
        uv: mesh.uv_node(2).unwrap(),
        normal: [normal.x(), normal.y(), normal.z()],
        triangle: mesh.triangle(1).unwrap().get(),
        cached_bounds: cached_bounds(mesh),
        deflection: mesh.deflection(),
    }
}

fn polygon_snapshot(
    edge: &Edge,
    mesh: &Handle<Triangulation>,
    location: &Location,
) -> PolygonSnapshot {
    let polygon_handle = BRepTool::polygon3d(edge).unwrap().0;
    let polygon = polygon_handle.get().unwrap();
    let indices = BRepTool::polygon_on_triangulation(edge, mesh, location).unwrap();
    PolygonSnapshot {
        node: *polygon.nodes().value(2).unwrap(),
        index: indices.get().unwrap().node(2).unwrap(),
        deflection: polygon.deflection(),
        indices_deflection: indices.get().unwrap().deflection(),
        polygon_parameter: *polygon.parameters().unwrap().value(2).unwrap(),
        index_parameter: indices.get().unwrap().parameter(2).unwrap(),
    }
}

fn cached_bounds(mesh: &Triangulation) -> Option<[f64; 6]> {
    if !mesh.has_cached_min_max() {
        return None;
    }
    Some(mesh.cached_min_max().get().unwrap())
}

pub(super) fn assert_mesh_baseline(edge: &Edge, face: &Face) {
    let actual = mesh_snapshot(edge, face);
    assert_eq!(actual.triangle.node, Point3::new(2.0, 0.0, 0.0));
    assert_eq!(actual.triangle.uv, Point2::new(2.0, 0.0));
    assert_eq!(actual.triangle.normal, [0.0, 0.0, 1.0]);
    assert_eq!(actual.triangle.triangle, (1, 2, 3));
    assert_eq!(
        actual.triangle.cached_bounds,
        Some([0.0, 0.0, 0.0, 2.0, 1.0, 0.0])
    );
    assert_eq!(actual.polygon.node, Point3::new(2.0, 0.0, 0.0));
    assert_eq!(actual.polygon.index, 2);
    assert_eq!(actual.polygon.deflection, 0.0);
}

fn edge_parameters() -> Array1<'static, f64> {
    let mut parameters = Array1::with_bounds(1, 2).unwrap();
    parameters.set_value(1, 0.0).unwrap();
    parameters.set_value(2, 2.0).unwrap();
    parameters
}

fn mutate_polygon(polygon: &Polygon3D, coordinate: f64) {
    polygon.change_nodes(|nodes| {
        nodes
            .set_value(2, Point3::new(coordinate, 0.0, 0.0))
            .unwrap()
    });
    polygon
        .change_parameters(|parameters| parameters.set_value(2, coordinate).unwrap())
        .unwrap();
    polygon.set_deflection(coordinate);
}
