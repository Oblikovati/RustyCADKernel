//! Unit tests of ChFi2d: the corner of two segments filleted by both algorithms and the
//! API, the chamfer API, and the builder editing a square face.
use super::*;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon,
    BRepBuilderAPIMakeVertex, BRepBuilderAPIMakeWire,
};

const TOL: f64 = 1e-9;

fn pt(x: f64, y: f64) -> Point3 {
    Point3::new(x, y, 0.0)
}

fn segment(p1: Point3, p2: Point3) -> Edge {
    BRepBuilderAPIMakeEdge::from_points(p1, p2)
        .and_then(|m| m.edge())
        .expect("segment")
}

fn vertex_at(p: Point3) -> Vertex {
    BRepBuilderAPIMakeVertex::new(p)
        .and_then(|m| m.vertex())
        .expect("vertex")
}

fn segment_between(v1: &Vertex, v2: &Vertex) -> Edge {
    BRepBuilderAPIMakeEdge::from_vertices(v1, v2)
        .and_then(|m| m.edge())
        .expect("segment")
}

/// The right-angle corner at the origin: (10, 0) -> (0, 0) -> (0, 10), the two edges
/// sharing the vertex at the origin.
fn corner() -> (Edge, Edge) {
    let (a, o, b) = (
        vertex_at(pt(10.0, 0.0)),
        vertex_at(pt(0.0, 0.0)),
        vertex_at(pt(0.0, 10.0)),
    );
    (segment_between(&a, &o), segment_between(&o, &b))
}

/// The points of the vertices of an edge, in edge orientation order.
fn end_points(e: &Edge) -> [Point3; 2] {
    let (v1, v2) = topexp::vertices(e, true);
    [
        BRepTool::pnt(&v1).expect("first"),
        BRepTool::pnt(&v2).expect("last"),
    ]
}

/// The point at the middle of the parameter range of an edge.
fn middle(e: &Edge) -> Point3 {
    let c = BRepAdaptorCurve::with_edge(e).expect("adaptor");
    let (f, l) = (
        c.first_parameter().expect("first"),
        c.last_parameter().expect("last"),
    );
    c.value(0.5 * (f + l)).expect("value")
}

/// Asserts the edge is the quarter circle of radius 1 around (1, 1) between (1, 0) and
/// (0, 1).
fn assert_unit_fillet(f: &Edge) {
    assert!(!f.is_null());
    let ends = end_points(f);
    let touches = |p: Point3| ends.iter().any(|q| q.distance(&p) < TOL);
    assert!(
        touches(pt(1.0, 0.0)) && touches(pt(0.0, 1.0)),
        "fillet ends {ends:?}"
    );
    assert!((middle(f).distance(&pt(1.0, 1.0)) - 1.0).abs() < TOL);
}

fn square_face() -> Face {
    let mut poly = BRepBuilderAPIMakePolygon::new();
    for p in [pt(0.0, 0.0), pt(10.0, 0.0), pt(10.0, 10.0), pt(0.0, 10.0)] {
        poly.add_point(p).expect("point");
    }
    poly.close().expect("close");
    let wire = poly.wire().expect("wire");
    BRepBuilderAPIMakeFace::from_wire(&wire, false)
        .and_then(|m| m.face())
        .expect("face")
}

fn shapes_of(face: &Face, kind: ShapeEnum) -> IndexedMapOfShape {
    let mut map = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(face, kind, &mut map).expect("map");
    map
}

fn edge_count(face: &Face) -> usize {
    let mut ex = Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape).expect("explorer");
    let mut n = 0;
    while ex.more() {
        n += 1;
        ex.next().expect("next");
    }
    n
}

#[test]
fn common_vertex_is_the_corner() {
    let (e1, e2) = corner();
    let v = common_vertex(&e1, &e2).expect("common vertex");
    assert!(BRepTool::pnt(&v).expect("point").distance(&pt(0.0, 0.0)) < TOL);
    assert!(common_vertex(&e1, &segment(pt(5.0, 5.0), pt(6.0, 6.0))).is_none());
}

#[test]
fn find_connected_edges_of_a_square_vertex() {
    let face = square_face();
    let v = topods::vertex(shapes_of(&face, ShapeEnum::Vertex).find_key(1).expect("v1"))
        .expect("vertex");
    let (mut e1, mut e2) = (Edge::default(), Edge::default());
    let status = find_connected_edges(&face, &v, &mut e1, &mut e2).expect("status");
    assert_eq!(status, ConstructionError::IsDone);
    assert!(!e1.is_null() && !e2.is_null() && !e1.is_same(&e2));
}

