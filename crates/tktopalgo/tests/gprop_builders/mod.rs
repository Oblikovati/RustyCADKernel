//! The shapes of the BRepGProp behavior tests that BRepBuilderAPI_MakeEdge and
//! BRepBuilderAPI_MakeFace build upstream, built with BRep_Builder as those makers give
//! them (lines and circles on their ranges, plane faces without stored pcurves, or with
//! line pcurves on the plane's UV rectangle).
#![allow(dead_code)]
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Edge, Face, Vertex, Wire};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomCircle, GeomCurve, GeomLine, GeomPlane, GeomSurface,
};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Axis2, Circle3, Direction2, Direction3, Plane3, Point2, Point3};
use std::f64::consts::FRAC_PI_2;

pub fn vertex(point: Point3) -> Vertex {
    let mut v = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut v, point, precision::confusion())
        .unwrap();
    v
}

/// The edge BRepBuilderAPI_MakeEdge gives on `curve` over `[first, last]` between two
/// points: the curve, the vertices and the range.
pub fn edge_on(
    curve: Handle<dyn GeomCurve>,
    (first, last): (f64, f64),
    ends: (Point3, Point3),
) -> Edge {
    let b = BRepBuilder::default();
    let mut e = Edge::default();
    b.make_edge_with_curve(&mut e, &curve, precision::confusion())
        .unwrap();
    b.add(&mut e, &vertex(ends.0).oriented(Orientation::Forward))
        .unwrap();
    b.add(&mut e, &vertex(ends.1).oriented(Orientation::Reversed))
        .unwrap();
    b.range(&e, first, last, false).unwrap();
    e
}

/// `BRepBuilderAPI_MakeEdge(P1, P2)`: a line from `p1` on `[0, |P1P2|]`.
pub fn segment(p1: Point3, p2: Point3) -> Edge {
    let length = p1.distance(&p2);
    let direction = Direction3::from_xyz(p2.xyz() - p1.xyz()).unwrap();
    let line = Handle::new(GeomLine::from_point(p1, direction)).cast();
    edge_on(line, (0.0, length), (p1, p2))
}

/// A face on `plane` bounded by the wire of `edges` (no stored pcurve, as MakeFace on a
/// plane gives it).
pub fn planar_face(plane: Plane3, edges: &[Edge]) -> Face {
    let b = BRepBuilder::default();
    let mut w = Wire::default();
    b.make_wire(&mut w);
    for e in edges {
        b.add(&mut w, e).unwrap();
    }
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::from_pln(plane)).cast();
    let mut f = Face::default();
    b.make_face_on_surface(&mut f, &surface, precision::confusion())
        .unwrap();
    b.add(&mut f, &w).unwrap();
    f
}

/// The degree 1 B-spline surface with GaussPointsMax + 1 U intervals of the upstream case.
pub fn wide_spline_surface(intervals: i32) -> Handle<dyn GeomSurface> {
    let nb_poles = intervals + 1;
    let mut poles = Array2::with_bounds(1, nb_poles, 1, 2).unwrap();
    let mut knots = Array1::with_bounds(1, nb_poles).unwrap();
    let mut mults = Array1::with_bounds(1, nb_poles).unwrap();
    for i in 1..=nb_poles {
        *poles.change_value(i, 1).unwrap() = Point3::new(f64::from(i - 1), 0.0, 0.0);
        *poles.change_value(i, 2).unwrap() = Point3::new(f64::from(i - 1), 2.0, 0.0);
        knots[i] = f64::from(i - 1);
        mults[i] = if i == 1 || i == nb_poles { 2 } else { 1 };
    }
    let mut v_knots = Array1::with_bounds(1, 2).unwrap();
    v_knots[2] = 1.0;
    let mut v_mults = Array1::with_bounds(1, 2).unwrap();
    v_mults.init(2);
    let surface = GeomBSplineSurface::new(
        &poles,
        (&knots, &mults),
        (&v_knots, &v_mults),
        (1, 1),
        (false, false),
    );
    Handle::new(surface.unwrap()).cast()
}

/// The quarter disk of radius 4 of the upstream case: the arc (a circle edge on
/// [0, pi/2]), then the segments to the origin and back to the arc start.
pub fn quarter_disk(radius: f64) -> Face {
    let origin = Point3::new(0.0, 0.0, 0.0);
    let (start, end) = (Point3::new(radius, 0.0, 0.0), Point3::new(0.0, radius, 0.0));
    let z = Direction3::new(0.0, 0.0, 1.0).unwrap();
    let circle = Circle3::new(Axis2::new(origin, z).unwrap(), radius).unwrap();
    let arc = Handle::new(GeomCircle::from_circ(circle)).cast();
    let edges = [
        edge_on(arc, (0.0, FRAC_PI_2), (start, end)),
        segment(end, origin),
        segment(origin, start),
    ];
    planar_face(Plane3::new(origin, z).unwrap(), &edges)
}

/// `BRepBuilderAPI_MakeFace(gp_Pln(P, N), 0, 3, 0, 4)`: the rectangle of the plane's UV
/// bounds, its edges carrying 3d lines and line pcurves.
pub fn plane_rectangle(plane: Plane3, (u1, u2, v1, v2): (f64, f64, f64, f64)) -> Face {
    let b = BRepBuilder::default();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::from_pln(plane)).cast();
    let corners = [(u1, v1), (u2, v1), (u2, v2), (u1, v2)];
    let point = |(u, v): (f64, f64)| rustycad_tkmath::elslib::plane_value(u, v, plane.position());
    let mut f = Face::default();
    b.make_face_on_surface(&mut f, &surface, precision::confusion())
        .unwrap();
    let mut w = Wire::default();
    b.make_wire(&mut w);
    for k in 0..4 {
        let (a, c) = (corners[k], corners[(k + 1) % 4]);
        let e = segment(point(a), point(c));
        let length = ((c.0 - a.0).powi(2) + (c.1 - a.1).powi(2)).sqrt();
        let direction = Direction2::new((c.0 - a.0) / length, (c.1 - a.1) / length).unwrap();
        let line: Handle<dyn Geom2dCurve> =
            Handle::new(Geom2dLine::from_point(Point2::new(a.0, a.1), direction)).cast();
        b.update_edge_pcurve_on_face(&e, &line, &f, precision::confusion())
            .unwrap();
        b.add(&mut w, &e).unwrap();
    }
    b.add(&mut f, &w).unwrap();
    f
}
