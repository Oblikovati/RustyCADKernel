// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of GeomBndLib_OtherCurve and GeomBndLib_OtherCurve2d at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomBndLib_OtherCurve` and `GeomBndLib_OtherCurve2d`: the sampling fallback for any
//! curve adaptor.
//!
//! Both local `FillBox` copies of the .cxx files have the body of
//! `GeomBndLib_SplineHelpers::FillBox` (their degenerate branch adds the first point a
//! second time, which does not change a box), so [`fill_box`] serves them.
use super::optimization::{CurveCoordFunc, CurveCoordinate};
use super::sampling::compute_nb_samples_2d;
use super::space::{SampleBounds, SampledCurve};
use super::spline::{OptimalCurveBox, curve_box_optimal, fill_box};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_to_int;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::bnd::{BoundingBox2, BoundingBox3};
use rustycad_tkmath::bvh::{std_max, std_min};
use rustycad_tkmath::math::{BrentMinimum, Pso, Vector};

// `N = 33` sample pairs and the 1.5 deflection factor of `Box`.
const SAMPLE_PAIRS: i32 = 33;
const WEAKNESS: f64 = 1.5;

// `Box(U1, U2, Tol)` of both classes: the sampled box enlarged by 1.5 times the
// deflection, read back with the gap through the legacy `Get`, rebuilt without gap and
// enlarged by `tol`.
fn sampled_box<C: ?Sized + SampledCurve>(
    curve: &C,
    u1: f64,
    u2: f64,
    tolerance: f64,
) -> Result<C::Bounds, Failure> {
    let mut sampled: C::Bounds = C::Bounds::default();
    let deflection: f64 = fill_box(&mut sampled, curve, u1, u2, SAMPLE_PAIRS)?;
    sampled.enlarge_by(WEAKNESS * deflection);
    let (min, max) = sampled.min_max();
    let mut bounds: C::Bounds = C::Bounds::default();
    bounds.update_min_max(&min, &max);
    bounds.enlarge_by(tolerance);
    Ok(bounds)
}

/// `GeomBndLib_OtherCurve`: sampled boxes of an arbitrary `Adaptor3d_Curve`.
pub struct OtherCurve<'a> {
    curve: &'a dyn Adaptor3dCurve,
}

impl<'a> OtherCurve<'a> {
    /// The evaluator of `curve`, which it borrows; `OtherCurve::new(&adaptor)`.
    pub fn new(curve: &'a dyn Adaptor3dCurve) -> Self {
        Self { curve }
    }
    /// `Box(Tol)`: the adaptor's parameter range; `evaluator.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        let (u1, u2) = (self.curve.first_parameter()?, self.curve.last_parameter()?);
        self.bounding_box_range(u1, u2, tolerance)
    }
    /// `Box(U1, U2, Tol)`: 67 samples enlarged by 1.5 times the deflection;
    /// `evaluator.bounding_box_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        sampled_box(self.curve, u1, u2, tolerance)
    }
    /// `BoxOptimal(Tol)`: the adaptor's parameter range; `evaluator.bounding_box_optimal(0.0)?`.
    pub fn bounding_box_optimal(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        let (u1, u2) = (self.curve.first_parameter()?, self.curve.last_parameter()?);
        self.bounding_box_optimal_range(u1, u2, tolerance)
    }
    /// `BoxOptimal(U1, U2, Tol)`: the body of `CurveBoxOptimal` (ComputeNbSamples,
    /// MathOpt PSO + Brent); `evaluator.bounding_box_optimal_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        curve_box_optimal(self.curve, u1, u2, tolerance)
    }
}

/// `GeomBndLib_OtherCurve2d`: sampled boxes of an arbitrary `Adaptor2d_Curve2d`.
pub struct OtherCurve2d<'a> {
    curve: &'a dyn Adaptor2dCurve2d,
}

