//! Named fakes for the TKTopAlgo and TKPrim builders that the upstream TKBRep suites
//! call (BRepBuilderAPI_MakeVertex, BRepBuilderAPI_MakeEdge, BRepPrimAPI_MakeBox).
//!
//! TKTopAlgo and TKPrim are not ported, so these fakes replay what the native builders
//! do with the real BRep types: `BRep_Builder` vertices with the BRepLib precision,
//! edges on a line or a curve with the vertices and range of BRepLib_MakeEdge, and a
//! box whose six faces lie on planes with a p-curve for each edge, sharing twelve
//! edges and eight vertices as BRepPrim does. They are replaced by the real builders
//! when those toolkits are ported.
#![allow(dead_code)]
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Edge, Face, Shell, Solid, Vertex, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::elclib;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{Axis1, Axis2d, Axis3, Direction2, Direction3, Point3, Vector2, Vector3};

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
pub fn make_edge_on_curve(curve: &Handle<dyn GeomCurve>, first: f64, last: f64) -> Edge {
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

/// `BRepBuilderAPI_MakeEdge(C)`: the edge on the whole curve.
pub fn make_edge_on_whole_curve(curve: &Handle<dyn GeomCurve>) -> Edge {
    let geometry = curve.get().unwrap();
    make_edge_on_curve(curve, geometry.first_parameter(), geometry.last_parameter())
}

/// `BRepPrimAPI_MakeBox(dx, dy, dz).Shape()`: a solid with one shell of six planar
/// faces, each bounded by one wire of four edges with p-curves; the twelve line edges
/// and the eight vertices are shared as in the native box (each edge by two faces).
pub fn make_box(dx: f64, dy: f64, dz: f64) -> Solid {
    let corners: Vec<Point3> = (0..8)
        .map(|i| {
            let (x, y, z) = (i & 1, (i >> 1) & 1, (i >> 2) & 1);
            Point3::new(x as f64 * dx, y as f64 * dy, z as f64 * dz)
        })
        .collect();
    let vertices: Vec<Vertex> = corners.iter().map(|p| make_vertex(*p)).collect();
    // Edges along X (0-3), Y (4-7) and Z (8-11), each from its low to its high corner.
    let pairs = [
        (0, 1),
        (2, 3),
        (4, 5),
        (6, 7),
        (0, 2),
        (1, 3),
        (4, 6),
        (5, 7),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];
    let edges: Vec<(Edge, usize)> = pairs
        .iter()
        .map(|&(a, b)| (edge_between(&vertices[a], &vertices[b]), a))
        .collect();
    let mut shell = Shell::default();
    BRepBuilder::default().make_shell(&mut shell);
    for side in box_sides(&corners) {
        let face = face_of(&edges, &corners, &side);
        BRepBuilder::default().add(&mut shell, &face).unwrap();
    }
    let mut solid = Solid::default();
    BRepBuilder::default().make_solid(&mut solid);
    BRepBuilder::default().add(&mut solid, &shell).unwrap();
    solid
}

/// One face of the box: its plane (origin corner, normal, X direction) and its four
/// edges around the boundary with their orientation.
struct BoxSide {
    frame: Axis3,
    edges: [(usize, bool); 4],
}

/// The six sides: -X, +X, -Y, +Y, -Z, +Z.
fn box_sides(corners: &[Point3]) -> [BoxSide; 6] {
    let axis = |x: f64, y: f64, z: f64| Direction3::new(x, y, z).unwrap();
    let side = |origin: usize, normal: Direction3, x: Direction3, edges| BoxSide {
        frame: Axis3::new(corners[origin], normal, x).unwrap(),
        edges,
    };
    [
        side(
            0,
            axis(-1.0, 0.0, 0.0),
            axis(0.0, 0.0, 1.0),
            [(4, true), (6, true), (8, false), (10, true)],
        ),
        side(
            1,
            axis(1.0, 0.0, 0.0),
            axis(0.0, 0.0, 1.0),
            [(5, true), (7, true), (9, false), (11, true)],
        ),
        side(
            0,
            axis(0.0, -1.0, 0.0),
            axis(0.0, 0.0, 1.0),
            [(0, true), (2, true), (8, false), (9, true)],
        ),
        side(
            2,
            axis(0.0, 1.0, 0.0),
            axis(0.0, 0.0, 1.0),
            [(1, true), (3, true), (10, false), (11, true)],
        ),
        side(
            0,
            axis(0.0, 0.0, -1.0),
            axis(1.0, 0.0, 0.0),
            [(0, true), (1, true), (4, false), (5, true)],
        ),
        side(
            4,
            axis(0.0, 0.0, 1.0),
            axis(1.0, 0.0, 0.0),
            [(2, true), (3, true), (6, false), (7, true)],
        ),
    ]
}

/// A planar face with one wire of the given edges and a p-curve for each of them.
fn face_of(edges: &[(Edge, usize)], corners: &[Point3], side: &BoxSide) -> Face {
    let builder = BRepBuilder::default();
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(side.frame)).into_cast();
    let mut face = Face::default();
    builder
        .make_face_on_surface(&mut face, &plane, PRECISION)
        .unwrap();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    for &(index, forward) in &side.edges {
        let (edge, start) = &edges[index];
        let pcurve = pcurve_on_plane(side.frame, corners[*start], edge);
        builder
            .update_edge_pcurve_on_face(edge, &pcurve, &face, PRECISION)
            .unwrap();
        let orientation = if forward {
            Orientation::Forward
        } else {
            Orientation::Reversed
        };
        builder.add(&mut wire, &edge.oriented(orientation)).unwrap();
    }
    builder.add(&mut face, &wire).unwrap();
    face
}

/// The 2D line of an edge in the parametric space of a plane: through the edge start
/// with the direction of the edge, with the same parameter as the 3D line.
fn pcurve_on_plane(frame: Axis3, start: Point3, edge: &Edge) -> Handle<dyn Geom2dCurve> {
    let (curve, _, last) = rustycad_tkbrep::brep::BRepTool::curve(edge).unwrap();
    let end = curve.get().unwrap().value(last).unwrap();
    let (u0, v0) = elslib::plane_parameters(frame, start);
    let (u1, v1) = elslib::plane_parameters(frame, end);
    let direction = Direction2::from_vector(Vector2::new(u1 - u0, v1 - v0)).unwrap();
    let origin = rustycad_tkmath::gp::Point2::new(u0, v0);
    Handle::new(Geom2dLine::new(Axis2d::new(origin, direction))).into_cast()
}
