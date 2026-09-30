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
// Rust translation of FindLine and SingleLine of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The line of the result that passes through a point on a restriction, its
//! parameters and its tangent there.
use super::super::IntPatchIType;
use super::super::aline::IntPatchALine;
use super::super::gline::IntPatchGLine;
use super::super::line::IntPatchLine;
use super::arc::intersection_with_an_arc;
use crate::int_start::TheArc;
use crate::int_surf::IntSurfQuadric;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Point3, Vector3};
use std::f64::consts::PI;

/// The outputs of `FindLine` beside the found line and the moved point: the parameters
/// on the line (`theLParams`), the tangent (`Vtgtint`), the restriction line to search
/// (`OnlyThisLine`, 0 for all), the point on the arc and the output tolerance
/// (`theOutputToler`, in/out).
pub(super) struct FindLineArgs {
    pub(super) lparams: Vec<f64>,
    pub(super) vtgint: Vector3,
    pub(super) only_this_line: i32,
    pub(super) point_on_arc: Point3,
    pub(super) vert_tol: f64,
}

impl FindLineArgs {
    /// The arguments of one call: `theOutputToler = vert_tol`, the tangent null.
    pub(super) fn new(vert_tol: f64, only_this_line: i32) -> Self {
        Self {
            lparams: Vec::new(),
            vtgint: Vector3::new(0.0, 0.0, 0.0),
            only_this_line,
            point_on_arc: Point3::default(),
            vert_tol,
        }
    }
}

/// The analytic or conic line of the handle (the null cast of another line is the
/// Standard_NullObject the native dereference would crash on).
fn gline(line: &Handle<dyn IntPatchLine>) -> Result<Handle<IntPatchGLine>, Failure> {
    let g: Handle<IntPatchGLine> = line.cast();
    if g.is_null() {
        return Err(super::super::failures::null_object("FindLine", "IntPatch_GLine"));
    }
    Ok(g)
}

fn aline(line: &Handle<dyn IntPatchLine>) -> Result<Handle<IntPatchALine>, Failure> {
    let a: Handle<IntPatchALine> = line.cast();
    if a.is_null() {
        return Err(super::super::failures::null_object("FindLine", "IntPatch_ALine"));
    }
    Ok(a)
}

/// The best candidate of the search.
struct Best {
    sq_dist_min: f64,
    para_int: f64,
    line_idx: i32,
}

impl Best {
    /// A candidate of a conic: kept when nearer than the tolerance and than the best.
    fn offer(&mut self, sq_dist: f64, sq_tol: f64, para: f64, i: i32) {
        if sq_dist < sq_tol && sq_dist < self.sq_dist_min {
            self.sq_dist_min = sq_dist;
            self.para_int = para;
            self.line_idx = i;
        }
    }
}

/// The parameter range of the line: the analytic domain, or the first and last
/// vertices of a conic (RealFirst/RealLast when there is none).
fn line_range(line: &Handle<dyn IntPatchLine>) -> Result<(f64, f64), Failure> {
    if line.arc_type() == IntPatchIType::Analytic {
        let a = aline(line)?;
        return Ok((a.first_parameter()?.0, a.last_parameter()?.0));
    }
    let g = gline(line)?;
    let lower: f64 = if g.has_first_point() { g.first_point()?.parameter_on_line() } else { -f64::MAX };
    let upper: f64 = if g.has_last_point() { g.last_point()?.parameter_on_line() } else { f64::MAX };
    Ok((lower, upper))
}

/// Whether `para` (or it plus or minus a turn) is within the range.
fn in_range_periodic(para: f64, (lower, upper): (f64, f64)) -> bool {
    (para <= upper && para >= lower)
        || (para + 2.0 * PI <= upper && para + 2.0 * PI >= lower)
        || (para - 2.0 * PI <= upper && para - 2.0 * PI >= lower)
}

