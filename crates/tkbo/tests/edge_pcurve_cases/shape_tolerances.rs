use super::*;
use rustycad_tkbo::bop_tools;
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp,
    topods::{Face, Vertex},
    toptools::IndexedMapOfShape,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
#[test]
fn native_shape_tolerance_pass_order_and_avoidance_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_shape_tolerances.expected").lines() {
        let row: Vec<f64> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let actual = observe(&row);
        assert_eq!(actual.len(), row.len() - 4, "{line}");
        for (actual, expected) in actual.iter().zip(&row[4..]) {
            assert!((actual - expected).abs() < 1e-14, "{line}: {actual}");
        }
    }
}
fn observe(row: &[f64]) -> Vec<f64> {
    let (face, edges, vertices) = tolerance_fixture(row[0] as usize);
    let avoided = avoidance(&edges, &vertices, row[1] as usize);
    for _ in 0..row[2] as usize {
        bop_tools::correct_shape_tolerances(&face, &avoided, row[3] != 0.).unwrap();
    }
    let mut actual = vec![edges.extent() as f64, vertices.extent() as f64];
    for index in 1..=edges.extent() {
        actual.push(
            BRepTool::edge_tolerance(&Edge::unchecked(edges.find_key(index).unwrap().clone()))
                .unwrap(),
        );
    }
    for index in 1..=vertices.extent() {
        actual.push(
            BRepTool::vertex_tolerance(&Vertex::unchecked(
                vertices.find_key(index).unwrap().clone(),
            ))
            .unwrap(),
        );
    }
    actual
}
fn tolerance_fixture(mode: usize) -> (Face, IndexedMapOfShape, IndexedMapOfShape) {
    let mut face = fixture(0).face;
    let builder = BRepBuilder::default();
    if mode == 2 {
        face = Face::unchecked(face.empty_copied().unwrap());
    }
    builder.update_face_tolerance(&face, 0.01).unwrap();
    let (mut edges, mut vertices) = (IndexedMapOfShape::default(), IndexedMapOfShape::default());
    topexp::map_shapes_of_type(&face, ShapeEnum::Edge, &mut edges).unwrap();
    topexp::map_shapes_of_type(&face, ShapeEnum::Vertex, &mut vertices).unwrap();
    if !edges.is_empty() {
        set_edge_tolerances(&edges);
    }
    if mode != 0 {
        let vertex = internal_vertex();
        builder.add(&mut face, &vertex).unwrap();
        vertices.add(vertex.into_shape());
    }
    (face, edges, vertices)
}
fn set_edge_tolerances(edges: &IndexedMapOfShape) {
    let builder = BRepBuilder::default();
    for (index, tolerance) in [(1, 0.001), (edges.extent(), 0.02)] {
        builder
            .update_edge_tolerance(
                &Edge::unchecked(edges.find_key(index).unwrap().clone()),
                tolerance,
            )
            .unwrap();
    }
}
fn internal_vertex() -> Vertex {
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, Point3::default(), 0.005)
        .unwrap();
    vertex.set_orientation(Orientation::Internal);
    vertex
}
fn avoidance(
    edges: &IndexedMapOfShape,
    vertices: &IndexedMapOfShape,
    mode: usize,
) -> IndexedMapOfShape {
    let mut avoided = IndexedMapOfShape::default();
    if mode == 1 && !vertices.is_empty() {
        avoided.add(vertices.find_key(1).unwrap().clone());
    }
    if mode == 2 && !edges.is_empty() {
        avoided.add(edges.find_key(1).unwrap().clone());
    }
    if mode == 3 {
        for shapes in [edges, vertices] {
            for index in 1..=shapes.extent() {
                avoided.add(shapes.find_key(index).unwrap().clone());
            }
        }
    }
    avoided
}
