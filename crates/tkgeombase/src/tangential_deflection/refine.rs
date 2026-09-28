// Copyright (c) 1995-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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
// Rust translation of the end of PerformCurve and of EstimDefl in
// ModelingData/TKGeomBase/GCPnts/GCPnts_TangentialDeflection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The refinement after the march: the second-to-last point is moved to the middle of
//! the last span, midpoints are added until `myMinNbPnts`, and every span whose
//! maximal deflection (EstimDefl) exceeds the criteria is split at that maximum.
use super::curve::{ChordLine, estimate_angle};
use super::{DeflectionCurve, TangentialDeflection};
use crate::failure_capture::FailureCapture;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::math::{BrentMinimum, Function, MultipleVarFunction, Pso, Vector};

impl TangentialDeflection {
    /// "Readjustment of the second to last point".
    pub(super) fn readjust_second_to_last(
        &mut self,
        curve: &impl DeflectionCurve,
    ) -> Result<(), Failure> {
        let i: usize = self.points.len().wrapping_sub(1);
        if self.points.len() >= 3 {
            let middle_u: f64 = (self.last_u + self.parameters[i - 2]) * 0.5;
            let middle_point: Point3 = curve.lifted_d0(middle_u)?;
            self.parameters[i - 1] = middle_u;
            self.points[i - 1] = middle_point;
        }
        Ok(())
    }
    /// Adds points at midpoints of segments while the minimum number of points is not
    /// reached; the pass bound follows the growing count, as natively.
    pub(super) fn reach_minimum_points(
        &mut self,
        curve: &impl DeflectionCurve,
    ) -> Result<(), Failure> {
        let mut nbp: usize = self.points.len();
        while (nbp as i32) < self.min_nb_pnts {
            let mut i: usize = 2;
            while i <= nbp {
                let middle_u: f64 = (self.parameters[i - 2] + self.parameters[i - 1]) * 0.5;
                let middle_point: Point3 = curve.lifted_d0(middle_u)?;
                self.parameters.insert(i - 1, middle_u);
                self.points.insert(i - 1, middle_point);
                nbp += 1;
                i += 2;
            }
        }
        Ok(())
    }
    /// "Additional check for intervals": split each span whose maximal deflection or
    /// angle exceeds the criteria at the point of maximal deflection.
    pub(super) fn split_deflected_intervals(
        &mut self,
        curve: &impl DeflectionCurve,
    ) -> Result<(), Failure> {
        let min_len2: f64 = self.min_len * self.min_len;
        let mut nbp: usize = self.points.len();
        let max_nbp: usize = 10 * nbp;
        let mut i: usize = 1;
        while i < nbp {
            if self.split_span(curve, i, min_len2)? {
                nbp += 1;
                if nbp > max_nbp {
                    break;
                }
                // i must point to first part of split interval: it is not advanced.
                continue;
            }
            i += 1;
        }
        Ok(())
    }
    // One span of the interval check; true when it was split.
    fn split_span(
        &mut self,
        curve: &impl DeflectionCurve,
        i: usize,
        min_len2: f64,
    ) -> Result<bool, Failure> {
        let (u1, u2) = (self.parameters[i - 1], self.parameters[i]);
        if u2 - u1 <= self.u_tol {
            return Ok(false);
        }
        // Check maximal deflection on interval;
        let (dmax, umax) = self.estimate_deflection(curve, u1, u2)?;
        let (p1, p2) = (self.points[i - 1], self.points[i]);
        let middle_point: Point3 = curve.lifted_d0(umax)?;
        let amax: f64 = estimate_angle(p1, middle_point, p2);
        let angle_max: f64 = self.angular_deflection * 0.5;
        let too_deflected: bool = dmax > self.curvature_deflection || amax > angle_max;
        let is_interior: bool = umax - u1 > self.u_tol && u2 - umax > self.u_tol;
        if !too_deflected || !is_interior {
            return Ok(false);
        }
        if p1.square_distance(&middle_point) > min_len2
            && p2.square_distance(&middle_point) > min_len2
        {
            self.parameters.insert(i, umax);
            self.points.insert(i, middle_point);
            return Ok(true);
        }
        Ok(false)
    }
    /// `EstimDefl`: the maximal distance between the curve and the chord of `[u1, u2]`
    /// and its parameter, by Brent minimization, retried from a PSO estimate.
    fn estimate_deflection(
        &self,
        curve: &impl DeflectionCurve,
        u1: f64,
        u2: f64,
    ) -> Result<(f64, f64), Failure> {
        let du: f64 = self.last_u - self.first_u;
        let chord: ChordLine = curve.chord(u1, u2)?;
        let mut function = ChordDistance {
            curve,
            chord,
            range: (u1, u2),
            capture: FailureCapture::default(),
        };
        let rel_tol: f64 = std_max(1.0e-3, 2.0 * self.u_tol / (u1.abs() + u2.abs()));
        let mut opt_loc = BrentMinimum::with_options(rel_tol, 100, self.u_tol);
        opt_loc.perform(&mut function, u1, (u1 + u2) / 2.0, u2);
        function.capture.take()?;
        if opt_loc.is_done() {
            return brent_maximum(&opt_loc);
        }
        self.estimate_with_swarm(&mut function, &mut opt_loc, du)
    }
    // The PSO fallback of EstimDefl, then a second Brent search around its estimate.
    fn estimate_with_swarm<C: DeflectionCurve>(
        &self,
        function: &mut ChordDistance<'_, C>,
        opt_loc: &mut BrentMinimum,
        du: f64,
    ) -> Result<(f64, f64), Failure> {
        let (u1, u2) = function.range;
        let steps: Vector<'static> = single(std_max(0.1 * du, 100.0 * self.u_tol))?;
        let nb_particles: i32 = real_to_int(32.0 * (u2 - u1) / du).max(8);
        let (low, upp) = (single(u1)?, single(u2)?);
        let (mut value, mut t) = (0.0, single(0.0)?);
        {
            let mut function_mv = ChordDistanceMV(function);
            let mut finder =
                Pso::with_options(&mut function_mv, &low, &upp, &steps, nb_particles, 100)?;
            finder.perform(&steps, &mut value, &mut t)?;
        }
        function.capture.take()?;
        let (t1, s1) = (t[1], steps[1]);
        opt_loc.perform(function, std_max(t1 - s1, u1), t1, std_min(t1 + s1, u2));
        function.capture.take()?;
        if opt_loc.is_done() {
            return brent_maximum(opt_loc);
        }
        Ok(((-value).sqrt(), t1))
    }
}