#[test]
fn analytical_fillet_of_a_corner() {
    let (e1, e2) = corner();
    let mut algo = ChFi2dAnaFilletAlgo::with_edges(&e1, &e2, &Plane3::default()).expect("init");
    assert!(algo.perform(1.0).expect("perform"));
    let (mut r1, mut r2) = (Edge::default(), Edge::default());
    assert_unit_fillet(&algo.result(&mut r1, &mut r2));
    assert!(
        end_points(&r1)
            .iter()
            .any(|p| p.distance(&pt(1.0, 0.0)) < TOL)
    );
    assert!(
        end_points(&r2)
            .iter()
            .any(|p| p.distance(&pt(0.0, 1.0)) < TOL)
    );
}

#[test]
fn iterative_fillet_of_a_corner() {
    let (e1, e2) = corner();
    let mut algo = ChFi2dFilletAlgo::with_edges(&e1, &e2, &Plane3::default()).expect("init");
    assert!(algo.perform(1.0).expect("perform"));
    assert!(algo.nb_results(pt(0.0, 0.0)).expect("count") >= 1);
    let (mut r1, mut r2) = (Edge::default(), Edge::default());
    let f = algo
        .result(pt(0.0, 0.0), &mut r1, &mut r2, -1)
        .expect("result");
    let ends = end_points(&f);
    assert!(
        ends.iter().all(|p| p.distance(&pt(1.0, 1.0)) - 1.0 < 1e-6),
        "fillet ends {ends:?}"
    );
}

#[test]
fn fillet_api_on_the_wire_of_a_corner() {
    let (e1, e2) = corner();
    let wire = BRepBuilderAPIMakeWire::from_two_edges(&e1, &e2)
        .and_then(|m| m.wire())
        .expect("wire");
    let mut api = ChFi2dFilletAPI::with_wire(&wire, &Plane3::default()).expect("init");
    assert!(api.perform(1.0).expect("perform"));
    assert_eq!(api.nb_results(pt(0.0, 0.0)).expect("count"), 1);
    let (mut r1, mut r2) = (Edge::default(), Edge::default());
    assert_unit_fillet(
        &api.result(pt(0.0, 0.0), &mut r1, &mut r2, -1)
            .expect("result"),
    );
}

#[test]
fn analytical_fillet_refuses_a_radius_larger_than_the_edges() {
    let (e1, e2) = corner();
    let mut algo = ChFi2dAnaFilletAlgo::with_edges(&e1, &e2, &Plane3::default()).expect("init");
    assert!(!algo.perform(12.0).expect("perform"));
}

#[test]
fn chamfer_api_of_a_corner() {
    let (e1, e2) = corner();
    let mut api = ChFi2dChamferAPI::with_edges(&e1, &e2);
    assert!(api.perform().expect("perform"));
    let (mut r1, mut r2) = (Edge::default(), Edge::default());
    let chamfer = api.result(&mut r1, &mut r2, (1.0, 2.0)).expect("result");
    let [a, b] = end_points(&chamfer);
    assert!(
        (a.distance(&b) - 5.0_f64.sqrt()).abs() < TOL,
        "chamfer ends {a:?} {b:?}"
    );
}

#[test]
fn builder_adds_and_removes_a_fillet() {
    let face = square_face();
    let mut b = ChFi2dBuilder::with_face(&face).expect("init");
    assert_eq!(b.status(), ConstructionError::Ready);
    let v = topods::vertex(shapes_of(&face, ShapeEnum::Vertex).find_key(1).expect("v1"))
        .expect("vertex");
    let fillet = b.add_fillet(&v, 1.0).expect("fillet");
    assert_eq!(b.status(), ConstructionError::IsDone);
    assert_eq!((b.nb_fillet(), edge_count(&b.result())), (1, 5));
    assert!(
        (middle(&fillet).distance(&BRepTool::pnt(&v).expect("point")) - (2.0_f64.sqrt() - 1.0))
            .abs()
            < TOL
    );
    let removed = b.remove_fillet(&fillet).expect("remove");
    assert!(!removed.is_null());
    assert_eq!((b.nb_fillet(), edge_count(&b.result())), (0, 4));
}

#[test]
fn builder_adds_a_chamfer_and_refuses_a_negative_distance() {
    let face = square_face();
    let mut b = ChFi2dBuilder::with_face(&face).expect("init");
    let edges = shapes_of(&face, ShapeEnum::Edge);
    let e = |i: usize| topods::edge(edges.find_key(i).expect("edge")).expect("edge");
    b.add_chamfer(&e(1), &e(2), -1.0, 1.0)
        .expect("refused chamfer");
    assert_eq!(b.status(), ConstructionError::ParametersError);
    let chamfer = b.add_chamfer(&e(1), &e(2), 1.0, 2.0).expect("chamfer");
    assert_eq!(b.status(), ConstructionError::IsDone);
    assert_eq!((b.nb_chamfer(), edge_count(&b.result())), (1, 5));
    assert!(b.is_modified(&e(1)) && b.has_descendant(&e(1)));
    let [p, q] = end_points(&chamfer);
    assert!((p.distance(&q) - 5.0_f64.sqrt()).abs() < TOL);
}
