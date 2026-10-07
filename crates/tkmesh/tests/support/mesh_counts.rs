//! Node and triangle totals of a meshed shape, shared by the incremental-mesh targets.
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{self, Shape},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::poly::mesh_purpose;

/// Sums nodes and triangles over every face triangulation; each face must have one;
/// `let (nodes, triangles) = shape_mesh_counts(&shape);`.
pub fn shape_mesh_counts(shape: &Shape) -> (i32, i32) {
    let mut faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let mut node_count = 0;
    let mut triangle_count = 0;
    while faces.more() {
        let face = topods::face(faces.current()).unwrap();
        let triangulation = BRepTool::triangulation(&face, mesh_purpose::NONE)
            .unwrap()
            .0;
        assert!(!triangulation.is_null());
        node_count += triangulation.nb_nodes();
        triangle_count += triangulation.nb_triangles();
        faces.next().unwrap();
    }
    (node_count, triangle_count)
}
