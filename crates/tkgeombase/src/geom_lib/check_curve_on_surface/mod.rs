// Created by: Nikolai BUKHALOV
// Copyright (c) 2015 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib_CheckCurveOnSurface.hxx/.cxx
// (the class and GeomLib_CheckCurveOnSurface_Local) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomLib_CheckCurveOnSurface`: the maximum distance between a 3D curve and a
//! curve on surface at the same parameters.
mod min_computing;
mod sub_intervals;
mod target_func;

use min_computing::min_computing;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface};
use sub_intervals::{SubIntervals, fill_sub_intervals};
use target_func::TargetFunc;

/// The check of a 3D curve against a curve on surface: the greatest distance between
/// their points of equal parameter and where it is reached, or an error status
/// (1: a null curve, 2: ranges that do not match within the parameter tolerance, 3: a
/// failed computation).
///
/// Natively the parallel mode spreads the sub-intervals over threads on shallow
/// copies of the curves; the sub-interval results do not depend on the order, and
/// the port computes them in sequence in both modes.
///
/// `let mut check = CheckCurveOnSurface::with_curve(&c3d, 1e-9); check.perform(&cos);`
#[derive(Clone)]
pub struct CheckCurveOnSurface {
    my_curve: Handle<dyn Adaptor3dCurve>,
    my_error_status: i32,
    my_max_distance: f64,
    my_max_parameter: f64,
    my_tol_range: f64,
    my_is_parallel: bool,
}

impl Default for CheckCurveOnSurface {
    /// `GeomLib_CheckCurveOnSurface()`: no curve, the default parameter tolerance.
    fn default() -> Self {
        Self::with_curve(&Handle::default(), precision::p_confusion())
    }
}

impl CheckCurveOnSurface {
    /// The empty check (`GeomLib_CheckCurveOnSurface()`); `CheckCurveOnSurface::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// The check of `curve` with the parameter tolerance `tol_range`
    /// (`GeomLib_CheckCurveOnSurface(theCurve, theTolRange = Precision::PConfusion())`);
    /// `CheckCurveOnSurface::with_curve(&c3d, 1e-9)`.
    pub fn with_curve(curve: &Handle<dyn Adaptor3dCurve>, tol_range: f64) -> Self {
        Self {
            my_curve: curve.clone(),
            my_error_status: 0,
            my_max_distance: f64::MAX,
            my_max_parameter: 0.0,
            my_tol_range: tol_range,
            my_is_parallel: false,
        }
    }

    /// Sets the curve and the tolerance and resets the results; the parallel flag is
    /// kept (`Init(theCurve, theTolRange = Precision::PConfusion())`);
    /// `check.init(&c3d, 1e-9)`.
    pub fn init(&mut self, curve: &Handle<dyn Adaptor3dCurve>, tol_range: f64) {
        self.my_curve = curve.clone();
        self.reset_results(tol_range);
    }

    /// Forgets the curve and resets the results and the tolerance (`Init()`);
    /// `check.init_empty()`.
    pub fn init_empty(&mut self) {
        self.my_curve = Handle::default();
        self.reset_results(precision::p_confusion());
    }

    fn reset_results(&mut self, tol_range: f64) {
        self.my_error_status = 0;
        self.my_max_distance = f64::MAX;
        self.my_max_parameter = 0.0;
        self.my_tol_range = tol_range;
    }

    /// Computes the maximum distance to `curve_on_surface` over the curve range (the
    /// curve must lie within the range of the curve on surface, within the tolerance):
    /// on each sub-interval between the knots of both curves, the exact maximum of
    /// lines and conics, or a particle swarm refined by Newton
    /// (`Perform(theCurveOnSurface)`); `check.perform(&cos)`.
    pub fn perform(&mut self, curve_on_surface: &Handle<Adaptor3dCurveOnSurface>) {
        let (Some(curve), Some(cos)) = (self.my_curve.get(), curve_on_surface.get()) else {
            self.my_error_status = 1;
            return;
        };
        let (first, last) = match (
            curve.first_parameter(),
            curve.last_parameter(),
            cos.first_parameter(),
            cos.last_parameter(),
        ) {
            (Ok(f), Ok(l), Ok(cf), Ok(cl)) => {
                if (f - cf > self.my_tol_range) || (l - cl < -self.my_tol_range) {
                    self.my_error_status = 2;
                    return;
                }
                (f, l)
            }
            _ => {
                self.my_error_status = 3;
                return;
            }
        };
        match compute(curve, cos, first, last) {
            Ok(Some((max_distance, max_parameter))) => {
                self.my_max_distance = max_distance;
                self.my_max_parameter = max_parameter;
            }
            Ok(None) | Err(_) => self.my_error_status = 3,
        }
    }

    /// Sets the parallel mode (`SetParallel(theIsParallel)`); `check.set_parallel(true)`.
    pub fn set_parallel(&mut self, is_parallel: bool) {
        self.my_is_parallel = is_parallel;
    }

    /// The parallel mode (`IsParallel()`); `check.is_parallel()`.
    pub fn is_parallel(&self) -> bool {
        self.my_is_parallel
    }

    /// Whether the last check succeeded (`IsDone()`); `check.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_error_status == 0
    }

    /// The error status: 0, 1, 2 or 3 (`ErrorStatus()`); `check.error_status()`.
    pub fn error_status(&self) -> i32 {
        self.my_error_status
    }

    /// The maximum distance, RealLast before a successful check (`MaxDistance()`);
    /// `check.max_distance()`.
    pub fn max_distance(&self) -> f64 {
        self.my_max_distance
    }

    /// The parameter of the maximum distance (`MaxParameter()`); `check.max_parameter()`.
    pub fn max_parameter(&self) -> f64 {
        self.my_max_parameter
    }
}

// The sub-intervals, the minimum of each (RealLast at its start when it fails), and
// the smallest of them (the first of equal ones); None when the sub-intervals fail.
fn compute(
    curve: &dyn Adaptor3dCurve,
    cos: &Adaptor3dCurveOnSurface,
    first: f64,
    last: f64,
) -> Result<Option<(f64, f64)>, Failure> {
    // Polynomial function with degree n has not more than n-1 maxima and minima, so
    // at most n monotonicity intervals: n particles, one per interval.
    let an_epsilon_range: f64 = 1.0e-3;
    let intervals: SubIntervals = fill_sub_intervals(curve, &cos.get_curve(), first, last)?;
    if intervals.count == 0 {
        return Ok(None);
    }
    let mut optimal: Option<(f64, f64)> = None;
    for pair in intervals.bounds.windows(2) {
        let mut func = TargetFunc::new(curve, cos, pair[0], pair[1]);
        let (dist, param) = min_computing(&mut func, an_epsilon_range, intervals.nb_particles)
            .unwrap_or((f64::MAX, func.first_parameter()));
        optimal = match optimal {
            Some((min_dist, _)) if !(dist < min_dist) => optimal,
            _ => Some((dist, param)),
        };
    }
    Ok(optimal.map(|(dist, param)| (dist.abs().sqrt(), param)))
}
