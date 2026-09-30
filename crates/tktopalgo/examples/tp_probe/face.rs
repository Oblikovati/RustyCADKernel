//! The face reader of the TKTopAlgo probe, the Rust twin of
//! tools/oracle/tktopalgo_probe_face.cxx (`tp_face`, `tp_edge_index`): a face built with
//! BRep_Builder from numbers, and the index of an edge among the edges of the face.
//! The record layout is documented in the native file. It only reads through
//! `Reader::real`, so the behavior tests include it by path with the bare reader.
use super::print::Reader;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Face, Shape, Vertex, Wire};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomConicalSurface, GeomCurve, GeomCylindricalSurface,
    GeomLine, GeomPlane, GeomRectangularTrimmedSurface, GeomSphericalSurface, GeomSurface,
    GeomToroidalSurface,
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{
    Axis2d, Axis3, Direction2, Direction3, Point2, Point3, Vector2, Vector3,
};

/// `TpFace`: the face and its edges in record order.
pub struct TpFace {
    pub face: Face,
    pub edges: Vec<Edge>,
}

pub fn point(r: &mut Reader<'_>) -> Point3 {
    let (x, y) = (r.real(), r.real());
    Point3::new(x, y, r.real())
}
pub fn direction(r: &mut Reader<'_>) -> Result<Direction3, Failure> {
    let (x, y) = (r.real(), r.real());
    Direction3::new(x, y, r.real())
}
pub fn point2d(r: &mut Reader<'_>) -> Point2 {
    let x = r.real();
    Point2::new(x, r.real())
}
pub fn orientation(r: &mut Reader<'_>) -> Orientation {
    Orientation::ALL[r.integer() as usize]
}

pub fn array_failure<E: std::fmt::Debug>(error: E) -> Failure {
    Failure::new(&format!("{error:?}"))
}

/// `tp_poles`: a grid of nu x nv poles, U rows first.
pub fn poles(r: &mut Reader<'_>, nu: i32, nv: i32) -> Result<Array2<'static, Point3>, Failure> {
    let mut grid = Array2::with_bounds(1, nu, 1, nv).map_err(array_failure)?;
    for i in 1..=nu {
        for j in 1..=nv {
            grid.set_value(i, j, point(r)).map_err(array_failure)?;
        }
    }
    Ok(grid)
}

/// `tp_knots`: `n (knot mult)...` of one direction.
pub fn knots(r: &mut Reader<'_>) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
    let n = r.integer();
    let mut values = Array1::<f64>::with_bounds(1, n).map_err(array_failure)?;
    let mut mults = Array1::<i32>::with_bounds(1, n).map_err(array_failure)?;
    for i in 1..=n {
        values[i] = r.real();
        mults[i] = r.integer();
    }
    Ok((values, mults))
}

/// `tp_bspline_surface`: degrees, poles, then the U and V knots.
pub fn bspline_surface(r: &mut Reader<'_>) -> Result<Handle<dyn GeomSurface>, Failure> {
    let (udeg, vdeg) = (r.integer(), r.integer());
    let (nu, nv) = (r.integer(), r.integer());
    let grid = poles(r, nu, nv)?;
    let (uk, um) = knots(r)?;
    let (vk, vm) = knots(r)?;
    let surface =
        GeomBSplineSurface::new(&grid, (&uk, &um), (&vk, &vm), (udeg, vdeg), (false, false))?;
    Ok(Handle::new(surface).cast())
}

/// The surface kinds 4 (cone), 5 (torus), 6 (Bezier) and 7 (B-spline).
fn other_surface(
    r: &mut Reader<'_>,
    kind: i32,
    (p, n): (Point3, Direction3),
) -> Result<Handle<dyn GeomSurface>, Failure> {
    Ok(match kind {
        4 => {
            let radius = r.real();
            let frame = Axis3::from_normal(p, n)?;
            Handle::new(GeomConicalSurface::new(frame, r.real(), radius)?).cast()
        }
        5 => {
            let major = r.real();
            let frame = Axis3::from_normal(p, n)?;
            Handle::new(GeomToroidalSurface::new(frame, major, r.real())?).cast()
        }
        6 => {
            let (nu, nv) = (r.integer(), r.integer());
            Handle::new(GeomBezierSurface::new(&poles(r, nu, nv)?)?).cast()
        }
        _ => bspline_surface(r)?,
    })
}

/// `tp_surface`: a plane, cylinder, sphere, trimmed plane, cone, torus, Bezier or
/// B-spline surface.
fn surface(r: &mut Reader<'_>) -> Result<Handle<dyn GeomSurface>, Failure> {
    let kind = r.integer();
    let (p, n) = (point(r), direction(r)?);
    if kind >= 4 {
        return other_surface(r, kind, (p, n));
    }
    Ok(match kind {
        0 => Handle::new(GeomPlane::from_point(p, n)?).cast(),
        1 => Handle::new(GeomCylindricalSurface::new(
            Axis3::from_normal(p, n)?,
            r.real(),
        )?)
        .cast(),
        2 => Handle::new(GeomSphericalSurface::new(
            Axis3::from_normal(p, n)?,
            r.real(),
        )?)
        .cast(),
        _ => {
            let (u1, u2, v1, v2) = (r.real(), r.real(), r.real(), r.real());
            let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::from_point(p, n)?).cast();
            Handle::new(GeomRectangularTrimmedSurface::new(
                &plane,
                (u1, u2),
                (v1, v2),
                (true, true),
            )?)
            .cast()
        }
    })
}

