//! Native wire orientation order, identity, and closed/branched component cases.
use rustycad_tkbo::bop_tools::orient_edges_on_wire;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Shape, ShapeIterator, Vertex, Wire},
};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point3;
#[test]
fn native_wire_orientation_graph_cases_match() {
    for line in include_str!("../../../tools/oracle/tkbo_orient_wire.expected").lines() {
        let row: Vec<usize> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let (mut wire, edges) = wire_fixture(row[0], row[1]);
        wire.set_orientation(
            [
                Orientation::Forward,
                Orientation::Reversed,
                Orientation::Internal,
                Orientation::External,
            ][row[2]],
        );
        let original = wire.clone();
        orient_edges_on_wire(&mut wire).unwrap();
        let actual = observe(&wire, &original, &edges);
        assert_eq!(actual, row[3..], "{line}");
    }
}
fn observe(wire: &Shape, original: &Shape, edges: &[Edge]) -> Vec<usize> {
    let children: Vec<Shape> = ShapeIterator::new(wire, false, true).collect();
    let mut actual = vec![
        usize::from(wire.is_same(original)),
        wire.orientation() as usize,
        children.len(),
    ];
    for child in children {
        let index = edges.iter().position(|edge| child.is_same(edge)).unwrap();
        actual.extend([index, child.orientation() as usize]);
    }
    actual
}
fn wire_fixture(mode: usize, mask: usize) -> (Shape, Vec<Edge>) {
    let mut wire = Wire::default();
    let builder = BRepBuilder::default();
    builder.make_wire(&mut wire);
    let vertices: Vec<Vertex> = (0..4).map(|index| make_vertex(index as f64)).collect();
    let count = match mode {
        5 => 0,
        4 => 1,
        _ => 3,
    };
    let mut edges = Vec::new();
    for index in 0..count {
        let edge = make_edge(mode, index, mask, &vertices);
        builder.add(&mut wire, &edge).unwrap();
        edges.push(edge);
    }
    (wire.into_shape(), edges)
}
fn make_vertex(coordinate: f64) -> Vertex {
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, Point3::new(coordinate, 0., 0.), 1e-7)
        .unwrap();
    vertex
}
fn make_edge(mode: usize, index: usize, mask: usize, vertices: &[Vertex]) -> Edge {
    let mut edge = Edge::default();
    let builder = BRepBuilder::default();
    builder.make_edge(&mut edge).unwrap();
    if mode != 4 {
        append_vertices(&mut edge, mode, index, vertices);
    }
    edge.set_orientation(if mask & (1 << index) != 0 {
        Orientation::Reversed
    } else {
        Orientation::Forward
    });
    edge
}
fn append_vertices(edge: &mut Edge, mode: usize, index: usize, vertices: &[Vertex]) {
    let links = [
        [[0, 1], [1, 2], [2, 3]],
        [[0, 1], [1, 2], [2, 0]],
        [[0, 1], [1, 2], [1, 3]],
        [[0, 0], [1, 2], [2, 3]],
    ];
    let mut first = vertices[links[mode][index][0]].clone();
    first.set_orientation(Orientation::Forward);
    let mut last = vertices[links[mode][index][1]].clone();
    last.set_orientation(Orientation::Reversed);
    let builder = BRepBuilder::default();
    builder.add(edge, &first).unwrap();
    builder.add(edge, &last).unwrap();
}
