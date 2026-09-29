//! Named fakes for the TKTopAlgo and TKPrim builders that the upstream TKBRep suites
//! call (BRepBuilderAPI_MakeVertex, BRepBuilderAPI_MakeEdge, BRepBuilderAPI_MakeWire,
//! BRepPrimAPI_MakeBox).
//!
//! TKTopAlgo and TKPrim are not ported, so these fakes replay what the native builders
//! do with the real BRep types: `BRep_Builder` vertices with the BRepLib precision,
//! edges on a line or a curve with the vertices and range of BRepLib_MakeEdge, and the
//! box of `BRepPrim_GWedge` (the `wedge_box` module), entity for entity. They are
//! replaced by the real builders when those toolkits are ported.
#![allow(dead_code)]
pub mod ref_test_tools;
mod wedge_box;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Edge, Solid, Vertex, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Vector3};

/// `EXPECT_DOUBLE_EQ`: equal within 4 units in the last place.
pub fn expect_double_eq(a: f64, b: f64) {
    let ulps = (a.to_bits() as i64 - b.to_bits() as i64).unsigned_abs();
    assert!(a == b || ulps <= 4, "{a} != {b}");
}

/// `BRepLib::Precision()`, the default construction tolerance.
const PRECISION: f64 = precision::confusion();

/// `BRepBuilderAPI_MakeVertex(P)`: `BRep_Builder::MakeVertex(V, P, BRepLib::Precision())`.
pub fn make_vertex(point: Point3) -> Vertex {
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, point, PRECISION)
        .unwrap();
    vertex
}

/// The line through `first` towards `last` (`gp_Lin(P1, gp_Vec(P1, P2))`).
fn line_between(first: Point3, last: Point3) -> Handle<dyn GeomCurve> {
    let direction = Direction3::from_vector(Vector3::between(first, last)).unwrap();
    Handle::new(GeomLine::new(Axis1::new(first, direction))).into_cast()
}

/// `BRepBuilderAPI_MakeEdge(P1, P2)`: a line edge on `[0, |P1P2|]` between two new
/// vertices, the first FORWARD and the second REVERSED.
pub fn make_edge(first: Point3, last: Point3) -> Edge {
    edge_between(&make_vertex(first), &make_vertex(last))
}

/// `BRepBuilderAPI_MakeEdge(V1, V2)` for existing vertices: the line edge between their
/// points. For coincident points (which BRepLib refuses) the fake keeps an edge
/// without curve, so the topology suites can still link a vertex twice.
pub fn edge_between(first: &Vertex, last: &Vertex) -> Edge {
    let builder = BRepBuilder::default();
    let (p1, p2) = (point_of(first), point_of(last));
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
    let curve = &untrimmed(curve.clone());
    let geometry = curve.get().unwrap();
    let (mut p1, mut p2) = (first, last);
    if geometry.is_periodic() {
        let (cf, cl) = (geometry.first_parameter(), geometry.last_parameter());
        (p1, p2) = elclib::adjust_periodic(cf, cl, precision::p_confusion(), p1, p2);
    } else if p1 > p2 {
        (p1, p2) = (p2, p1);
    }
    let (start, end) = (geometry.value(p1).unwrap(), geometry.value(p2).unwrap());
    let closed = start.distance(&end) <= PRECISION;
    let v1 = make_vertex(start);
    let v2 = if closed { v1.clone() } else { make_vertex(end) };
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder
        .make_edge_with_curve(&mut edge, curve, PRECISION)
        .unwrap();
    link_vertices(&mut edge, &v1, &v2);
    builder.range(&edge, p1, p2, false).unwrap();
    builder.degenerated(&edge, false).unwrap();
    edge.set_closed(closed).unwrap();
    edge
}

/// The innermost basis of nested `Geom_TrimmedCurve`s, the curve itself otherwise.
fn untrimmed(mut curve: Handle<dyn GeomCurve>) -> Handle<dyn GeomCurve> {
    loop {
        let trimmed = curve.cast::<GeomTrimmedCurve>();
        let Some(inner) = trimmed.get() else {
            return curve;
        };
        curve = inner.basis_curve();
    }
}

/// `BRepBuilderAPI_MakeWire(E).Wire()`: a wire holding the edge as given; a closed
/// edge makes a closed wire, as BRepLib_MakeWire checks the free vertices.
pub fn make_wire(edge: &Edge) -> Wire {
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, edge).unwrap();
    let (first, last) = rustycad_tkbrep::topexp::vertices(edge, false);
    wire.set_closed(!first.is_null() && first.is_same(&last))
        .unwrap();
    wire
}

/// `BRepBuilderAPI_MakeEdge(C)`: the edge on the whole curve.
pub fn make_edge_on_whole_curve(curve: &Handle<dyn GeomCurve>) -> Edge {
    let geometry = curve.get().unwrap();
    make_edge_on_curve(curve, geometry.first_parameter(), geometry.last_parameter())
}

/// `BRepPrimAPI_MakeBox(dx, dy, dz).Shape()`: the solid of the wedge box
/// ([`wedge_box`]), built as `BRepPrim_GWedge` builds it.
pub fn make_box(dx: f64, dy: f64, dz: f64) -> Solid {
    make_box_at(Point3::new(0.0, 0.0, 0.0), dx, dy, dz)
}

/// `BRepPrimAPI_MakeBox(P, dx, dy, dz).Shape()`: the box of [`make_box`] with its
/// corner at `origin` (the low corner for positive extents).
pub fn make_box_at(origin: Point3, dx: f64, dy: f64, dz: f64) -> Solid {
    wedge_box::make_wedge_box(origin, dx, dy, dz)
}
