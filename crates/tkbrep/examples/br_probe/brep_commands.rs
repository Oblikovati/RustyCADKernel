//! The BRep_Builder commands of a scene, the Rust twin of the command part of
//! tools/oracle/tkbrep_probe_brep.cxx (the `br_*_command` functions).
use super::brep::{Scene, at, point, point2d};
use super::common::{Reader, failure, word};
use super::graph;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point2;
/// Commands 1-9 and 47: new shapes.
fn make_command(s: &mut Scene, r: &mut Reader<'_>, code: i32) -> Result<bool, Failure> {
    let b = BRepBuilder::default();
    let g = &s.geometry;
    match code {
        1 => {
            let mut v = Vertex::default();
            let p = point(r);
            b.make_vertex_at(&mut v, p, r.real())?;
            s.push(v);
        }
        2 => {
            let mut v = Vertex::default();
            b.make_vertex(&mut v);
            s.push(v);
        }
        3 => {
            let mut e = Edge::default();
            b.make_edge(&mut e)?;
            s.push(e);
        }
        4 => {
            let mut e = Edge::default();
            let (c, l) = (at(&g.curves, r.integer()), s.loc(r));
            b.make_edge_with_located_curve(&mut e, &c, &l, r.real())?;
            s.push(e);
        }
        5 => {
            let mut f = Face::default();
            let (c, l) = (at(&g.surfaces, r.integer()), s.loc(r));
            b.make_face_on_located_surface(&mut f, &c, &l, r.real())?;
            s.push(f);
        }
        6 => {
            let mut f = Face::default();
            b.make_face(&mut f);
            s.push(f);
        }
        7 => {
            let mut f = Face::default();
            b.make_face_with_triangulation(&mut f, &at(&g.triangulations, r.integer()))?;
            s.push(f);
        }
        8 => {
            let mut e = Edge::default();
            b.make_edge_with_polygon3d(&mut e, &at(&g.polygons3d, r.integer()))?;
            s.push(e);
        }
        9 => {
            let mut e = Edge::default();
            let (n, t) = (
                at(&g.nodes, r.integer()),
                at(&g.triangulations, r.integer()),
            );
            let l = s.loc(r);
            b.make_edge_on_triangulation(&mut e, &n, &t, &l)?;
            s.push(e);
        }
        47 => {
            let mut f = Face::default();
            let list: Vec<_> = (0..r.integer())
                .map(|_| at(&g.triangulations, r.integer()))
                .collect();
            b.make_face_with_triangulations(&mut f, &list, &at(&g.triangulations, r.integer()))?;
            s.push(f);
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// `GeomAbs_Shape(i)`.
pub fn continuity(i: i32) -> Continuity {
    use Continuity::*;
    [C0, G1, C1, G2, C2, C3, CN][i as usize]
}

/// The optional UV points of commands 11 and 12.
fn uv(r: &mut Reader<'_>) -> Option<(Point2, Point2)> {
    if !r.flag() {
        return None;
    }
    let p1 = point2d(r);
    Some((p1, point2d(r)))
}

/// Commands 10-22: edge geometry.
fn edge_command(s: &Scene, r: &mut Reader<'_>, code: i32) -> Result<bool, Failure> {
    let b = BRepBuilder::default();
    let g = &s.geometry;
    match code {
        10 => {
            let e = s.e(r)?;
            let (c, l) = (at(&g.curves, r.integer()), s.loc(r));
            b.update_edge_located_curve(&e, &c, &l, r.real())?;
        }
        11 | 12 => {
            let e = s.e(r)?;
            let c1 = at(&g.pcurves, r.integer());
            let c2 = if code == 12 {
                at(&g.pcurves, r.integer())
            } else {
                Handle::default()
            };
            let (sf, l, tol) = (at(&g.surfaces, r.integer()), s.loc(r), r.real());
            let uv = uv(r);
            if code == 11 {
                b.update_edge_pcurve(&e, &c1, &sf, &l, tol, uv)?;
            } else {
                b.update_edge_pcurves(&e, (&c1, &c2), &sf, &l, tol, uv)?;
            }
        }
        13 => {
            let (e, c) = (s.e(r)?, at(&g.pcurves, r.integer()));
            let f = s.f(r)?;
            b.update_edge_pcurve_on_face(&e, &c, &f, r.real())?;
        }
        14 => {
            let (e, c1, c2) = (
                s.e(r)?,
                at(&g.pcurves, r.integer()),
                at(&g.pcurves, r.integer()),
            );
            let f = s.f(r)?;
            b.update_edge_pcurves_on_face(&e, (&c1, &c2), &f, r.real())?;
        }
        15 => {
            let (e, p) = (s.e(r)?, at(&g.polygons3d, r.integer()));
            b.update_edge_polygon3d(&e, &p, &s.loc(r))?;
        }
        16 => {
            let (e, n, t) = (
                s.e(r)?,
                at(&g.nodes, r.integer()),
                at(&g.triangulations, r.integer()),
            );
            b.update_edge_polygon_on_triangulation(&e, &n, &t, &s.loc(r))?;
        }
        17 => {
            let (e, n1, n2) = (
                s.e(r)?,
                at(&g.nodes, r.integer()),
                at(&g.nodes, r.integer()),
            );
            let t = at(&g.triangulations, r.integer());
            b.update_edge_polygons_on_triangulation(&e, (&n1, &n2), &t, &s.loc(r))?;
        }
        _ => return edge_polygon_command(s, r, code),
    }
    Ok(true)
}

/// Commands 18-22: edge 2D polygons and tolerance.
fn edge_polygon_command(s: &Scene, r: &mut Reader<'_>, code: i32) -> Result<bool, Failure> {
    let b = BRepBuilder::default();
    let g = &s.geometry;
    match code {
        18 => {
            let (e, p, sf) = (
                s.e(r)?,
                at(&g.polygons2d, r.integer()),
                at(&g.surfaces, r.integer()),
            );
            b.update_edge_polygon_on_surface(&e, &p, &sf, &s.loc(r))?;
        }
        19 => {
            let (e, p1, p2) = (
                s.e(r)?,
                at(&g.polygons2d, r.integer()),
                at(&g.polygons2d, r.integer()),
            );
            let sf = at(&g.surfaces, r.integer());
            b.update_edge_polygons_on_surface(&e, (&p1, &p2), &sf, &s.loc(r))?;
        }
        20 => {
            let (e, p) = (s.e(r)?, at(&g.polygons2d, r.integer()));
            b.update_edge_polygon_on_face(&e, &p, &s.f(r)?)?;
        }
        21 => {
            let (e, p1, p2) = (
                s.e(r)?,
                at(&g.polygons2d, r.integer()),
                at(&g.polygons2d, r.integer()),
            );
            b.update_edge_polygons_on_face(&e, (&p1, &p2), &s.f(r)?)?;
        }
        22 => {
            let e = s.e(r)?;
            b.update_edge_tolerance(&e, r.real())?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// Commands 23-32: edge flags, ranges and transfers.
fn flag_command(s: &Scene, r: &mut Reader<'_>, code: i32) -> Result<bool, Failure> {
    let b = BRepBuilder::default();
    let g = &s.geometry;
    match code {
        23 => {
            let (e, s1, s2) = (
                s.e(r)?,
                at(&g.surfaces, r.integer()),
                at(&g.surfaces, r.integer()),
            );
            let (l1, l2) = (s.loc(r), s.loc(r));
            let c = continuity(r.integer());
            b.continuity(&e, (&s1, &s2), (&l1, &l2), c)?;
        }
        24 => {
            let (e, f1, f2) = (s.e(r)?, s.f(r)?, s.f(r)?);
            b.continuity_on_faces(&e, &f1, &f2, continuity(r.integer()))?;
        }
        25 => b.same_parameter(&s.e(r)?, r.flag())?,
        26 => b.same_range(&s.e(r)?, r.flag())?,
        27 => b.degenerated(&s.e(r)?, r.flag())?,
        28 => {
            let (e, f, l) = (s.e(r)?, r.real(), r.real());
            b.range(&e, f, l, r.flag())?;
        }
        29 => {
            let (e, sf, lo, f) = (s.e(r)?, at(&g.surfaces, r.integer()), s.loc(r), r.real());
            b.range_on_surface(&e, &sf, &lo, f, r.real())?;
        }
        30 => {
            let (e, fc, f) = (s.e(r)?, s.f(r)?, r.real());
            b.range_on_face(&e, &fc, f, r.real())?;
        }
        31 => {
            let e1 = s.e(r)?;
            b.transfert(&e1, &s.e(r)?)?;
        }
        32 => {
            let (e1, e2, v1) = (s.e(r)?, s.e(r)?, s.v(r)?);
            b.transfert_vertex(&e1, &e2, &v1, &s.v(r)?)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// Commands 33-38: vertices.
fn vertex_command(s: &Scene, r: &mut Reader<'_>, code: i32) -> Result<bool, Failure> {
    let b = BRepBuilder::default();
    let g = &s.geometry;
    match code {
        33 => {
            let (v, p) = (s.v(r)?, point(r));
            b.update_vertex_point(&v, p, r.real())?;
        }
        34 => {
            let (v, p, e) = (s.v(r)?, r.real(), s.e(r)?);
            b.update_vertex_on_edge(&v, p, &e, r.real())?;
        }
        35 => {
            let (v, p, e, f) = (s.v(r)?, r.real(), s.e(r)?, s.f(r)?);
            b.update_vertex_on_edge_face(&v, p, &e, &f, r.real())?;
        }
        36 => {
            let (v, p, e) = (s.v(r)?, r.real(), s.e(r)?);
            let (sf, l) = (at(&g.surfaces, r.integer()), s.loc(r));
            b.update_vertex_on_edge_surface(&v, p, &e, &sf, &l, r.real())?;
        }
        37 => {
            let (v, u, w, f) = (s.v(r)?, r.real(), r.real(), s.f(r)?);
            b.update_vertex_on_face(&v, u, w, &f, r.real())?;
        }
        38 => b.update_vertex_tolerance(&s.v(r)?, r.real())?,
        _ => return face_command(s, r, code),
    }
    Ok(true)
}

/// Commands 39-42, 49 and 50: faces and UV points.
fn face_command(s: &Scene, r: &mut Reader<'_>, code: i32) -> Result<bool, Failure> {
    let b = BRepBuilder::default();
    let g = &s.geometry;
    match code {
        39 => {
            let (f, sf, l) = (s.f(r)?, at(&g.surfaces, r.integer()), s.loc(r));
            b.update_face(&f, &sf, &l, r.real())?;
        }
        40 => {
            let (f, t) = (s.f(r)?, at(&g.triangulations, r.integer()));
            b.update_face_triangulation(&f, &t, r.flag())?;
        }
        41 => b.update_face_tolerance(&s.f(r)?, r.real())?,
        42 => b.natural_restriction(&s.f(r)?, r.flag())?,
        49 => {
            let (e, sf, l, p1) = (s.e(r)?, at(&g.surfaces, r.integer()), s.loc(r), point2d(r));
            BRepTool::set_uv_points(&e, &sf, &l, p1, point2d(r))?;
        }
        50 => {
            let (e, f, p1) = (s.e(r)?, s.f(r)?, point2d(r));
            BRepTool::set_uv_points_on_face(&e, &f, p1, point2d(r))?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// `br_brep_command`: a BRep command, or a graph command for codes from 100.
pub fn command(s: &mut Scene, r: &mut Reader<'_>) {
    let code = r.integer();
    if code >= 100 {
        match graph::apply(&mut s.graph, r, code - 100) {
            Ok(()) => word("-"),
            Err(e) => failure(&e),
        }
        return;
    }
    let known = make_command(s, r, code).and_then(|done| {
        if done {
            return Ok(true);
        }
        let s: &Scene = s;
        Ok(edge_command(s, r, code)? || flag_command(s, r, code)? || vertex_command(s, r, code)?)
    });
    match known {
        Ok(known) => word(if known { "-" } else { "?" }),
        Err(e) => failure(&e),
    }
}
