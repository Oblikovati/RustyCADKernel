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
// Rust translation of AddPointIntoWL and SeekAdditionalPoints of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The points of the walking lines of two cylinders.
use super::coeffs::{CoeffsValue, cyl_cyl_compute_uvv};
use super::inscribe::{inscribe_point, min_max};
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S, IntSurfQuadric};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::{epsilon, real_to_int};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkmath::gp::{Point2, Point3};

/// The parameter bounds of the two cylinders: U first/last of S1, of S2, V first/last
/// of S1, of S2.
#[derive(Clone, Copy, Debug, Default)]
pub(in super::super) struct SurfBounds {
    pub(in super::super) u1f: f64,
    pub(in super::super) u1l: f64,
    pub(in super::super) u2f: f64,
    pub(in super::super) u2l: f64,
    pub(in super::super) v1f: f64,
    pub(in super::super) v1l: f64,
    pub(in super::super) v2f: f64,
    pub(in super::super) v2l: f64,
}

/// The fixed arguments of `AddPointIntoWL` (the quadrics, the system, the bounds and
/// the tolerances); S1 is the surface whose U parameter varies.
pub(in super::super) struct WlContext<'a> {
    pub(in super::super) quad1: &'a IntSurfQuadric,
    pub(in super::super) quad2: &'a IntSurfQuadric,
    pub(in super::super) coeffs: &'a CoeffsValue,
    pub(in super::super) is_reverse: bool,
    pub(in super::super) bounds: SurfBounds,
    pub(in super::super) period: f64,
    pub(in super::super) tol_3d: f64,
    pub(in super::super) tol_2d: f64,
}

/// The flags of `AddPointIntoWL`: refine the line after the point, allow a U1 before
/// the last one (`theFlBefore`), only check (`theOnlyCheck`).
#[derive(Clone, Copy, Default)]
pub(in super::super) struct AddFlags {
    pub(in super::super) precise: bool,
    pub(in super::super) before: bool,
    pub(in super::super) only_check: bool,
}

/// The parameters of a line point on the surface of varying U1 (`on_s2` when reversed).
fn params_on(p: &IntSurfPntOn2S, on_s2: bool) -> (f64, f64) {
    if on_s2 {
        p.parameters_on_s2()
    } else {
        p.parameters_on_s1()
    }
}

impl WlContext<'_> {
    /// The U1 parameter inscribed in the range of S1 (forcefully past its middle), then
    /// moved by a period when both bounds fit and the last point of the line is nearer.
    fn inscribed_u1(&self, u1: f64, line: &IntSurfLineOn2S) -> Result<Option<f64>, Failure> {
        let b = &self.bounds;
        let (period, tol) = (self.period, self.tol_2d);
        let mut u1par: f64 = u1;
        // aU1par always increases: its value must be reduced to continue the line.
        let force: bool = u1par > 0.5 * (b.u1f + b.u1l);
        if !inscribe_point(b.u1f, b.u1l, &mut u1par, tol, period, force) {
            return Ok(None);
        }
        if line.nb_points() > 0
            && (b.u1l - b.u1f) >= (period - tol)
            && ((u1par + period - b.u1l) <= tol || (u1par - period - b.u1f) >= tol)
        {
            // aU1par can be adjusted to both bounds: choose by the last point.
            let (u1_last, _) = params_on(&line.value(line.nb_points())?, self.is_reverse);
            let delta: f64 = u1_last - u1par;
            if 2.0 * delta.abs() > period {
                u1par += period.copysign(delta);
            }
        }
        Ok(Some(u1par))
    }

    /// The point on both surfaces when U2 and both V are inside the ranges.
    fn domain_point(&self, u1par: f64, p1: Point2, p2: Point2) -> Option<IntSurfPntOn2S> {
        let b = &self.bounds;
        let tol: f64 = self.tol_2d;
        let mut u2par: f64 = p2.x();
        if !inscribe_point(b.u2f, b.u2l, &mut u2par, tol, self.period, false) {
            return None;
        }
        let (v1par, v2par) = (p1.y(), p2.y());
        if (v1par - b.v1l > tol) || (b.v1f - v1par > tol) {
            return None;
        }
        if (v2par - b.v2l > tol) || (b.v2f - v2par > tol) {
            return None;
        }
        let pt1: Point3 = self.quad1.value(p1.x(), p1.y());
        let pt2: Point3 = self.quad2.value(p2.x(), p2.y());
        let mid = Point3::from((pt1.xyz() + pt2.xyz()) / 2.0);
        let mut pnt = IntSurfPntOn2S::new();
        if self.is_reverse {
            pnt.set_value_all(mid, u2par, v2par, u1par, v1par);
        } else {
            pnt.set_value_all(mid, u1par, v1par, u2par, v2par);
        }
        Some(pnt)
    }
}

