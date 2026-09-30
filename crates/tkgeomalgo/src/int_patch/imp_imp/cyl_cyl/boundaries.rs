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
// Rust translation of the WorkWithBoundaries class and ExtremaLineLine of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The boundaries of the cylinder-cylinder walking: the U ranges where the lines exist,
//! the estimated V ranges, and the points where a line crosses a V boundary.
// The negated comparisons keep the native result for NaN operands.
#![allow(clippy::neg_cmp_op_on_partial_ord)]
use super::coeffs::{CoeffsValue, cyl_cyl_compute_uvv};
use super::inscribe::insertion_sort;
use super::wline::{AddFlags, SurfBounds, WlContext, add_point_into_wl};
use crate::int_surf::{IntSurfLineOn2S, IntSurfQuadric};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::bnd::{BoundingBox2, Range1};
use rustycad_tkmath::gp::{Axis1, Cylinder3, Point2};
use rustycad_tkmath::math_root::brent;
use rustycad_tkmath::math_utils::{Config, Status};

/// `WorkWithBoundaries`: the quadrics, the system and the bounds of the walking (the
/// first surface is the one whose U parameter varies, reversed or not).
pub(in super::super) struct WorkWithBoundaries<'a> {
    pub(in super::super) quad1: &'a IntSurfQuadric,
    pub(in super::super) quad2: &'a IntSurfQuadric,
    pub(in super::super) coeffs: &'a CoeffsValue,
    pub(in super::super) uv_surf1: &'a BoundingBox2,
    pub(in super::super) uv_surf2: &'a BoundingBox2,
    pub(in super::super) period: f64,
    pub(in super::super) tol_3d: f64,
    pub(in super::super) tol_2d: f64,
    pub(in super::super) is_reverse: bool,
}

/// `StPInfo`: a point found on a V boundary (`mySurfID` 0 for the first surface).
#[derive(Clone, Copy, Debug)]
struct StPInfo {
    surf_id: i32,
    u1: f64,
    v1: f64,
    u2: f64,
    v2: f64,
}

impl Default for StPInfo {
    fn default() -> Self {
        Self { surf_id: 0, u1: f64::MAX, v1: f64::MAX, u2: f64::MAX, v2: f64::MAX }
    }
}

/// The inputs of `AddBoundaryPoint`: the current and previous U1 and V values of the
/// line and the least U1 allowed (the native U2 input is overwritten before any read).
pub(in super::super) struct BoundaryStep {
    pub(in super::super) u1: f64,
    pub(in super::super) u1_prev: f64,
    pub(in super::super) u1_min: f64,
    pub(in super::super) v1: f64,
    pub(in super::super) v1_prev: f64,
    pub(in super::super) v2: f64,
    pub(in super::super) v2_prev: f64,
}