impl<'a> OtherCurve2d<'a> {
    /// The evaluator of `curve`, which it borrows; `OtherCurve2d::new(&adaptor)`.
    pub fn new(curve: &'a dyn Adaptor2dCurve2d) -> Self {
        Self { curve }
    }
    /// `Box(Tol)`: the adaptor's parameter range; `evaluator.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox2, Failure> {
        let (u1, u2) = (self.curve.first_parameter()?, self.curve.last_parameter()?);
        self.bounding_box_range(u1, u2, tolerance)
    }
    /// `Box(U1, U2, Tol)`: 67 samples enlarged by 1.5 times the deflection;
    /// `evaluator.bounding_box_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox2, Failure> {
        sampled_box(self.curve, u1, u2, tolerance)
    }
    /// `BoxOptimal(Tol)`: the adaptor's parameter range; `evaluator.bounding_box_optimal(0.0)?`.
    pub fn bounding_box_optimal(&self, tolerance: f64) -> Result<BoundingBox2, Failure> {
        let (u1, u2) = (self.curve.first_parameter()?, self.curve.last_parameter()?);
        self.bounding_box_optimal_range(u1, u2, tolerance)
    }
    /// `BoxOptimal(U1, U2, Tol)`: ComputeNbSamples2d samples refined by the legacy
    /// `math_PSO` + `math_BrentMinimum` search of the .cxx file (not the MathOpt one the
    /// other classes use); `evaluator.bounding_box_optimal_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox2, Failure> {
        let count: i32 = compute_nb_samples_2d(self.curve, u1, u2)?;
        let optimal = OptimalCurveBox {
            curve: self.curve,
            u1,
            u2,
            count,
        };
        optimal.compute(tolerance, &|range, extr0, index, eps, is_min| {
            adjust_extr_2d_curve(self.curve, range, extr0, index, eps, is_min)
        })
    }
}

// `AdjustExtr2dCurve` of GeomBndLib_OtherCurve2d.cxx: math_BrentMinimum on a small range,
// otherwise math_PSO then math_BrentMinimum around the swarm optimum.
fn adjust_extr_2d_curve(
    curve: &dyn Adaptor2dCurve2d,
    (u_min, u_max): (f64, f64),
    extr0: f64,
    index: i32,
    tolerance: f64,
    is_min: bool,
) -> Result<f64, Failure> {
    let sign: f64 = if is_min { 1.0 } else { -1.0 };
    let u_tol: f64 = std_max(
        curve.parametric_resolution(tolerance)?,
        precision::p_confusion(),
    );
    let du: f64 = curve.parameter_span()?;
    let rel_tol: f64 = u_tol / std_max(u_min.abs(), u_max.abs());
    let brent = |first: f64, middle: f64, last: f64| -> Result<Option<f64>, Failure> {
        let mut solver: BrentMinimum = BrentMinimum::with_options(rel_tol, 100, u_tol);
        let mut func = CurveCoordFunc::new(curve, u_min, u_max, index, sign);
        solver.perform(&mut func, first, middle, last);
        func.take_failure()?;
        if solver.is_done() {
            Ok(Some(solver.minimum()?))
        } else {
            Ok(None)
        }
    };
    if u_max - u_min < 0.01 * du
        && let Some(minimum) = brent(u_min, (u_min + u_max) / 2.0, u_max)?
    {
        return Ok(sign * minimum);
    }
    let particles: i32 = 8.max(real_to_int(32.0 * (u_max - u_min) / du));
    let step: f64 = std_min(0.1 * du, (u_max - u_min) / f64::from(particles + 1));
    let seed: f64 = sign * extr0;
    let (mut extr, t): (f64, f64) = legacy_pso(
        curve,
        (u_min, u_max),
        (index, sign),
        (step, seed),
        particles,
    )?;
    let lower: f64 = std_max(t - step, u_min);
    if let Some(minimum) = brent(lower, t, std_min(t + step, u_max))? {
        extr = minimum;
    }
    Ok(sign * extr)
}

// `math_PSO aFinder(&aFunc, Low, Upp, Steps, NbParticles); aFinder.Perform(Steps, extr,
// aT)`: the best value and its parameter.
fn legacy_pso(
    curve: &dyn Adaptor2dCurve2d,
    (u_min, u_max): (f64, f64),
    (index, sign): (i32, f64),
    (step, seed): (f64, f64),
    particles: i32,
) -> Result<(f64, f64), Failure> {
    let lower: Vector<'static> = Vector::filled(1, 1, u_min)?;
    let upper: Vector<'static> = Vector::filled(1, 1, u_max)?;
    let steps: Vector<'static> = Vector::filled(1, 1, step)?;
    let mut point: Vector<'static> = Vector::filled(1, 1, 0.0)?;
    // `extr` enters Perform holding `sign * extr0`; math_PSO always overwrites it.
    let mut value: f64 = seed;
    let mut func = CurveCoordFunc::new(curve, u_min, u_max, index, sign);
    let mut finder = Pso::with_options(&mut func, &lower, &upper, &steps, particles, 100)?;
    finder.perform(&steps, &mut value, &mut point)?;
    drop(finder);
    func.take_failure()?;
    Ok((value, point[1]))
}
