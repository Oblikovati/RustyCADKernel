//! Shared fixtures for the TKBRep legacy suites, using ported TKTopAlgo/TKPrim builders.
//!
//! These originally replayed the builders while their toolkits were unavailable.
//! The explicit `edge_between` helper still permits a repeated vertex for topology-only
//! fixtures; ordinary MakeEdge/MakeWire/MakeBox fixtures now call the production ports.
#![allow(dead_code)]
pub mod ref_test_tools;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Edge, Solid, Vertex, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Vector3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeVertex, BRepBuilderAPIMakeWire,
};

/// `EXPECT_DOUBLE_EQ`: equal within 4 units in the last place.
pub fn expect_double_eq(a: f64, b: f64) {
    let ulps = (a.to_bits() as i64 - b.to_bits() as i64).unsigned_abs();
    assert!(a == b || ulps <= 4, "{a} != {b}");
}

/// `BRepLib::Precision()`, the default construction tolerance.
const PRECISION: f64 = precision::confusion();

/// `BRepBuilderAPI_MakeVertex(P)`: `BRep_Builder::MakeVertex(V, P, BRepLib::Precision())`.
pub fn make_vertex(point: Point3) -> Vertex {
    BRepBuilderAPIMakeVertex::new(point)
        .unwrap()
        .vertex()
        .unwrap()
}

/// The line through `first` towards `last` (`gp_Lin(P1, gp_Vec(P1, P2))`).
fn line_between(first: Point3, last: Point3) -> Handle<dyn GeomCurve> {
    let direction = Direction3::from_vector(Vector3::between(first, last)).unwrap();
    Handle::new(GeomLine::new(Axis1::new(first, direction))).into_cast()
}

/// `BRepBuilderAPI_MakeEdge(P1, P2)`: a line edge on `[0, |P1P2|]` between two new
/// vertices, the first FORWARD and the second REVERSED.
pub fn make_edge(first: Point3, last: Point3) -> Edge {
    BRepBuilderAPIMakeEdge::from_points(first, last)
        .unwrap()
        .edge()
        .unwrap()
}

/// `BRepBuilderAPI_MakeEdge(V1, V2)` for existing vertices: the line edge between their
/// points. For coincident points (which BRepLib refuses) the fake keeps an edge
/// without curve, so the topology suites can still link a vertex twice.
pub fn edge_between(first: &Vertex, last: &Vertex) -> Edge {
    let builder = BRepBuilder::default();
    let (p1, p2) = (point_of(first), point_of(last));
    if p1.distance(&p2) > PRECISION {
        return BRepBuilderAPIMakeEdge::from_vertices(first, last)
            .unwrap()
            .edge()
            .unwrap();
    }
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    let length = p1.distance(&p2);
    if length > f64::MIN_POSITIVE {
        builder
            .update_edge_curve(&edge, &line_between(p1, p2), PRECISION)
            .unwrap();
        builder.range(&edge, 0.0, length, false).unwrap();
    }
    link_vertices(&mut edge, first, last);
    edge
}

fn point_of(vertex: &Vertex) -> Point3 {
    rustycad_tkbrep::brep::BRepTool::pnt(vertex).unwrap()
}

/// Adds the vertices FORWARD then REVERSED, as BRepLib_MakeEdge does.
fn link_vertices(edge: &mut Edge, first: &Vertex, last: &Vertex) {
    let builder = BRepBuilder::default();
    builder
        .add(edge, &first.oriented(Orientation::Forward))
        .unwrap();
    builder
        .add(edge, &last.oriented(Orientation::Reversed))
        .unwrap();
}

/// `BRepBuilderAPI_MakeEdge(C)` and `BRepBuilderAPI_MakeEdge(C, p1, p2)`: the core of
/// `BRepLib_MakeEdge::Init(C, V1, V2, p1, p2)` without given vertices. A periodic
/// range is adjusted into the period; a closed curve gets one vertex at both ends.
/// Trimmed curves are replaced by their basis first ("kill trimmed curves").
pub fn make_edge_on_curve(curve: &Handle<dyn GeomCurve>, first: f64, last: f64) -> Edge {
    BRepBuilderAPIMakeEdge::from_curve_params(curve, first, last)
        .unwrap()
        .edge()
        .unwrap()
}

/// `BRepBuilderAPI_MakeWire(E).Wire()`: a wire holding the edge as given; a closed
/// edge makes a closed wire, as BRepLib_MakeWire checks the free vertices.
pub fn make_wire(edge: &Edge) -> Wire {
    BRepBuilderAPIMakeWire::from_edge(edge)
        .unwrap()
        .wire()
        .unwrap()
}

/// `BRepBuilderAPI_MakeEdge(C)`: the edge on the whole curve.
pub fn make_edge_on_whole_curve(curve: &Handle<dyn GeomCurve>) -> Edge {
    BRepBuilderAPIMakeEdge::from_curve(curve)
        .unwrap()
        .edge()
        .unwrap()
}

/// `BRepPrimAPI_MakeBox(dx, dy, dz).Shape()`: the solid of the wedge box
/// built by the ported `BRepPrimAPI_MakeBox` and `BRepPrim_GWedge`.
pub fn make_box(dx: f64, dy: f64, dz: f64) -> Solid {
    make_box_at(Point3::new(0.0, 0.0, 0.0), dx, dy, dz)
}

/// `BRepPrimAPI_MakeBox(P, dx, dy, dz).Shape()`: the box of [`make_box`] with its
/// corner at `origin` (the low corner for positive extents).
pub fn make_box_at(origin: Point3, dx: f64, dy: f64, dz: f64) -> Solid {
    BRepPrimAPIMakeBox::at_point(origin, dx, dy, dz)
        .unwrap()
        .solid()
        .unwrap()
}