/// `AddPointIntoWL(theQuad1, theQuad2, theCoeffs, isTheReverse, isThePrecise,
/// thePntOnSurf1, thePntOnSurf2, ..., theLine, theWLIndex, theTol3D, theTol2D,
/// theFlBefore, theOnlyCheck)`: adds the intersection point of the parameters `pnt1`
/// (on S1, whose U varies) and `pnt2` to the line when it can be inscribed in the
/// domain; U1 must increase unless `before`; a point the same as the last one replaces
/// it; with `precise`, the line is refined when its steps become non-uniform.
pub(in super::super) fn add_point_into_wl(
    ctx: &WlContext<'_>,
    flags: AddFlags,
    (pnt1, pnt2): (Point2, Point2),
    line: &IntSurfLineOn2S,
    wl_index: i32,
) -> Result<bool, Failure> {
    let Some(mut u1par) = ctx.inscribed_u1(pnt1.x(), line)? else {
        return Ok(false);
    };
    let Some(pnt) = ctx.domain_point(u1par, pnt1, pnt2) else {
        return Ok(false);
    };
    let nb_pnts: i32 = line.nb_points();
    if nb_pnts > 0 {
        let plast: IntSurfPntOn2S = line.value(nb_pnts)?;
        let (ul, _) = params_on(&plast, ctx.is_reverse);
        if !flags.before && u1par <= ul {
            // The parameter must increase unless theFlBefore.
            u1par += ctx.period;
            let b = &ctx.bounds;
            if (b.u1f - u1par > ctx.tol_2d) || (u1par - b.u1l > ctx.tol_2d) {
                // The new aU1par is out of the target interval.
                return Ok(false);
            }
        }
        if flags.only_check {
            return Ok(true);
        }
        // theTol2D is the minimal step: two neighbours would always be "same" with it.
        let dtol: f64 = 1.0 - epsilon(1.0);
        if pnt.is_same(&plast, ctx.tol_3d * dtol, ctx.tol_2d * dtol) {
            line.remove_point(nb_pnts)?;
        }
    }
    if flags.only_check {
        return Ok(true);
    }
    line.add(&pnt);
    if !flags.precise {
        return Ok(true);
    }
    precise_last_steps(ctx, line, wl_index)?;
    Ok(true)
}

/// Adds points before the last one when the step before the last one is more than
/// twice (and less than 2000 times) the last step.
fn precise_last_steps(
    ctx: &WlContext<'_>,
    line: &IntSurfLineOn2S,
    wl_index: i32,
) -> Result<(), Failure> {
    let nb_pnts: i32 = line.nb_points();
    if nb_pnts < 3 {
        return Ok(());
    }
    let (u3, _) = params_on(&line.value(nb_pnts)?, ctx.is_reverse);
    let (u2, _) = params_on(&line.value(nb_pnts - 1)?, ctx.is_reverse);
    let (u1, _) = params_on(&line.value(nb_pnts - 2)?, ctx.is_reverse);
    let step_prev: f64 = u2 - u1;
    let step: f64 = u3 - u2;
    let delta_step: i32 = real_to_int(step_prev / step);
    if 1 < delta_step && delta_step < 2000 {
        // New points where the existing points are not uniformly distributed.
        let range = (nb_pnts - 2, nb_pnts - 1);
        seek_additional_points(ctx, line, wl_index, delta_step, range)?;
    }
    Ok(())
}

