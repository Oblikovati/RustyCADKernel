// Rust translation of the static Project functions of
// ModelingAlgorithms/TKFeat/LocOpe/LocOpe_WiresOnShape.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The projections of a vertex of a splitting wire on the edges of a face: on the 3D
//! curve of an edge, on its p-curve, and on the nearest edge of the face, the vertex
//! tolerance being enlarged to cover the 2D gap.
use crate::errors::null_object;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
use rustycad_tkgeomalgo::geom2d_api::ProjectPointOnCurve as ProjectPointOnCurve2d;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkshhealing::shape_analysis::adjust_to_period;

/// The 3D curve of `edge` with its location applied, and its range.
pub(super) fn located_curve(edge: &Edge) -> Result<(Handle<dyn GeomCurve>, f64, f64), Failure> {
    let located = BRepTool::curve_with_location(edge)?;
    let mut c = located.curve;
    if !located.location.is_identity() {
        let curve = c
            .get()
            .ok_or_else(|| null_object("LocOpe_WiresOnShape", "edge without 3D curve"))?;
        c = curve.transformed(located.location.transformation())?.cast();
    }
    Ok((c, located.first, located.last))
}

/// `Project(V, Edge)`: the parameter of the nearest projection of the vertex point on
/// the 3D curve of `edge` over its range; infinite without projection.
pub(super) fn project_on_edge(v: &Vertex, edge: &Edge) -> Result<f64, Failure> {
    let toproj = BRepTool::pnt(v)?;
    let (c, f, l) = located_curve(edge)?;
    let mut proj = ProjectPointOnCurve::new();
    proj.init_range(toproj, &c, f, l)?;
    if proj.nb_points() > 0 {
        return proj.lower_distance_parameter();
    }
    Ok(precision::infinite())
}

/// `Project(V, p2d, Edge, Face)`: the parameter of the nearest projection of `p2d` on
/// the p-curve of `edge` on `face`; infinite without projection.
pub(super) fn project_2d(p2d: Point2, edge: &Edge, face: &Face) -> Result<f64, Failure> {
    let pc = BRepTool::curve_on_face(edge, face)?;
    let mut proj = ProjectPointOnCurve2d::new();
    proj.init_range(p2d, &pc.pcurve, pc.first, pc.last)?;
    if proj.nb_points() > 0 {
        return proj.lower_distance_parameter();
    }
    Ok(precision::infinite())
}

/// The parameter and squared distance of the projection of the vertex on one edge of
/// the face, by its 3D curve or else by its p-curve; None to skip the edge.
fn edge_candidate(
    v: &Vertex,
    toproj: Point3,
    p2d: Point2,
    f: &Face,
    edg: &Edge,
    surf: &Handle<dyn GeomSurface>,
) -> Result<Option<(f64, f64)>, Failure> {
    let (c, _, _) = BRepTool::curve(edg)?;
    if let Some(curve) = c.get() {
        let par = project_on_edge(v, edg)?;
        if precision::is_infinite(par) {
            return Ok(None);
        }
        return Ok(Some((par, curve.d0(par)?.square_distance(&toproj))));
    }
    if precision::is_infinite(p2d.x()) {
        return Ok(Some((precision::infinite(), precision::infinite())));
    }
    // Geom2dAPI_ProjectPointOnCurve proj;
    let pc = BRepTool::curve_on_face(edg, f)?;
    let Some(pcurve) = pc.pcurve.get() else {
        return Ok(None);
    };
    let par = project_2d(p2d, edg, f)?;
    if precision::is_infinite(par) {
        return Ok(None);
    }
    let pproj = pcurve.d0(par)?;
    let surface = surf
        .get()
        .ok_or_else(|| null_object("LocOpe_WiresOnShape::Project", "face without surface"))?;
    Ok(Some((
        par,
        surface.d0(pproj.x(), pproj.y())?.square_distance(&toproj),
    )))
}

/// The nearest edge of the FORWARD face to the vertex, its parameter and squared
/// distance, when one is nearer than `dmin`.
fn nearest_edge(
    v: &Vertex,
    p2d: Point2,
    f: &Face,
    surf: &Handle<dyn GeomSurface>,
    dmin: f64,
) -> Result<Option<(Edge, f64, f64)>, Failure> {
    let toproj = BRepTool::pnt(v)?;
    let mut best: Option<(Edge, f64, f64)> = None;
    let mut dmin = dmin;
    for e in Explorer::new(
        &f.oriented(Orientation::Forward),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )? {
        let edg = topods::edge(&e)?;
        let Some((par, dist)) = edge_candidate(v, toproj, p2d, f, &edg, surf)? else {
            continue;
        };
        if dist < dmin {
            dmin = dist;
            best = Some((edg, par, dist));
        }
    }
    Ok(best)
}

/// A periodic coordinate of the edge point and of `p2d` brought into the surface
/// period: the adjusted bound and point coordinates and their gap (zero when the gap is
/// one period).
fn periodic_gap(
    bound: f64,
    point: f64,
    first: f64,
    last: f64,
    period: f64,
    tol: f64,
    dmax: f64,
) -> (f64, f64, f64) {
    let x1 = bound + adjust_to_period(bound, first, last);
    let mut x2 = point + adjust_to_period(point, first, last);
    let mut gap = (x2 - x1).abs();
    if gap > dmax && (gap - period).abs() < tol {
        x2 = x1;
        gap = 0.;
    }
    (x1, x2, gap)
}

