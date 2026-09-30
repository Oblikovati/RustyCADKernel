//! Behavior tests of BRepBndLib after the cases of the upstream suite
//! `BRepBndLib_Test.cxx`, which is not mapped: it builds its shapes with
//! BRepBuilderAPI_MakeEdge/MakeFace and BRepPrimAPI_MakeCylinder and meshes them with
//! BRepMesh_IncrementalMesh (TKMesh), none of which the port provides (TKMesh is not in
//! the shared native build either, so the unchanged suite does not link there). Here the
//! same shapes are built with BRep_Builder: the edge MakeEdge gives on a curve (its
//! vertices at the curve ends, tolerance 1e-7), the plane face MakeFace gives on an
//! infinite plane (no wire, natural restriction), and the 26-segment 3d polygon of
//! deflection 0.1 that BRepMesh gives the unit circle (its nodes at the angles
//! 2 pi k / 26, which the reference bounds of the upstream cases pin). Each case checks
//! the same properties with the same values and tolerances.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCircle, GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Axis2, Direction3, Plane3, Point3};
use rustycad_tkmath::poly::Polygon3D;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use std::f64::consts::PI;

fn expect_same_bounds(first: &BoundingBox3, second: &BoundingBox3) {
    assert_eq!(first.get().unwrap(), second.get().unwrap());
}

fn expect_bounds(bounds: &BoundingBox3, expected: [f64; 6]) {
    // Bnd_Box::Limits is (Xmin, Xmax, Ymin, Ymax, Zmin, Zmax); get() is xmin, ymin,
    // zmin, xmax, ymax, zmax, as the six-argument Get.
    let limits = bounds.get().unwrap();
    for (value, want) in limits.iter().zip(expected) {
        assert!((value - want).abs() <= 1.0e-12, "{value} != {want}");
    }
}

/// `BRepBuilderAPI_MakeEdge(C)` on a closed curve: the edge with one vertex at both ends.
fn edge_on_curve(curve: Handle<dyn GeomCurve>, first: f64, last: f64) -> Edge {
    let b = BRepBuilder::default();
    let (start, end) = {
        let c = curve.get().unwrap();
        (c.value(first).unwrap(), c.value(last).unwrap())
    };
    let mut edge = Edge::default();
    b.make_edge_with_curve(&mut edge, &curve, precision::confusion())
        .unwrap();
    let (mut v1, mut v2) = (Vertex::default(), Vertex::default());
    b.make_vertex_at(&mut v1, start, precision::confusion())
        .unwrap();
    let closed = start.distance(&end) <= precision::confusion();
    if !closed {
        b.make_vertex_at(&mut v2, end, precision::confusion())
            .unwrap();
    }
    let v2 = if closed { v1.clone() } else { v2 };
    b.add(&mut edge, &v1.oriented(Orientation::Forward))
        .unwrap();
    b.add(&mut edge, &v2.oriented(Orientation::Reversed))
        .unwrap();
    b.range(&edge, first, last, false).unwrap();
    edge
}

fn make_circle_edge() -> Edge {
    let frame = Axis2::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let circle: Handle<dyn GeomCurve> = Handle::new(GeomCircle::new(frame, 1.0).unwrap()).cast();
    edge_on_curve(circle, 0.0, 2.0 * PI)
}

/// The 26-segment polygon of deflection 0.1 that BRepMesh gives the unit circle at 0.1.
fn meshed_circle_edge() -> Edge {
    let edge = make_circle_edge();
    let mut nodes = Array1::with_bounds(1, 27).unwrap();
    for k in 0..27 {
        let angle = 2.0 * PI * f64::from(k) / 26.0;
        nodes[k + 1] = Point3::new(angle.cos(), angle.sin(), 0.0);
    }
    let polygon = Polygon3D::from_nodes(&nodes);
    polygon.set_deflection(0.1);
    BRepBuilder::default()
        .update_edge_polygon3d(&edge, &Handle::new(polygon), &Default::default())
        .unwrap();
    edge
}

