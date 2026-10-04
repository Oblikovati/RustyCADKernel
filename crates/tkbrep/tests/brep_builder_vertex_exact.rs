//! Exact `BRep_TVertex` setter semantics used by Boolean same-domain mutation.
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTVertex, BRepTool},
    topods::Vertex,
};
use rustycad_tkmath::gp::Point3;

#[test]
fn direct_vertex_point_and_tolerance_setters_can_lower_tolerance() {
    let builder = BRepBuilder::default();
    let mut vertex = Vertex::default();
    builder
        .make_vertex_at(&mut vertex, Point3::default(), 1.0e-4)
        .unwrap();
    builder
        .set_vertex_point_and_tolerance(&vertex, Point3::new(2.0, 3.0, 4.0), 1.0e-8)
        .unwrap();
    assert_eq!(BRepTool::pnt(&vertex).unwrap(), Point3::new(2.0, 3.0, 4.0));
    assert_eq!(
        vertex
            .tshape()
            .cast::<BRepTVertex>()
            .get()
            .unwrap()
            .tolerance(),
        1.0e-8
    );
    assert_eq!(BRepTool::vertex_tolerance(&vertex).unwrap(), 1.0e-7);
}
