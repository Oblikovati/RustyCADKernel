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
// Rust translation of PerformCurve of ModelingData/TKGeomBase/GCPnts/
// GCPnts_TangentialDeflection.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `PerformCurve`: the start-up of the march (line and singularity detection) and the
//! march itself. The native function keeps its many locals across one long body; here
//! they live in [`March`] so that each step of the body is one method.
use super::curve::difference;
use super::{DeflectionCurve, TangentialDeflection, US3};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Point3, Xyz};

/// The locals of `PerformCurve` that live across its phases.
pub(super) struct March {
    pub u1: f64,
    pub u2: f64,
    pub du: f64,
    pub dusave: f64,
    pub not_done: bool,
    pub a_tol: f64,
    pub last_point: Point3,
    pub current_point: Point3,
    pub intervs: Array1<'static, f64>,
}

/// Whether the start-up finished the whole computation.
enum Startup {
    Finished,
    March,
}

impl TangentialDeflection {
    /// `PerformCurve`: march along the curve, then refine the sampled points.
    pub(super) fn perform_curve(&mut self, curve: &impl DeflectionCurve) -> Result<(), Failure> {
        let mut march: March = self.march_start(curve)?;
        if let Startup::Finished = self.detect_line(curve, &mut march)? {
            return Ok(());
        }
        if self.clamp_initial_step(&mut march) {
            return Ok(());
        }
        self.march_points(curve, &mut march)?;
        self.readjust_second_to_last(curve)?;
        self.reach_minimum_points(curve)?;
        self.split_deflected_intervals(curve)
    }
    // "Initialization of the computation": the angular tolerance, the last point, the
    // first step estimate and the CN intervals.
    fn march_start(&mut self, curve: &impl DeflectionCurve) -> Result<March, Failure> {
        // `if (ATol > 1.e-2) ATol = 1.e-2; else if (ATol < 1.e-7) ATol = 1.e-7;` (NaN stays)
        let a_tol: f64 = (1.0e-2 * self.angular_deflection).clamp(1.0e-7, 1.0e-2);
        let last_point: Point3 = curve.lifted_d0(self.last_u)?;
        let dusave: f64 = (self.last_u - self.first_u) * US3;
        let estimate = self.evaluate_du(curve, self.first_u, dusave, true)?;
        self.append(self.first_u, estimate.point);
        // Used to detect "isLine" current bspline and in Du computation in general handling.
        let intervs: Array1<'static, f64> = curve.cn_intervals()?;
        Ok(March {
            u1: self.first_u,
            u2: self.first_u,
            du: estimate.du,
            dusave,
            not_done: estimate.not_done,
            a_tol,
            last_point,
            current_point: estimate.point,
            intervs,
        })
    }
    // `if (NotDone || Du > 5. * Dusave)`: a line (computed as such) or a singularity.
    fn detect_line(
        &mut self,
        curve: &impl DeflectionCurve,
        march: &mut March,
    ) -> Result<Startup, Failure> {
        if !(march.not_done || march.du > 5.0 * march.dusave) {
            return Ok(Startup::March);
        }
        let v1: Xyz = difference(march.last_point, march.current_point);
        let l1: f64 = v1.modulus();
        if l1 > precision::confusion() {
            return self.line_or_singularity(curve, march, v1, l1);
        }
        self.degenerate_chord(curve, march)
    }
    // The chord is not degenerate: sample each CN interval to decide whether the curve
    // is a line.
    fn line_or_singularity(
        &mut self,
        curve: &impl DeflectionCurve,
        march: &mut March,
        v1: Xyz,
        l1: f64,
    ) -> Result<Startup, Failure> {
        let nb_points: i32 = self.line_test_points(curve)?;
        let (is_line, param) = self.is_line(curve, march, (v1, l1), nb_points)?;
        if is_line {
            self.parameters.clear();
            self.points.clear();
            self.perform_linear(curve)?;
            return Ok(Startup::Finished);
        }
        // It was a singularity, continue:
        let estimate = self.evaluate_du(curve, param, march.du, march.not_done)?;
        (march.du, march.not_done) = (estimate.du, estimate.not_done);
        Ok(Startup::March)
    }
    // The number of samples per interval of the line test.
    fn line_test_points(&self, curve: &impl DeflectionCurve) -> Result<i32, Failure> {
        let nb_points: i32 = if self.min_nb_pnts > 3 {
            self.min_nb_pnts
        } else {
            3
        };
        Ok(match curve.kind()? {
            CurveType::BSplineCurve => (curve.bspline_shape()?.1 + 1).max(nb_points),
            CurveType::BezierCurve => (curve.bezier_shape()?.1 + 1).max(nb_points),
            _ => nb_points,
        })
    }
    // The IsLine loop; it clips the intervals to [myFirstu, myLastU] in place, as
    // natively, and returns the last sampled parameter.
    fn is_line(
        &self,
        curve: &impl DeflectionCurve,
        march: &mut March,
        chord: (Xyz, f64),
        nb_points: i32,
    ) -> Result<(bool, f64), Failure> {
        let (mut is_line, mut param) = (true, 0.0);
        let nb_interv: i32 = march.intervs.upper() - 1;
        let mut i: i32 = 1;
        while i <= nb_interv && is_line {
            if self.clip_interval(&mut march.intervs, i) {
                let (lower, upper) = (march.intervs[i], march.intervs[i + 1]);
                let delta: f64 = (upper - lower) / f64::from(nb_points);
                let mut j: i32 = 1;
                while j <= nb_points && is_line {
                    param = lower + f64::from(j) * delta;
                    is_line = self.is_aligned(curve, march, chord, param)?;
                    j += 1;
                }
            }
            i += 1;
        }
        Ok((is_line, param))
    }
    // Skips intervals outside [myFirstu, myLastU] and clips the border ones.
    fn clip_interval(&self, intervs: &mut Array1<'static, f64>, i: i32) -> bool {
        if (intervs[i + 1] < self.first_u) || (intervs[i] > self.last_u) {
            return false;
        }
        if (intervs[i] < self.first_u) && (intervs[i + 1] > self.first_u) {
            intervs[i] = self.first_u;
        }
        if (intervs[i] < self.last_u) && (intervs[i + 1] > self.last_u) {
            intervs[i + 1] = self.last_u;
        }
        true
    }
    // One sample of the line test: the angle between the chord and the sample direction.
    fn is_aligned(
        &self,
        curve: &impl DeflectionCurve,
        march: &March,
        (v1, l1): (Xyz, f64),
        param: f64,
    ) -> Result<bool, Failure> {
        let middle_point: Point3 = curve.lifted_d0(param)?;
        let v2: Xyz = difference(middle_point, march.current_point);
        let l2: f64 = v2.modulus();
        if l2 > precision::confusion() {
            let angle: f64 = v2.cross_magnitude(v1) / (l1 * l2);
            return Ok(angle < march.a_tol);
        }
        Ok(true)
    }
    // The chord is degenerate: probe the middle; a zero-length curve gives a segment.
    fn degenerate_chord(
        &mut self,
        curve: &impl DeflectionCurve,
        march: &mut March,
    ) -> Result<Startup, Failure> {
        march.du = (self.last_u - self.first_u) / 2.1;
        let middle_u: f64 = self.first_u + march.du;
        let middle_point: Point3 = curve.lifted_d0(middle_u)?;
        let l1: f64 = difference(middle_point, march.current_point).modulus();
        if l1 < precision::confusion() {
            // L1 < LTol: this is a zero-length curve, computation is finished:
            // return a segment of 2 points (protection)
            self.append(self.last_u, march.last_point);
            return Ok(Startup::Finished);
        }
        Ok(Startup::March)
    }
    // Keeps the first step below one third of the range and above myUTol; true when the
    // range itself is below myUTol (the result is then a segment).
    fn clamp_initial_step(&mut self, march: &mut March) -> bool {
        if march.du > march.dusave {
            march.du = march.dusave;
        } else {
            march.dusave = march.du;
        }
        if march.du < self.u_tol {
            march.du = self.last_u - self.first_u;
            if march.du < self.u_tol {
                self.append(self.last_u, march.last_point);
                return true;
            }
        }
        false
    }
}

/// `getIntervalIdx`: the interval of `param`, searched from `previous`.
pub(super) fn interval_index(param: f64, intervs: &Array1<'static, f64>, previous: i32) -> i32 {
    let mut index: i32 = previous;
    while index < intervs.upper() {
        if param >= intervs[index] && param <= intervs[index + 1] {
            break;
        }
        index += 1;
    }
    index
}