// `std::sqrt(-anOptLoc.Minimum())` and `anOptLoc.Location()` of a done search.
fn brent_maximum(opt_loc: &BrentMinimum) -> Result<(f64, f64), Failure> {
    Ok(((-opt_loc.minimum()?).sqrt(), opt_loc.location()?))
}

// A 1-based math_Vector(1, 1) holding `value`.
fn single(value: f64) -> Result<Vector<'static>, Failure> {
    let mut vector: Vector<'static> = Vector::with_bounds(1, 1)?;
    vector[1] = value;
    Ok(vector)
}

/// `RealToInt`: saturating conversion; NaN converts as the x86 `cvttsd2si` does.
fn real_to_int(value: f64) -> i32 {
    if value < f64::from(i32::MIN) {
        return i32::MIN;
    }
    if value > f64::from(i32::MAX) {
        return i32::MAX;
    }
    if value.is_nan() {
        return i32::MIN;
    }
    value as i32
}

/// `GCPnts_DistFunction` / `GCPnts_DistFunction2d`: minus the square distance from the
/// curve to the chord of `[U1, U2]`, undefined (false) outside the range.
struct ChordDistance<'c, C: DeflectionCurve> {
    curve: &'c C,
    chord: ChordLine,
    range: (f64, f64),
    capture: FailureCapture,
}

impl<C: DeflectionCurve> Function for ChordDistance<'_, C> {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        if x < self.range.0 || x > self.range.1 {
            return false;
        }
        match self.capture.keep(self.curve.chord_value(&self.chord, x)) {
            Some(value) => {
                *result = value;
                true
            }
            None => false,
        }
    }
}

/// `GCPnts_DistFunctionMV` / `GCPnts_DistFunction2dMV`: the one-variable function seen
/// by math_PSO.
struct ChordDistanceMV<'f, 'c, C: DeflectionCurve>(&'f mut ChordDistance<'c, C>);

impl<C: DeflectionCurve> MultipleVarFunction for ChordDistanceMV<'_, '_, C> {
    fn variables(&self) -> i32 {
        1
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        Function::value(self.0, point[1], value)
    }
}