/// `tp_segment`: the line from `a` towards `b` and its length.
fn segment(a: Point2, b: Point2) -> Result<(Handle<dyn Geom2dCurve>, f64), Failure> {
    let line = Geom2dLine::new(Axis2d::new(
        a,
        Direction2::from_vector(Vector2::between(a, b))?,
    ));
    Ok((Handle::new(line).cast(), a.distance(b)))
}

/// `tp_pcurves`: the pcurves of one edge on `face` and their range.
fn pcurves(r: &mut Reader<'_>, e: &Edge, face: &Face, kind: i32, tol: f64) -> Result<(), Failure> {
    let b = BRepBuilder::default();
    if kind == 5 {
        return Ok(());
    }
    if kind == 3 {
        let (a, end) = (point2d(r), point2d(r));
        let (du, dv) = (r.real(), r.real());
        let (c1, last) = segment(a, end)?;
        let shifted = Point2::new(a.x() + du, a.y() + dv);
        let dir = Direction2::from_vector(Vector2::between(a, end))?;
        let c2: Handle<dyn Geom2dCurve> =
            Handle::new(Geom2dLine::new(Axis2d::new(shifted, dir))).cast();
        b.update_edge_pcurves_on_face(e, (&c1, &c2), face, tol)?;
        return b.range_on_face(e, face, 0.0, last);
    }
    if kind == 6 {
        return segment_with_line(r, e, face, tol);
    }
    let (c, first, last) = single_pcurve(r, kind)?;
    b.update_edge_pcurve_on_face(e, &c, face, tol)?;
    b.range_on_face(e, face, first, last)
}

/// The kind 6: the segment pcurve and the 3d line from S(u1, v1) towards S(u2, v2),
/// both on [0, length].
fn segment_with_line(r: &mut Reader<'_>, e: &Edge, face: &Face, tol: f64) -> Result<(), Failure> {
    let b = BRepBuilder::default();
    let (a, end) = (point2d(r), point2d(r));
    let (c, last) = segment(a, end)?;
    let s = BRepTool::surface(face)?;
    let (pa, pb) = (s.value(a.x(), a.y())?, s.value(end.x(), end.y())?);
    let line = GeomLine::from_point(pa, Direction3::from_vector(Vector3::between(pa, pb))?);
    let curve: Handle<dyn GeomCurve> = Handle::new(line).cast();
    b.update_edge_curve(e, &curve, tol)?;
    b.update_edge_pcurve_on_face(e, &c, face, tol)?;
    b.range(e, 0.0, last, false)
}