/// The 2D gap between `p2d` and the end of the projection on the edge (`pbound2d`),
/// adjusted on the periodic directions, and the bound point updated.
fn gap_2d(
    ad_surf: &GeomAdaptorSurface,
    p2d: Point2,
    pbound2d: &mut Point2,
) -> Result<(f64, f64, f64, f64), Failure> {
    let (u1, u2) = (ad_surf.first_u_parameter()?, ad_surf.last_u_parameter()?);
    let (v1, v2) = (ad_surf.first_v_parameter()?, ad_surf.last_v_parameter()?);
    let dumax = 0.01 * (u2 - u1);
    let dvmax = 0.01 * (v2 - v1);
    let mut dumin = (p2d.x() - pbound2d.x()).abs();
    let mut dvmin = (p2d.y() - pbound2d.y()).abs();
    if dumin > dumax && ad_surf.is_u_periodic()? {
        let (x1, _, gap) = periodic_gap(
            pbound2d.x(),
            p2d.x(),
            u1,
            u2,
            ad_surf.u_period()?,
            precision::p_confusion(),
            dumax,
        );
        pbound2d.set_x(x1);
        dumin = gap;
    }
    if dvmin > dvmax && ad_surf.is_v_periodic()? {
        let (y1, _, gap) = periodic_gap(
            pbound2d.y(),
            p2d.y(),
            v1,
            v2,
            ad_surf.v_period()?,
            precision::confusion(),
            dvmax,
        );
        pbound2d.set_y(y1);
        dvmin = gap;
    }
    Ok((dumin, dvmin, dumax, dvmax))
}

/// Enlarges the vertex tolerance to cover the 2D gap between `p2d` and the point of
/// parameter `param` on the p-curve of `edge` (recomputed in 3D, as BRepCheck_Wire
/// checks it).
fn cover_2d_gap(
    v: &Vertex,
    p2d: Point2,
    f: &Face,
    edge: &Edge,
    param: f64,
    surf: &Handle<dyn GeomSurface>,
) -> Result<(), Failure> {
    let atol_v = BRepTool::vertex_tolerance(v)?;
    let ad_surf = GeomAdaptorSurface::with_surface(surf)?;
    let u_resolution = ad_surf.u_resolution(1.)?;
    // Native quirk kept: the V resolution is the U one.
    let v_resolution = ad_surf.u_resolution(1.)?;
    let pc = BRepTool::curve_on_face(edge, f)?;
    let Some(crv_bound) = pc.pcurve.get() else {
        return Ok(());
    };
    let mut pbound2d = crv_bound.d0(param)?;
    // distance in 2D space recomputed in the 3D space in order to tolerance of vertex
    // cover gap in 2D space. For consistency with check of the validity in the  BRepCheck_Wire
    let (dumin, dvmin, dumax, dvmax) = gap_2d(&ad_surf, p2d, &mut pbound2d)?;
    let mut dist3d = atol_v;
    if dumin > dumax || dvmin > dvmax {
        let dumax = ad_surf.u_resolution(atol_v)?;
        let dvmax = ad_surf.v_resolution(atol_v)?;
        let tol2d = 2. * dumax.max(dvmax);
        let dist2d = dumin.max(dvmin);
        if dist2d > tol2d {
            dist3d = dist3d.max(dist2d / u_resolution.max(v_resolution));
        }
    }
    // added check by 3D the same as in the BRepCheck_Wire::SelfIntersect
    let surface = surf
        .get()
        .ok_or_else(|| null_object("LocOpe_WiresOnShape::Project", "face without surface"))?;
    let pbound = surface.d0(pbound2d.x(), pbound2d.y())?;
    let pv2d = surface.d0(p2d.x(), p2d.y())?;
    let max_dist = pv2d.square_distance(&pbound).max(dist3d * dist3d);
    if atol_v * atol_v < max_dist {
        BRepBuilder::default().update_vertex_tolerance(v, max_dist.sqrt())?;
    }
    Ok(())
}

/// `Project(V, p2d, F, theEdge, param)`: without `the_edge`, the nearest edge of `f`
/// to the vertex (by its 3D curve, or its p-curve from `p2d`); with it, its parameter
/// when `param` is infinite. True when the vertex lies within the sum of the
/// tolerances; the vertex tolerance then grows to cover the 2D gap.
pub(super) fn project_on_face(
    v: &Vertex,
    p2d: Point2,
    f: &Face,
    the_edge: &mut Edge,
    param: &mut f64,
) -> Result<bool, Failure> {
    let atol_v = BRepTool::vertex_tolerance(v)?;
    let mut dmin = if the_edge.is_null() {
        f64::MAX
    } else {
        atol_v * atol_v
    };
    let surf = BRepTool::surface(f)?;
    if the_edge.is_null() {
        let Some((edge, par, dist)) = nearest_edge(v, p2d, f, &surf, dmin)? else {
            return Ok(false);
        };
        (*the_edge, *param, dmin) = (edge, par, dist);
    } else if precision::is_infinite(*param) {
        let (c, _, _) = BRepTool::curve(the_edge)?;
        *param = if c.is_null() {
            project_2d(p2d, the_edge, f)?
        } else {
            project_on_edge(v, the_edge)?
        };
    }
    let ttol = atol_v + BRepTool::edge_tolerance(the_edge)?;
    if dmin > ttol * ttol {
        return Ok(false);
    }
    cover_2d_gap(v, p2d, f, the_edge, *param, &surf)?;
    Ok(true)
}
