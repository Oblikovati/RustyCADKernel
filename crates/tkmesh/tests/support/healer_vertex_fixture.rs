//! Distinct topological vertices exercise the healer's geometric tolerance fallback.
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topexp,
    topods::{Edge, Vertex},
};
use rustycad_tkmath::gp::Point3;
pub fn separate_vertices(edge: &mut Edge, index: i32, mode: i32) {
    let (first, last) = topexp::vertices(edge, false);
    edge.set_free(true).unwrap();
    replace_vertex(edge, &first, index, mode, true);
    replace_vertex(edge, &last, index, mode, false);
}
fn replace_vertex(edge: &mut Edge, vertex: &Vertex, index: i32, mode: i32, first: bool) {
    let builder = BRepBuilder::default();
    builder.remove(edge, vertex).unwrap();
    if mode == 11 || (mode == 10 && first) {
        return;
    }
    let replacement = displaced_vertex(vertex, index, mode);
    builder.add(edge, &replacement).unwrap();
}
fn displaced_vertex(vertex: &Vertex, index: i32, mode: i32) -> Vertex {
    let builder = BRepBuilder::default();
    let point = BRepTool::pnt(vertex).unwrap();
    let offset = if index % 2 == 0 || mode >= 10 {
        0.0
    } else {
        [0.0, 1e-7, 2e-7, 4e-7][(mode - 6) as usize]
    };
    let mut replacement = Vertex::default();
    builder
        .make_vertex_at(
            &mut replacement,
            Point3::new(point.x() + offset, point.y(), point.z()),
            BRepTool::vertex_tolerance(vertex).unwrap(),
        )
        .unwrap();
    replacement.set_orientation(vertex.orientation());
    replacement
}