fn make_polygon3d_edge() -> Edge {
    let mut nodes = Array1::with_bounds(1, 5).unwrap();
    nodes[1] = Point3::new(1.0, 0.0, 0.0);
    nodes[2] = Point3::new(0.0, 1.0, 0.0);
    nodes[3] = Point3::new(-1.0, 0.0, 0.0);
    nodes[4] = Point3::new(0.0, -1.0, 0.0);
    nodes[5] = Point3::new(1.0, 0.0, 0.0);
    let polygon = Handle::new(Polygon3D::from_nodes(&nodes));
    let mut edge = Edge::default();
    BRepBuilder::default()
        .make_edge_with_polygon3d(&mut edge, &polygon)
        .unwrap();
    edge
}

fn add_both(edge: &Edge) -> (BoundingBox3, BoundingBox3) {
    let (mut with, mut without) = (BoundingBox3::default(), BoundingBox3::default());
    BRepBndLib::add(edge, &mut with, true).unwrap();
    BRepBndLib::add(edge, &mut without, false).unwrap();
    (with, without)
}

#[test]
// BRepBndLibTest.OCC30292_GeometricEdgeWithoutDiscreteRepresentation
fn occ30292_geometric_edge_without_discrete_representation() {
    let (with, without) = add_both(&make_circle_edge());
    expect_same_bounds(&with, &without);
}

#[test]
// BRepBndLibTest.OCC30292_MeshedGeometricEdgeUsesRequestedRepresentation
fn occ30292_meshed_geometric_edge_uses_requested_representation() {
    let (with, without) = add_both(&meshed_circle_edge());
    expect_bounds(
        &with,
        [
            -1.1000001,
            -1.0927089740980542,
            -0.1000001,
            1.1000001,
            1.092708974098054,
            0.1000001,
        ],
    );
    expect_bounds(
        &without,
        [
            -1.0000001, -1.0000001, -1.0e-7, 1.0000001, 1.0000001, 1.0e-7,
        ],
    );
}

// The third edge of BRepPrimAPI_MakeCylinder(1, 1) is its bottom circle, meshed as the
// unit circle is; the case reduces to the one above on that edge.
#[test]
// BRepBndLibTest.OCC30292_MeshedCylinderEdgeUsesPolygon3d
fn occ30292_meshed_cylinder_edge_uses_polygon3d() {
    let (with, without) = add_both(&meshed_circle_edge());
    expect_bounds(
        &with,
        [
            -1.1000001,
            -1.0927089740980542,
            -0.1000001,
            1.1000001,
            1.092708974098054,
            0.1000001,
        ],
    );
    expect_bounds(
        &without,
        [
            -1.0000001, -1.0000001, -1.0e-7, 1.0000001, 1.0000001, 1.0e-7,
        ],
    );
}

#[test]
// BRepBndLibTest.OCC30292_Polygon3dEdgeIgnoresRepresentationFlag
fn occ30292_polygon3d_edge_ignores_representation_flag() {
    let (with, without) = add_both(&make_polygon3d_edge());
    expect_same_bounds(&with, &without);
}

#[test]
// BRepBndLibTest.OCC23575_PlanarBSplineEdgeBounds
fn occ23575_planar_bspline_edge_bounds() {
    let mut poles = Array1::with_bounds(1, 4).unwrap();
    poles[1] = Point3::new(0.0, 0.0, 0.0);
    poles[2] = Point3::new(0.0, 1.0, 0.0);
    poles[3] = Point3::new(1.0, 1.0, 0.0);
    poles[4] = Point3::new(1.0, 0.0, 0.0);
    let mut weights = Array1::with_bounds(1, 4).unwrap();
    weights.init(1.0);
    let mut knots = Array1::with_bounds(1, 2).unwrap();
    knots[1] = 0.0;
    knots[2] = 1.0;
    let mut mults = Array1::with_bounds(1, 2).unwrap();
    mults[1] = 4;
    mults[2] = 4;
    let curve = GeomBSplineCurve::with_weights(&poles, &weights, &knots, &mults, 3, false).unwrap();
    let edge = edge_on_curve(Handle::new(curve).cast(), 0.0, 1.0);
    let mut bounds = BoundingBox3::default();
    BRepBndLib::add(&edge, &mut bounds, true).unwrap();
    assert!(!bounds.is_void());
    let [xmin, ymin, zmin, xmax, ymax, zmax] = bounds.get().unwrap();
    let tolerance = 0.1;
    assert!((xmin - 0.0).abs() <= tolerance);
    assert!((ymin - 0.0).abs() <= tolerance);
    assert!((zmin - 0.0).abs() <= tolerance);
    assert!((xmax - 1.0).abs() <= tolerance);
    assert!((ymax - 1.0).abs() <= tolerance);
    assert!((zmax - 0.0).abs() <= tolerance);
    assert!(zmax - zmin <= 1.0e-6);
}

