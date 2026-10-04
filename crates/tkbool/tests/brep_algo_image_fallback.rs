//! OCCT's unbound-image accessor appends to a process-wide static list on each call.
use rustycad_tkbool::brep_algo::BRepAlgoImage;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Shape, Vertex},
};
use rustycad_tkmath::gp::Point3;

#[test]
fn unbound_image_calls_accumulate_in_the_shared_fallback_list() {
    let shape = vertex(1.0);
    let graph = BRepAlgoImage::new();

    let first = graph.image(&shape);
    let second = graph.image(&shape);

    assert_eq!(first.len(), 1);
    assert_eq!(second.len(), 2);
    assert!(second.iter().all(|image| image.is_same(&shape)));
}

fn vertex(x: f64) -> Shape {
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, Point3::new(x, 0.0, 0.0), 1.0e-7)
        .unwrap();
    vertex.into_shape()
}