/// The parameters of the points passed to `FindLine`.
struct Target<'a> {
    psurf: &'a mut Point3,
    sq_tol: f64,
    arc: &'a TheArc,
    param_on_arc: &'a mut f64,
    quad1: &'a IntSurfQuadric,
}

/// `FindLine(Psurf, slin, Tol, theLParams, Vtgtint, theLineIdx, OnlyThisLine, thearc,
/// theparameteronarc, thepointonarc, QuadSurf1, QuadSurf2, theOutputToler)`: the line
/// (1-based index) nearest to the point on both quadrics within `tol`. The conics are
/// projected on, the analytic lines searched by `FindParameter`, else intersected with
/// the arc in 2d (which may move the point and the arc parameter). The parameters of an
/// accepted analytic line replace the list, those of the arc search and of the chosen
/// conic are appended to it, as natively.
pub(super) fn find_line(
    psurf: &mut Point3,
    slin: &[Handle<dyn IntPatchLine>],
    tol: f64,
    args: &mut FindLineArgs,
    (thearc, param_on_arc): (&TheArc, &mut f64),
    (quad1, quad2): (&IntSurfQuadric, &IntSurfQuadric),
) -> Result<Option<i32>, Failure> {
    if quad1.distance(*psurf) > tol || quad2.distance(*psurf) > tol {
        return Ok(None);
    }
    let mut best = Best { sq_dist_min: f64::MAX, para_int: f64::MAX, line_idx: 0 };
    let mut target = Target { psurf, sq_tol: tol * tol, arc: thearc, param_on_arc, quad1 };
    let mut i: i32 = 1;
    let mut nblin: i32 = slin.len() as i32;
    while i <= nblin {
        if args.only_this_line != 0 {
            i = args.only_this_line;
            nblin = 0;
        }
        let line = sequence_line(slin, i)?;
        search_line(line, i, &mut target, &mut best, args)?;
        i += 1;
    }
    if best.sq_dist_min == f64::MAX {
        return Ok(None);
    }
    args.vert_tol = std_max(args.vert_tol, best.sq_dist_min.sqrt());
    tangent_of(sequence_line(slin, best.line_idx)?, best.para_int, args)?;
    Ok(Some(best.line_idx))
}

fn sequence_line(slin: &[Handle<dyn IntPatchLine>], i: i32) -> Result<&Handle<dyn IntPatchLine>, Failure> {
    crate::int_start::failures::sequence_value(slin, i, "NCollection_Sequence::Value")
}

/// The candidate of the line `i`.
fn search_line(
    line: &Handle<dyn IntPatchLine>,
    i: i32,
    target: &mut Target<'_>,
    best: &mut Best,
    args: &mut FindLineArgs,
) -> Result<(), Failure> {
    let (lower, upper) = line_range(line)?;
    let p: Point3 = *target.psurf;
    let sq_tol: f64 = target.sq_tol;
    match line.arc_type() {
        IntPatchIType::Lin => {
            let lin = gline(line)?.line()?;
            let para: f64 = elclib::parameter(&lin, p)?;
            if para <= upper && para >= lower {
                let pt: Point3 = elclib::value(para, &lin);
                best.offer(p.square_distance(&pt), sq_tol, para, i);
            }
        }
        IntPatchIType::Circle => {
            let c = gline(line)?.circle()?;
            let para: f64 = elclib::parameter(&c, p)?;
            if in_range_periodic(para, (lower, upper)) {
                best.offer(p.square_distance(&elclib::value(para, &c)), sq_tol, para, i);
            }
        }
        IntPatchIType::Ellipse => {
            let e = gline(line)?.ellipse()?;
            let para: f64 = elclib::parameter(&e, p)?;
            if in_range_periodic(para, (lower, upper)) {
                best.offer(p.square_distance(&elclib::value(para, &e)), sq_tol, para, i);
            }
        }
        IntPatchIType::Parabola => search_parabola(line, i, (lower, upper), target, best)?,
        IntPatchIType::Hyperbola => {
            let h = gline(line)?.hyperbola()?;
            let para: f64 = elclib::parameter(&h, p)?;
            if para <= upper && para >= lower {
                best.offer(p.square_distance(&elclib::value(para, &h)), sq_tol, para, i);
            }
        }
        IntPatchIType::Analytic => search_analytic(line, i, (lower, upper), target, best, args)?,
        IntPatchIType::Walking | IntPatchIType::Restriction => {}
    }
    Ok(())
}

