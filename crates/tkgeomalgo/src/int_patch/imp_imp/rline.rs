// Copyright (c) 1992-1999 Matra Datavision
// Copyright (c) 1999-2024 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ProcessRLine, IsRLineGood and SquareDistance of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The multiple points of the other lines put on the restriction lines, and the
//! restriction lines that overlap a conic removed.
use super::super::IntPatchIType;
use super::super::gline::IntPatchGLine;
use super::super::h_inter_tool::IntPatchHInterTool;
use super::super::line::IntPatchLine;
use super::super::point::IntPatchPoint;
use super::super::point_line::IntPatchPointLine;
use super::super::rline::IntPatchRLine;
use super::lines::VertexLine;
use crate::int_start::TheArc;
use crate::int_surf::{IntSurfQuadric, IntSurfTransition, IntSurfTypeTrans, make_transition};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomEllipse, GeomHyperbola, GeomParabola};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::extrema::ExtPC;
use rustycad_tkmath::gp::{Direction3, Point2, Point3, Vector2, Vector3};

/// `SquareDistance(theGLine, theP, theExtr)`: the squared distance of the point to the
/// line or circle, or the smallest extremum of the other conics (RealLast when the
/// extrema fail or find nothing: the lines do not overlap).
fn square_distance(gline: &IntPatchGLine, p: Point3, extr: &mut ExtPC<'_>) -> Result<f64, Failure> {
    match gline.arc_type() {
        IntPatchIType::Lin => Ok(gline.line()?.square_distance(p)),
        IntPatchIType::Circle => Ok(gline.circle()?.square_distance(p)),
        _ => {
            extr.perform(p)?;
            if !extr.is_done() || extr.nb_ext()? == 0 {
                return Ok(f64::MAX);
            }
            let mut sq_dist: f64 = extr.square_distance(1)?;
            for i in 2..=extr.nb_ext()? {
                sq_dist = sq_dist.min_native(extr.square_distance(i)?);
            }
            Ok(sq_dist)
        }
    }
}

/// `if (aSQD < aSQDist) aSQDist = aSQD;`.
trait MinNative {
    fn min_native(self, other: f64) -> f64;
}
impl MinNative for f64 {
    fn min_native(self, other: f64) -> f64 {
        if other < self { other } else { self }
    }
}

/// The Geom curve of an ellipse, parabola or hyperbola line, loaded over its bounds.
fn conic_curve(gline: &IntPatchGLine) -> Result<Option<Handle<dyn GeomCurve>>, Failure> {
    Ok(match gline.arc_type() {
        IntPatchIType::Ellipse => Some(Handle::new(GeomEllipse::from_elips(gline.ellipse()?)).cast()),
        IntPatchIType::Parabola => Some(Handle::new(GeomParabola::from_parab(gline.parabola()?)).cast()),
        IntPatchIType::Hyperbola => Some(Handle::new(GeomHyperbola::from_hypr(gline.hyperbola()?)).cast()),
        _ => None,
    })
}

/// The points of the restriction line checked by `IsRLineGood`: its polygon points, or
/// its vertices.
fn rline_value(rline: &IntPatchRLine, index: i32) -> Result<Point3, Failure> {
    if rline.has_polygon() {
        return Ok(rline.point(index)?.value());
    }
    Ok(rline.vertex(index)?.value())
}