impl WorkWithBoundaries<'_> {
    /// The bounds of both surfaces (`Bnd_Box2d::Get`, Standard_ConstructionError when
    /// void).
    pub(in super::super) fn bounds(&self) -> Result<SurfBounds, Failure> {
        let [u1f, v1f, u1l, v1l] = self.uv_surf1.get()?;
        let [u2f, v2f, u2l, v2l] = self.uv_surf2.get()?;
        Ok(SurfBounds { u1f, u1l, u2f, u2l, v1f, v1l, v2f, v2l })
    }

    /// The context of `AddPointIntoWL`.
    pub(in super::super) fn context(&self) -> Result<WlContext<'_>, Failure> {
        Ok(WlContext {
            quad1: self.quad1,
            quad2: self.quad2,
            coeffs: self.coeffs,
            is_reverse: self.is_reverse,
            bounds: self.bounds()?,
            period: self.period,
            tol_3d: self.tol_3d,
            tol_2d: self.tol_2d,
        })
    }

    /// The crossing of `V(U1) = v_bound` on the line `wl_index` in `[u_lo, u_hi]`: a
    /// bracketed 1D root (MathRoot::Brent to Precision::PConfusion), which replaced the
    /// rank-deficient 3x3 Newton near a V extremum.
    fn find_v_bound_crossing(&self, wl_index: i32, is_v1: bool, v_bound: f64, (u_lo, u_hi): (f64, f64)) -> Result<Option<f64>, Failure> {
        if !(u_lo < u_hi) {
            return Ok(None);
        }
        let coeffs: &CoeffsValue = self.coeffs;
        let mut failure: Option<Failure> = None;
        let mut func = |x: f64, f: &mut f64| -> bool {
            match cyl_cyl_compute_uvv(x, wl_index, coeffs) {
                Ok(Some((_, v1, v2))) => {
                    *f = (if is_v1 { v1 } else { v2 }) - v_bound;
                    true
                }
                Ok(None) => false,
                Err(e) => {
                    failure.get_or_insert(e);
                    false
                }
            }
        };
        let (mut flo, mut fhi) = (0.0, 0.0);
        if !func(u_lo, &mut flo) || !func(u_hi, &mut fhi) {
            return failure.map_or(Ok(None), Err);
        }
        if flo == 0.0 {
            return Ok(Some(u_lo));
        }
        if fhi == 0.0 {
            return Ok(Some(u_hi));
        }
        if flo * fhi > 0.0 {
            return Ok(None); // no bracketed crossing
        }
        let cfg = Config { x_tolerance: precision::p_confusion(), ..Config::default() };
        let res = brent(&mut func, u_lo, u_hi, &cfg);
        if let Some(e) = failure {
            return Err(e);
        }
        if res.status != Status::Ok {
            return Ok(None);
        }
        Ok(res.root)
    }

    /// The point on the bound `index` (V first or last of S1, then of S2) when the V
    /// segment of the step reaches it, `RealLast` in U1 otherwise.
    fn boundary_point(&self, step: &BoundaryStep, wl_index: i32, index: usize, arr_vzad: &[f64; 4]) -> Result<StPInfo, Failure> {
        let on_first: bool = index < 2;
        let mut pt = StPInfo { surf_id: if on_first { 0 } else { 2 }, ..StPInfo::default() };
        let (vf, vl) = if on_first { (step.v1, step.v1_prev) } else { (step.v2, step.v2_prev) };
        let vz: f64 = arr_vzad[index];
        if (vf - vz).abs() > self.tol_2d && (vf - vz) * (vl - vz) > 0.0 {
            return Ok(pt);
        }
        // [Vf, Vl] crosses a V bound: resolved by bisection on the analytical curve.
        let range = (std_min(step.u1_prev, step.u1), std_max(step.u1_prev, step.u1));
        let found = self.find_v_bound_crossing(wl_index, on_first, vz, range)?;
        let Some(u1) = found.filter(|&u| !(u >= step.u1) && !(u < step.u1_min)) else {
            // Not found, or out of the domain.
            return Ok(pt);
        };
        let Some((u2, v1, v2)) = cyl_cyl_compute_uvv(u1, wl_index, self.coeffs)? else {
            // The found point is wrong.
            return Ok(pt);
        };
        (pt.u1, pt.u2, pt.v1, pt.v2) = (u1, u2, v1, v2);
        // The point on the true V boundary.
        if on_first {
            pt.v1 = vz;
        } else {
            pt.v2 = vz;
        }
        Ok(pt)
    }

    /// `AddBoundaryPoint(theWL, theU1, theU1Prev, theU1Min, theU2, theV1, theV1Prev,
    /// theV2, theV2Prev, theWLIndex, theFlForce, isTheFound1, isTheFound2)`: the points
    /// where the line crosses the V boundaries between the previous and the current U1
    /// go into the line, in increasing U1; returns whether a point on S1 (and on S2)
    /// was added.
    pub(in super::super) fn add_boundary_point(
        &self,
        line: &IntSurfLineOn2S,
        step: &BoundaryStep,
        wl_index: i32,
        fl_force: bool,
    ) -> Result<(bool, bool), Failure> {
        let b = self.bounds()?;
        let arr_vzad: [f64; 4] = [b.v1f, b.v1l, b.v2f, b.v2l];
        let mut uv_point = [StPInfo::default(); 4];
        for (index, slot) in uv_point.iter_mut().enumerate() {
            *slot = self.boundary_point(step, wl_index, index, &arr_vzad)?;
        }
        // Sort with ascending U1.
        insertion_sort(&mut uv_point, |a, b| a.u1 < b.u1);
        let ctx = self.context()?;
        let flags = AddFlags { precise: false, before: fl_force, only_check: false };
        let (mut found1, mut found2) = (false, false);
        for p in &uv_point {
            if p.u1 == f64::MAX {
                break;
            }
            let pnts = (Point2::new(p.u1, p.v1), Point2::new(p.u2, p.v2));
            if !add_point_into_wl(&ctx, flags, pnts, line, wl_index)? {
                continue;
            }
            if p.surf_id == 0 {
                found1 = true;
            } else {
                found2 = true;
            }
        }
        Ok((found1, found2))
    }

    /// `BoundaryEstimation(theCy1, theCy2, theOutBoxS1, theOutBoxS2)`: the V ranges of
    /// the parallelogram of the projections of both cylinders on the plane of their
    /// axes, enlarged by Precision::Confusion and cut by the surface ranges (void for
    /// nearly parallel axes).
    pub(in super::super) fn boundary_estimation(&self, cy1: &Cylinder3, cy2: &Cylinder3) -> Result<(Range1, Range1), Failure> {
        let (mut out1, mut out2) = (Range1::default(), Range1::default());
        let (d1, d2) = (cy1.axis().direction(), cy2.axis().direction());
        let (r1, r2) = (cy1.radius(), cy2.radius());
        let cos_a: f64 = d1.dot(d2);
        let sq_sin_a: f64 = d1.xyz().cross_square_magnitude(d2.xyz());
        // A small sine can be compared with the angle.
        if sq_sin_a < precision::angular() * precision::angular() {
            return Ok((out1, out2));
        }
        let sin_a: f64 = sq_sin_a.sqrt();
        let abs_cos_a: f64 = cos_a.abs();
        let hdv1: f64 = (r1 * abs_cos_a + r2) / sin_a;
        let hdv2: f64 = (r2 * abs_cos_a + r1) / sin_a;
        let (v01, v02) = extrema_line_line(&cy1.axis(), &cy2.axis(), cos_a, sq_sin_a);
        out1.add(v01 - hdv1);
        out1.add(v01 + hdv1);
        out2.add(v02 - hdv2);
        out2.add(v02 + hdv2);
        out1.enlarge(precision::confusion());
        out2.enlarge(precision::confusion());
        let [_, v1, _, v2] = self.uv_surf1.get()?;
        out1.common(Range1::new(v1, v2)?);
        let [_, v1, _, v2] = self.uv_surf2.get()?;
        out2.common(Range1::new(v1, v2)?);
        Ok((out1, out2))
    }
}