#[test]
// BRepBndLibTest.OCC6503_InfinitePlaneBounds
fn occ6503_infinite_plane_bounds() {
    let plane = Plane3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::from_pln(plane)).cast();
    let b = BRepBuilder::default();
    let mut face = Face::default();
    b.make_face_on_surface(&mut face, &surface, precision::confusion())
        .unwrap();
    b.natural_restriction(&face, true).unwrap();
    let mut bounds = BoundingBox3::default();
    BRepBndLib::add(&face, &mut bounds, true).unwrap();
    assert!(!bounds.is_void());
    let [xmin, ymin, zmin, xmax, ymax, zmax] = bounds.get().unwrap();
    let open_bound = precision::infinite() / 2.0;
    assert_eq!(xmin, -open_bound);
    assert_eq!(xmax, open_bound);
    assert_eq!(ymin, -open_bound);
    assert_eq!(ymax, open_bound);
    assert!((zmin - -1.0e-7).abs() <= 1.0e-12);
    assert!((zmax - 1.0e-7).abs() <= 1.0e-12);
}

#[test]
fn add_close_adds_the_exact_curve_without_tolerance() {
    let mut bounds = BoundingBox3::default();
    BRepBndLib::add_close(&make_circle_edge(), &mut bounds).unwrap();
    let limits = bounds.get().unwrap();
    let expected = [-1.0, -1.0, 0.0, 1.0, 1.0, 0.0];
    for (value, want) in limits.iter().zip(expected) {
        assert!((value - want).abs() <= 1.0e-12, "{value} != {want}");
    }
}

#[test]
fn add_optimal_bounds_the_curve_tighter_than_add() {
    let mut poles = Array1::with_bounds(1, 4).unwrap();
    poles[1] = Point3::new(0.0, 0.0, 0.0);
    poles[2] = Point3::new(0.0, 1.0, 0.0);
    poles[3] = Point3::new(1.0, 1.0, 0.0);
    poles[4] = Point3::new(1.0, 0.0, 0.0);
    let mut knots = Array1::with_bounds(1, 2).unwrap();
    knots[1] = 0.0;
    knots[2] = 1.0;
    let mut mults = Array1::with_bounds(1, 2).unwrap();
    mults[1] = 4;
    mults[2] = 4;
    let curve = GeomBSplineCurve::new(&poles, &knots, &mults, 3, false).unwrap();
    let edge = edge_on_curve(Handle::new(curve).cast(), 0.0, 1.0);
    let (mut optimal, mut coarse) = (BoundingBox3::default(), BoundingBox3::default());
    BRepBndLib::add_optimal(&edge, &mut optimal, true, false).unwrap();
    BRepBndLib::add(&edge, &mut coarse, true).unwrap();
    let ymax = optimal.get().unwrap()[4];
    // The cubic with pole ordinates 0, 1, 1, 0 peaks at 3/4 (t = 1/2).
    assert!((ymax - 0.75).abs() <= 1.0e-6, "{ymax}");
    assert!(ymax < coarse.get().unwrap()[4]);
}

#[test]
fn add_optimal_uses_the_vertex_tolerance_only_on_request() {
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, Point3::new(1.0, 2.0, 3.0), 0.5)
        .unwrap();
    let (mut with, mut without) = (BoundingBox3::default(), BoundingBox3::default());
    BRepBndLib::add_optimal(&vertex, &mut with, true, true).unwrap();
    BRepBndLib::add_optimal(&vertex, &mut without, true, false).unwrap();
    assert_eq!(with.get().unwrap(), [0.5, 1.5, 2.5, 1.5, 2.5, 3.5]);
    assert_eq!(without.get().unwrap(), [1.0, 2.0, 3.0, 1.0, 2.0, 3.0]);
}