/// `IsRLineGood(Quad1, Quad2, theGLine, theRLine, theTol)`: whether the restriction line
/// is not degenerated and leaves the conic somewhere (a two-point line is checked at
/// the middle of its arc, a longer one at its inner points).
fn is_rline_good(
    quads: (&IntSurfQuadric, &IntSurfQuadric),
    gline: &IntPatchGLine,
    rline: &IntPatchRLine,
    tol: f64,
) -> Result<bool, Failure> {
    let sq_tol: f64 = tol * tol;
    let nb_pnts_m1: i32 = if rline.has_polygon() { rline.nb_pnts()? } else { rline.nb_vertex() } - 1;
    if nb_pnts_m1 < 1 {
        return Ok(false);
    }
    let adaptor: Option<GeomAdaptorCurve> = match conic_curve(gline)? {
        Some(curve) => Some(GeomAdaptorCurve::with_range(&curve, curve.first_parameter(), curve.last_parameter())?),
        None => None,
    };
    let mut extr = ExtPC::new();
    if let Some(ac) = &adaptor {
        let ac: &dyn Adaptor3dCurve = ac;
        extr.initialize(ac, ac.first_parameter()?, ac.last_parameter()?, 1.0e-10)?;
    }
    if nb_pnts_m1 == 1 {
        let (p1, p2) = (rline_value(rline, 1)?, rline_value(rline, 2)?);
        if p1.square_distance(&p2) < sq_tol {
            // The restriction line is degenerated.
            return Ok(false);
        }
        let p_mid: Point3 = middle_of_arc(quads, rline)?;
        return Ok(square_distance(gline, p_mid, &mut extr)? > sq_tol);
    }
    for i in 2..=nb_pnts_m1 {
        if square_distance(gline, rline_value(rline, i)?, &mut extr)? > sq_tol {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The point of the quadric at the middle of the arc of the restriction line.
fn middle_of_arc((quad1, quad2): (&IntSurfQuadric, &IntSurfQuadric), rline: &IntPatchRLine) -> Result<Point3, Failure> {
    let (arc, quad) = if rline.is_arc_on_s1() {
        (rline.arc_on_s1(), quad1)
    } else {
        (rline.arc_on_s2(), quad2)
    };
    let (f, l) = (arc.first_parameter()?, arc.last_parameter()?);
    let p2d: Point2 = arc.value(0.5 * (f + l))?;
    Ok(quad.value(p2d.x(), p2d.y()))
}

/// The locals of `ProcessRLine` that keep their values from one line (or vertex) to the
/// next: the side and arc of the restriction, its range and the vertices.
#[derive(Default)]
struct RLineState {
    on_first: bool,
    arcref: TheArc,
    paramf: f64,
    paraml: f64,
    u: f64,
    v: f64,
    ptvtx: IntPatchPoint,
    newptvtx: IntPatchPoint,
    d1u: Vector3,
    d1v: Vector3,
    trest: IntSurfTransition,
    tarc: IntSurfTransition,
    seq_pnt3d: Vec<Point3>,
    seq_real: Vec<f64>,
}

/// `ProcessRLine(slin, Quad1, Quad2, _TolArc, theIsReqToKeepRLine)`: the vertices of the
/// other lines that are not on the domain of a restriction line are projected on it
/// (with `100 * TolArc`, capped at 0.1, as the projection tolerance) and added to it,
/// made multiple; unless asked to keep them, the restriction lines that do not leave a
/// conic are removed.
pub(super) fn process_rline(
    slin: &mut Vec<Handle<dyn IntPatchLine>>,
    quads: (&IntSurfQuadric, &IntSurfQuadric),
    tol_arc_in: f64,
    is_req_to_keep_rline: bool,
) -> Result<(), Failure> {
    let tol_arc: f64 = (100.0 * tol_arc_in).min_native(0.1);
    let mut st = RLineState::default();
    let mut i: usize = 0;
    while i < slin.len() {
        let slini = slin[i].clone();
        let mut has_to_delete = false;
        if slini.arc_type() == IntPatchIType::Restriction {
            let rline: Handle<IntPatchRLine> = slini.cast();
            (st.seq_pnt3d, st.seq_real) = (Vec::new(), Vec::new());
            has_to_delete = rline_against(&mut st, slin, &rline, quads, (tol_arc_in, tol_arc, is_req_to_keep_rline))?;
        }
        if has_to_delete {
            slin.remove(i);
            continue;
        }
        i += 1;
    }
    Ok(())
}

/// The restriction line against every other line; true when it must be deleted.
fn rline_against(
    st: &mut RLineState,
    slin: &[Handle<dyn IntPatchLine>],
    rline: &Handle<IntPatchRLine>,
    quads: (&IntSurfQuadric, &IntSurfQuadric),
    (tol_in, tol_arc, keep): (f64, f64, bool),
) -> Result<bool, Failure> {
    for slinj in slin {
        let nbpt: usize = st.seq_pnt3d.len(); // important that it is read here
        if slinj.arc_type() == IntPatchIType::Restriction {
            continue;
        }
        restriction_side(st, rline)?;
        let vline = VertexLine::analytic_or_conic(slinj)?;
        if !is_degenerated(st, quads)? {
            for k in 1..=vline.nb_vertex() {
                vertex_on_rline(st, slin, (&vline, k), rline, quads, (nbpt, tol_in, tol_arc))?;
            }
        }
        if keep {
            continue;
        }
        let gl: Handle<IntPatchGLine> = slinj.cast();
        if let Some(g) = gl.get() {
            if !is_rline_good(quads, g, rline, tol_arc)? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// The side, arc and range of the restriction line (kept from before when it has no
/// arc, or no first or last point, as natively).
fn restriction_side(st: &mut RLineState, rline: &IntPatchRLine) -> Result<(), Failure> {
    if rline.is_arc_on_s1() {
        st.on_first = true;
        st.arcref = rline.arc_on_s1();
    } else if rline.is_arc_on_s2() {
        st.arcref = rline.arc_on_s2();
        st.on_first = false;
    }
    st.paramf = if rline.has_first_point() { rline.first_point()?.parameter_on_line() } else { -f64::MAX };
    st.paraml = if rline.has_last_point() { rline.last_point()?.parameter_on_line() } else { f64::MAX };
    Ok(())
}

/// The quadric derivatives at the arc point of `prm`, with the arc derivative.
fn derivatives_at(st: &mut RLineState, quad: &IntSurfQuadric, arc: &TheArc, prm: f64) -> Result<Vector2, Failure> {
    let (p2d, d2d): (Point2, Vector2) = arc.d1(prm)?;
    if let Some((_, d1u, d1v)) = quad.d1(p2d.x(), p2d.y()) {
        (st.d1u, st.d1v) = (d1u, d1v);
    }
    Ok(d2d)
}

/// Whether the U derivative of the quadric is null at the 11 samples of the arc range.
fn is_degenerated(st: &mut RLineState, (quad1, quad2): (&IntSurfQuadric, &IntSurfQuadric)) -> Result<bool, Failure> {
    let quad = if st.on_first { quad1 } else { quad2 };
    let arc = st.arcref.clone();
    for edg in 0..=10 {
        let prm: f64 = st.paramf + (st.paraml - st.paramf) * f64::from(edg) * 0.1;
        derivatives_at(st, quad, &arc, prm)?;
        if st.d1u.magnitude() > 1e-7 {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The vertex `k` of the other line, when not on the domain of the restriction: an
/// extremity is processed by ProcessSegments.
fn vertex_on_rline(
    st: &mut RLineState,
    slin: &[Handle<dyn IntPatchLine>],
    (vline, k): (&VertexLine, i32),
    rline: &IntPatchRLine,
    quads: (&IntSurfQuadric, &IntSurfQuadric),
    (nbpt, tol_in, tol_arc): (usize, f64, f64),
) -> Result<(), Failure> {
    st.ptvtx = vline.vertex(k)?;
    let off = if st.on_first { !st.ptvtx.is_on_dom_s1() } else { !st.ptvtx.is_on_dom_s2() };
    if !off {
        return Ok(());
    }
    let toproj: Point3 = st.ptvtx.value();
    let known: Option<usize> = st.seq_pnt3d[..nbpt].iter().position(|p| toproj.distance(p) < tol_in);
    let keeppoint: bool = match known {
        Some(jj) => {
            st.newptvtx = st.ptvtx.clone();
            st.newptvtx.set_parameter(st.seq_real[jj]);
            true
        }
        None => project_on_rline(st, slin, quads, toproj, tol_arc)?,
    };
    if keeppoint {
        keep_vertex(st, (vline, k), rline, quads, tol_in)?;
    }
    Ok(())
}

/// Projects the vertex on the arc of the restriction: kept when the projection is
/// within `100 * tol_arc` inside the range, then added to every restriction line on
/// the same arc (a check that such a line carries the vertex).
fn project_on_rline(
    st: &mut RLineState,
    slin: &[Handle<dyn IntPatchLine>],
    (quad1, quad2): (&IntSurfQuadric, &IntSurfQuadric),
    toproj: Point3,
    tol_arc: f64,
) -> Result<bool, Failure> {
    (st.u, st.v) = if st.on_first { st.ptvtx.parameters_on_s1() } else { st.ptvtx.parameters_on_s2() };
    let Some((paramproj, p2d)) = IntPatchHInterTool::project(&st.arcref, Point2::new(st.u, st.v))? else {
        return Ok(false);
    };
    let quad = if st.on_first { quad1 } else { quad2 };
    let ptproj: Point3 = quad.value(p2d.x(), p2d.y());
    if !(toproj.distance(&ptproj) <= 100.0 * tol_arc && paramproj >= st.paramf && paramproj <= st.paraml) {
        return Ok(false);
    }
    st.newptvtx = st.ptvtx.clone();
    st.newptvtx.set_parameter(paramproj);
    st.seq_pnt3d.push(toproj);
    st.seq_real.push(paramproj);
    for slinri in slin {
        if slinri.arc_type() != IntPatchIType::Restriction {
            continue;
        }
        let r: Handle<IntPatchRLine> = slinri.cast();
        let same_arc = if st.on_first {
            r.is_arc_on_s1() && r.arc_on_s1().object_address() == st.arcref.object_address()
        } else {
            !st.on_first && r.is_arc_on_s2() && r.arc_on_s2().object_address() == st.arcref.object_address()
        };
        if same_arc {
            r.add_vertex(&st.newptvtx, false);
        }
    }
    Ok(true)
}

/// The kept vertex is made multiple on its line; its copy on the restriction gets the
/// transitions on the arc of the other domain, and is added to the restriction line.
fn keep_vertex(
    st: &mut RLineState,
    (vline, k): (&VertexLine, i32),
    rline: &IntPatchRLine,
    (quad1, quad2): (&IntSurfQuadric, &IntSurfQuadric),
    tol_in: f64,
) -> Result<(), Failure> {
    st.ptvtx.set_multiple(true);
    st.ptvtx.set_tolerance(tol_in);
    st.newptvtx.set_multiple(true);
    vline.replace(k, &st.ptvtx)?;
    if st.ptvtx.is_on_dom_s1() || st.ptvtx.is_on_dom_s2() {
        let arcref = st.arcref.clone();
        let prm: f64 = st.newptvtx.parameter_on_line();
        // On the first domain means on the domain of S2 here, and conversely.
        let (own, other) = if st.on_first { (quad1, quad2) } else { (quad2, quad1) };
        let d2d = derivatives_at(st, own, &arcref, prm)?;
        let mut tgrest = Vector3::default();
        tgrest.set_linear_form(d2d.x(), st.d1u, d2d.y(), st.d1v);
        let on_s1: bool = !st.on_first;
        let (arc, prm_arc) = if on_s1 {
            (st.ptvtx.arc_on_s1()?, st.ptvtx.parameter_on_arc1()?)
        } else {
            (st.ptvtx.arc_on_s2()?, st.ptvtx.parameter_on_arc2()?)
        };
        let d2d = derivatives_at(st, other, &arc, prm_arc)?;
        let mut tgarc = Vector3::default();
        tgarc.set_linear_form(d2d.x(), st.d1u, d2d.y(), st.d1v);
        let norm: Vector3 = st.d1u.crossed(st.d1v);
        if norm.square_magnitude() < 1e-16 {
            st.trest.set_type(true, IntSurfTypeTrans::Undecided);
            st.tarc.set_type(true, IntSurfTypeTrans::Undecided);
        } else {
            make_transition(tgrest, tgarc, Direction3::from_vector(norm)?, &mut st.trest, &mut st.tarc);
        }
        st.newptvtx.set_arc(on_s1, &arc, prm_arc, &st.trest, &st.tarc);
    }
    rline.add_vertex(&st.newptvtx, false);
    Ok(())
}
