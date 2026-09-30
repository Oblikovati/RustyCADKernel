//! Shared concrete planar connectivity fixtures.
use rustycad_tkernel::standard::Handle;
use rustycad_tkmesh::brep_mesh::{
    BRepMeshDataStructureOfDelaun as Mesh, BRepMeshEdge, BRepMeshTriangle, BRepMeshVertex,
    DegreeOfFreedom as Freedom,
};
pub fn mesh() -> Mesh {
    let mesh = Mesh::new(&Handle::new(Default::default()), 1);
    mesh.vertex_tool().set_cell_size(0.5).unwrap();
    for index in 0..5 {
        mesh.add_node(&vertex(index as f64), true).unwrap();
    }
    mesh
}
pub fn vertex(x: f64) -> BRepMeshVertex {
    BRepMeshVertex::from_uv(x, 0., Freedom::Free)
}
pub fn edge(first: i32, last: i32) -> BRepMeshEdge {
    BRepMeshEdge::new(first, last, Freedom::Free)
}
pub fn triangle(edges: [i32; 3]) -> BRepMeshTriangle {
    BRepMeshTriangle::new(edges, [true; 3], Freedom::Free)
}
pub fn triangulated() -> Mesh {
    let mesh = mesh();
    for (a, b) in [(1, 2), (2, 3), (3, 1), (3, 4), (4, 1)] {
        mesh.add_link(&edge(a, b)).unwrap();
    }
    mesh.add_element(&triangle([1, 2, 3])).unwrap();
    mesh.add_element(&triangle([3, 4, 5])).unwrap();
    mesh
}
/// Add a disconnected-by-edge triangle sharing vertex three; `selection_mesh()`.
pub fn selection_mesh() -> Handle<Mesh> {
    let mesh = Handle::new(triangulated());
    mesh.add_node(&vertex(5.), true).unwrap();
    mesh.add_node(&vertex(6.), true).unwrap();
    for (a, b) in [(3, 6), (6, 7), (7, 3)] {
        mesh.add_link(&edge(a, b)).unwrap();
    }
    mesh.add_element(&triangle([6, 7, 8])).unwrap();
    mesh
}
