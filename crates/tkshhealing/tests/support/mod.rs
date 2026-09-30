//! Shapes built with BRep_Builder for the behavior tests: the vertices and edges of a
//! unit square (no geometry is needed by the list operations under test) and wires of them.
#![allow(dead_code)]
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Edge, Shape, Vertex, Wire};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point3;

/// A vertex at `(x, y, 0)`.
pub fn vertex(x: f64, y: f64) -> Vertex {
    let mut v = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut v, Point3::new(x, y, 0.0), 1e-7)
        .unwrap();
    v
}

/// An edge without geometry from `first` (FORWARD) to `last` (REVERSED).
pub fn edge(first: &Vertex, last: &Vertex) -> Edge {
    let builder = BRepBuilder::default();
    let mut e = Edge::default();
    builder.make_edge(&mut e).unwrap();
    builder
        .add(&mut e, &first.oriented(Orientation::Forward))
        .unwrap();
    builder
        .add(&mut e, &last.oriented(Orientation::Reversed))
        .unwrap();
    e
}

/// The four edges of a unit square, chained.
pub fn square_edges() -> Vec<Edge> {
    let v: Vec<Vertex> = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
        .iter()
        .map(|&(x, y)| vertex(x, y))
        .collect();
    (0..4).map(|i| edge(&v[i], &v[(i + 1) % 4])).collect()
}

/// A new wire of the shapes, each added as it is.
pub fn wire(shapes: &[Shape]) -> Wire {
    let builder = BRepBuilder::default();
    let mut w = Wire::default();
    builder.make_wire(&mut w);
    for s in shapes {
        builder.add(&mut w, s).unwrap();
    }
    w
}

/// The edges as shapes.
pub fn shapes(edges: &[Edge]) -> Vec<Shape> {
    edges.iter().map(|e| e.clone().into_shape()).collect()
}