/// `ExtremaLineLine(theC1, theC2, theCosA, theSqSinA, thePar1, thePar2)`: the parameters
/// of the extrema of two lines (as Extrema_ExtElC).
fn extrema_line_line(c1: &Axis1, c2: &Axis1, cos_a: f64, sq_sin_a: f64) -> (f64, f64) {
    let (d1, d2) = (c1.direction(), c2.direction());
    let l1l2 = c2.location().xyz() - c1.location().xyz();
    let (d1l, d2l) = (d1.xyz().dot(l1l2), d2.xyz().dot(l1l2));
    ((d1l - cos_a * d2l) / sq_sin_a, (cos_a * d1l - d2l) / sq_sin_a)
}

/// Adds `[a, b]` to a range.
fn add_pair(range: &mut Range1, a: f64, b: f64) {
    range.add(a);
    range.add(b);
}

/// `acos` of the argument clamped to [-1, 1].
fn clamped_acos(arg: f64) -> f64 {
    // f64::clamp keeps NaN, as the two native comparisons do.
    arg.clamp(-1.0, 1.0).acos()
}

/// `BoundariesComputing(theCoeffs, thePeriod, theURange)`: the U1 ranges (one or two)
/// where `-1 <= B*cos(U1-FI1) + C <= 1`; false when there is none.
pub(in super::super) fn boundaries_computing(coeffs: &CoeffsValue, period: f64, u_range: &mut [Range1; 2]) -> bool {
    let (b, c, fi1) = (coeffs.b, coeffs.c, coeffs.fi1);
    if b > 0.0 {
        return positive_b(b, c, fi1, period, u_range);
    }
    if b < 0.0 {
        return negative_b(b, c, fi1, period, u_range);
    }
    false
}

/// `-(1+C)/B <= cos(U1-FI1) <= (1-C)/B`.
fn positive_b(b: f64, c: f64, fi1: f64, period: f64, u_range: &mut [Range1; 2]) -> bool {
    if b + c.abs() < -1.0 {
        // No solution.
        return false;
    }
    if b + c.abs() <= 1.0 {
        add_pair(&mut u_range[0], fi1, period + fi1);
    } else if (1.0 + c <= b) && (b <= 1.0 - c) {
        let d: f64 = clamped_acos(-(c + 1.0) / b);
        add_pair(&mut u_range[0], fi1, d + fi1);
        add_pair(&mut u_range[1], period - d + fi1, period + fi1);
    } else if (1.0 - c <= b) && (b <= 1.0 + c) {
        let d: f64 = clamped_acos((1.0 - c) / b);
        add_pair(&mut u_range[0], d + fi1, period - d + fi1);
    } else if b - c.abs() >= 1.0 {
        let (d1, d2) = (clamped_acos((1.0 - c) / b), clamped_acos(-(c + 1.0) / b));
        add_pair(&mut u_range[0], d1 + fi1, d2 + fi1);
        add_pair(&mut u_range[1], period - d2 + fi1, period - d1 + fi1);
    } else {
        return false;
    }
    true
}

/// `(1-C)/B <= cos(U1-FI1) <= -(1+C)/B`; the fourth test compares B with `-B - 1`, as
/// natively.
fn negative_b(b: f64, c: f64, fi1: f64, period: f64, u_range: &mut [Range1; 2]) -> bool {
    if b + c.abs() > 1.0 {
        // No solutions.
        return false;
    }
    if -b + c.abs() <= 1.0 {
        add_pair(&mut u_range[0], fi1, period + fi1);
    } else if (-c - 1.0 <= b) && (b <= c - 1.0) {
        let d: f64 = clamped_acos((1.0 - c) / b);
        add_pair(&mut u_range[0], fi1, d + fi1);
        add_pair(&mut u_range[1], period - d + fi1, period + fi1);
    } else if (c - 1.0 <= b) && (b <= -b - 1.0) {
        let d: f64 = clamped_acos(-(c + 1.0) / b);
        add_pair(&mut u_range[0], d + fi1, period - d + fi1);
    } else if -b - c.abs() >= 1.0 {
        let (d1, d2) = (clamped_acos(-(c + 1.0) / b), clamped_acos((1.0 - c) / b));
        add_pair(&mut u_range[0], d1 + fi1, d2 + fi1);
        add_pair(&mut u_range[1], period - d2 + fi1, period - d1 + fi1);
    } else {
        return false;
    }
    true
}
