use super::*;
use rustycad_tkbo::bop_tools;
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkg3d::geom::GeomLine;
use rustycad_tkmath::gp::{Axis1, Point3};
fn micro_fixture(kind: usize, tolerance: f64) -> Edge {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    if kind == 0 {
        return edge;
    }
    let length = [0., 1e-8, 1e-6, 1e-4, 1., 1.][kind];
    let curve = Handle::new(GeomLine::new(Axis1::new(
        Point3::default(),
        Direction3::new(1., 0., 0.).unwrap(),
    )))
    .cast();
    builder
        .make_edge_with_curve(&mut edge, &curve, tolerance)
        .unwrap();
    builder.range(&edge, 0., length, false).unwrap();
    let first = micro_vertex(&edge, 0., tolerance, Orientation::Forward);
    let last = micro_vertex(&edge, length, tolerance, Orientation::Reversed);
    let mut shape = edge.clone().into_shape();
    builder.add(&mut shape, &first).unwrap();
    builder.add(&mut shape, &last).unwrap();
    if kind == 5 {
        builder.degenerated(&edge, true).unwrap();
    }
    edge
}
fn micro_vertex(edge: &Edge, parameter: f64, tolerance: f64, orientation: Orientation) -> Vertex {
    let builder = BRepBuilder::default();
    let mut vertex = Vertex::default();
    builder
        .make_vertex_at(&mut vertex, Point3::new(parameter, 0., 0.), tolerance)
        .unwrap();
    builder
        .update_vertex_on_edge(&vertex, parameter, edge, tolerance)
        .unwrap();
    vertex.set_orientation(orientation);
    vertex
}
#[test]
fn native_micro_edge_tolerance_and_orientation_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_micro_edge.expected").lines() {
        let row: Vec<usize> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let mut edge = micro_fixture(row[0], if row[1] == 0 { 1e-7 } else { 1e-5 });
        edge.set_orientation(
            [
                Orientation::Forward,
                Orientation::Reversed,
                Orientation::Internal,
                Orientation::External,
            ][row[4]],
        );
        let context = if row[3] == 0 {
            Handle::default()
        } else {
            Handle::new(Context::default())
        };
        assert_eq!(
            bop_tools::is_micro_edge(&edge, &context, row[2] != 0).unwrap(),
            row[5] != 0,
            "{line}"
        );
    }
}