/// The parabola: ElCLib does not choose the best axis for the parameter, so a very
/// sharp parabola (focal near 1e-2) at a great parameter goes wrong; up to 5 secant
/// steps improve it, each candidate being offered.
fn search_parabola(
    line: &Handle<dyn IntPatchLine>,
    i: i32,
    (lower, upper): (f64, f64),
    target: &mut Target<'_>,
    best: &mut Best,
) -> Result<(), Failure> {
    let parab = gline(line)?.parabola()?;
    let p: Point3 = *target.psurf;
    let mut para: f64 = elclib::parameter(&parab, p)?;
    if !(para <= upper && para >= lower) {
        return Ok(());
    }
    let mut amelioration: i32 = 0;
    loop {
        let parabis: f64 = para + 0.0000001;
        let pt: Point3 = elclib::value(para, &parab);
        let sq_dist: f64 = p.square_distance(&pt);
        let distbis: f64 = p.distance(&elclib::value(parabis, &parab));
        let dist: f64 = sq_dist.sqrt();
        let ddist: f64 = distbis - dist;
        best.offer(sq_dist, target.sq_tol, para, i);
        if sq_dist < precision::square_p_confusion() {
            amelioration = 100;
        }
        if ddist > 1.0e-9 || ddist < -1.0e-9 {
            para -= dist * (parabis - para) / ddist;
        } else {
            amelioration = 100;
        }
        amelioration += 1;
        if amelioration >= 5 {
            break;
        }
    }
    Ok(())
}

/// The analytic line: the nearest of the parameters of the projection (already within
/// the internal tolerance of `FindParameter`), else the 2d intersection with the arc.
fn search_analytic(
    line: &Handle<dyn IntPatchLine>,
    i: i32,
    range: (f64, f64),
    target: &mut Target<'_>,
    best: &mut Best,
    args: &mut FindLineArgs,
) -> Result<(), Failure> {
    let alin = aline(line)?;
    let p: Point3 = *target.psurf;
    let mut params: List<f64> = List::default();
    alin.find_parameter(p, &mut params)?;
    if !params.is_empty() {
        let mut sq_dist: f64 = f64::MAX;
        for &prm in params.iter() {
            let sq_d: f64 = p.square_distance(&alin.value(prm)?);
            if sq_d < sq_dist {
                sq_dist = sq_d;
            }
        }
        if sq_dist < best.sq_dist_min {
            best.sq_dist_min = sq_dist;
            args.lparams = params.iter().copied().collect();
            best.line_idx = i;
        }
        return Ok(());
    }
    // The point was not found by projection: try the 2d intersection with the arc.
    let mut paramonarc: f64 = *target.param_on_arc;
    let mut copie: Point3 = p;
    let mut para: f64 = 0.0;
    let ok = intersection_with_an_arc(
        &mut copie,
        &alin,
        &mut para,
        target.arc,
        &mut paramonarc,
        &mut args.point_on_arc,
        target.quad1,
        range,
    )?;
    let sq_dist: f64 = copie.square_distance(&p);
    if ok && sq_dist < target.sq_tol {
        *target.param_on_arc = paramonarc;
        *target.psurf = args.point_on_arc;
        best.sq_dist_min = sq_dist;
        args.lparams.push(para);
        best.line_idx = i;
    }
    Ok(())
}