/// The pcurve of the kinds 0 (segment), 1 (arc), 2 (Bezier) and 4 (line on a range).
fn single_pcurve(
    r: &mut Reader<'_>,
    kind: i32,
) -> Result<(Handle<dyn Geom2dCurve>, f64, f64), Failure> {
    match kind {
        0 => {
            let (a, end) = (point2d(r), point2d(r));
            let (c, last) = segment(a, end)?;
            Ok((c, 0.0, last))
        }
        1 => {
            let (center, radius, first, last) = (point2d(r), r.real(), r.real(), r.real());
            let axis = Axis2d::new(center, Direction2::new(1.0, 0.0)?);
            let circle = Geom2dCircle::from_axis(axis, radius, r.flag())?;
            Ok((Handle::new(circle).cast(), first, last))
        }
        2 => {
            let n = r.integer();
            let mut poles = Array1::<Point2>::with_bounds(1, n)
                .map_err(|error| Failure::new(&format!("{error:?}")))?;
            for i in 1..=n {
                poles[i] = point2d(r);
            }
            Ok((
                Handle::new(Geom2dBezierCurve::new(&poles)?).cast(),
                0.0,
                1.0,
            ))
        }
        _ => {
            let p = point2d(r);
            let (dx, dy) = (r.real(), r.real());
            let (first, last) = (r.real(), r.real());
            let line = Geom2dLine::new(Axis2d::new(p, Direction2::new(dx, dy)?));
            Ok((Handle::new(line).cast(), first, last))
        }
    }
}

/// `tp_edge`: an edge without 3d curve, its pcurves, vertices and degenerated flag.
fn edge(r: &mut Reader<'_>, face: &Face, vertices: &[Vertex]) -> Result<Edge, Failure> {
    let b = BRepBuilder::default();
    let mut e = Edge::default();
    b.make_edge(&mut e)?;
    let (kind, vf, vl, degenerated, tol) =
        (r.integer(), r.integer(), r.integer(), r.flag(), r.real());
    pcurves(r, &e, face, kind, tol)?;
    if vf >= 0 {
        b.add(
            &mut e,
            &vertices[vf as usize].oriented(Orientation::Forward),
        )?;
    }
    if vl >= 0 {
        b.add(
            &mut e,
            &vertices[vl as usize].oriented(Orientation::Reversed),
        )?;
    }
    b.degenerated(&e, degenerated)?;
    Ok(e)
}

/// `tp_face`: the face of one record.
pub fn tp_face(r: &mut Reader<'_>) -> Result<TpFace, Failure> {
    let b = BRepBuilder::default();
    let s = surface(r)?;
    let mut face = Face::default();
    b.make_face_on_surface(&mut face, &s, r.real())?;
    let mut vertices = Vec::new();
    for _ in 0..r.integer() {
        let (u, w) = (r.real(), r.real());
        let mut v = Vertex::default();
        b.make_vertex_at(&mut v, s.value(u, w)?, r.real())?;
        vertices.push(v);
    }
    let mut edges = Vec::new();
    for _ in 0..r.integer() {
        edges.push(edge(r, &face, &vertices)?);
    }
    for _ in 0..r.integer() {
        let wire = wire(r, &edges)?;
        b.add(&mut face, &wire)?;
    }
    face.set_orientation(orientation(r));
    Ok(TpFace { face, edges })
}

/// One wire of `tp_face`: its edges with their orientations, then its orientation.
fn wire(r: &mut Reader<'_>, edges: &[Edge]) -> Result<Wire, Failure> {
    let b = BRepBuilder::default();
    let mut w = Wire::default();
    b.make_wire(&mut w);
    let o = orientation(r);
    for _ in 0..r.integer() {
        let k = r.integer() as usize;
        b.add(&mut w, &edges[k].oriented(orientation(r)))?;
    }
    w.set_orientation(o);
    Ok(w)
}

/// `tp_edge_index`: the index of `edge` (IsEqual) among the edges of the FORWARD face,
/// -1 for a null edge, -2 when it is not there.
pub fn tp_edge_index(face: &Face, edge: &Shape) -> i32 {
    if edge.is_null() {
        return -1;
    }
    let forward = face.oriented(Orientation::Forward);
    let edges = Explorer::new(&forward, ShapeEnum::Edge, ShapeEnum::Shape).expect("face explorer");
    edges
        .enumerate()
        .find(|(_, candidate)| candidate.is_equal(edge))
        .map_or(-2, |(index, _)| index as i32)
}