/// `SeekAdditionalPoints(theQuad1, theQuad2, theLine, theCoeffs, theWLIndex,
/// theMinNbPoints, theStartPointOnLine, theEndPointOnLine, theTol2D, thePeriodOfSurf2,
/// isTheReverse)`: inserts a point in the middle of every interval of the part of the
/// line longer than the minimal step, pass after pass, until the part has
/// `min_nb_points` points or a pass adds nothing.
pub(in super::super) fn seek_additional_points(
    ctx: &WlContext<'_>,
    line: &IntSurfLineOn2S,
    wl_index: i32,
    min_nb_points: i32,
    (start, end): (i32, i32),
) -> Result<(), Failure> {
    let mut nb_points: i32 = end - start + 1;
    let (u1, _) = params_on(&line.value(start)?, ctx.is_reverse);
    let (u2, _) = params_on(&line.value(end)?, ctx.is_reverse);
    let min_delta: f64 = std_max((u2 - u1).abs() / f64::from(min_nb_points), ctx.tol_2d);
    let mut last_point_index: i32 = end;
    loop {
        let nb_points_prev: i32 = nb_points;
        let mut fp: i32 = start;
        while fp < last_point_index {
            let lp: i32 = fp + 1;
            if insert_middle(ctx, line, wl_index, (fp, lp), min_delta)? {
                nb_points += 1;
                last_point_index += 1;
            }
            fp = lp + 1;
        }
        if nb_points >= min_nb_points {
            return Ok(());
        }
        if !(nb_points < min_nb_points && nb_points != nb_points_prev) {
            return Ok(());
        }
    }
}

/// The middle point of the interval `(fp, lp)` inserted before `lp`, when the interval
/// is longer than the minimal step and the U2 of the middle fits the interval on S2.
fn insert_middle(
    ctx: &WlContext<'_>,
    line: &IntSurfLineOn2S,
    wl_index: i32,
    (fp, lp): (i32, i32),
    min_delta: f64,
) -> Result<bool, Failure> {
    let (pf, pl) = (line.value(fp)?, line.value(lp)?);
    let (u1f, _) = params_on(&pf, ctx.is_reverse);
    let (u1l, _) = params_on(&pl, ctx.is_reverse);
    let (mut u2f, _) = params_on(&pf, !ctx.is_reverse);
    let (mut u2l, _) = params_on(&pl, !ctx.is_reverse);
    if (u1l - u1f).abs() <= min_delta {
        // The step is minimal: it is not divided.
        return Ok(false);
    }
    let u1prec: f64 = 0.5 * (u1f + u1l);
    let Some((mut u2prec, v1prec, v2prec)) = cyl_cyl_compute_uvv(u1prec, wl_index, ctx.coeffs)?
    else {
        return Ok(false);
    };
    min_max(&mut u2f, &mut u2l);
    if !inscribe_point(u2f, u2l, &mut u2prec, ctx.tol_2d, ctx.period, false) {
        return Ok(false);
    }
    let p1: Point3 = ctx.quad1.value(u1prec, v1prec);
    let p2: Point3 = ctx.quad2.value(u2prec, v2prec);
    let pint = Point3::from(0.5 * (p1.xyz() + p2.xyz()));
    let mut ip = IntSurfPntOn2S::new();
    if ctx.is_reverse {
        ip.set_value_all(pint, u2prec, v2prec, u1prec, v1prec);
    } else {
        ip.set_value_all(pint, u1prec, v1prec, u2prec, v2prec);
    }
    line.insert_before(lp, &ip)?;
    Ok(true)
}