/// The tangent of the chosen line at its parameter (a conic appends the parameter of
/// its projection); a singular analytic point gives a null tangent (#29807: the
/// singularities are handled by IntPatch_SpecialPoints).
fn tangent_of(line: &Handle<dyn IntPatchLine>, para_int: f64, args: &mut FindLineArgs) -> Result<(), Failure> {
    let conic_tangent: Option<Vector3> = match line.arc_type() {
        IntPatchIType::Lin => Some(Vector3::from(gline(line)?.line()?.direction())),
        IntPatchIType::Circle => Some(elclib::dn(para_int, &gline(line)?.circle()?, 1)),
        IntPatchIType::Ellipse => Some(elclib::dn(para_int, &gline(line)?.ellipse()?, 1)),
        IntPatchIType::Parabola => Some(elclib::dn(para_int, &gline(line)?.parabola()?, 1)),
        IntPatchIType::Hyperbola => Some(elclib::dn(para_int, &gline(line)?.hyperbola()?, 1)),
        _ => None,
    };
    if let Some(tangent) = conic_tangent {
        args.lparams.push(para_int);
        args.vtgint = tangent;
        return Ok(());
    }
    if line.arc_type() != IntPatchIType::Analytic {
        return Ok(());
    }
    let last: f64 = *args.lparams.last().ok_or_else(|| {
        Failure::with_message("Standard_NoSuchObject", "FindLine() - the list of parameters is empty; expected the parameters of the analytic line")
    })?;
    let (mut pt, mut tgt) = (Point3::default(), args.vtgint);
    if !aline(line)?.d1(last, &mut pt, &mut tgt)? {
        tgt = Vector3::new(0.0, 0.0, 0.0);
    }
    args.vtgint = tgt;
    Ok(())
}

/// `SingleLine(Psurf, lin, Tol, Paraint, Vtgtint)`: the projection of the point on the
/// line (the last parameter of `FindParameter` on an analytic line), its parameter and
/// tangent, when it is within `tol` of the point; `None` when an analytic line has no
/// parameter or the projection is farther.
pub(super) fn single_line(
    psurf: Point3,
    lin: &Handle<dyn IntPatchLine>,
    tol: f64,
) -> Result<Option<(f64, Vector3)>, Failure> {
    let (parproj, ptproj, tgint) = match lin.arc_type() {
        IntPatchIType::Lin => project(&gline(lin)?.line()?, psurf)?,
        IntPatchIType::Circle => project(&gline(lin)?.circle()?, psurf)?,
        IntPatchIType::Ellipse => project(&gline(lin)?.ellipse()?, psurf)?,
        IntPatchIType::Parabola => project(&gline(lin)?.parabola()?, psurf)?,
        IntPatchIType::Hyperbola => project(&gline(lin)?.hyperbola()?, psurf)?,
        IntPatchIType::Analytic => {
            let alin = aline(lin)?;
            let mut params: List<f64> = List::default();
            alin.find_parameter(psurf, &mut params)?;
            let Some(&parproj) = params.iter().last() else {
                return Ok(None);
            };
            let (mut ptemp, mut tgint) = (Point3::default(), Vector3::default());
            if !alin.d1(parproj, &mut ptemp, &mut tgint)? {
                tgint = Vector3::new(0.0, 0.0, 0.0);
            }
            (parproj, psurf, tgint)
        }
        IntPatchIType::Walking | IntPatchIType::Restriction => (0.0, Point3::default(), Vector3::default()),
    };
    if psurf.distance(&ptproj) <= tol {
        return Ok(Some((parproj, tgint)));
    }
    Ok(None)
}

/// The projection of `p` on an elementary curve: its parameter, point and tangent.
fn project<C>(curve: &C, p: Point3) -> Result<(f64, Point3, Vector3), Failure>
where
    C: elclib::ElementaryCurve<Point = Point3, Vector = Vector3>,
{
    let parproj: f64 = elclib::parameter(curve, p)?;
    let (ptproj, tgint) = elclib::d1(parproj, curve);
    Ok((parproj, ptproj, tgint))
}
