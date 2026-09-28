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
// Rust translation of ModelingData/TKGeomBase/ExtremaPC2d/ExtremaPC2d_DistanceFunction.hxx and
// ExtremaPC/ExtremaPC_DistanceFunction.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The normalized stationarity function `F(u) = (C(u) - P) . C'(u) / |C'(u)|` of the
//! point-curve distance, whose roots are the extrema of `|C(u) - P|^2`.
use super::space::{PcPoint, PcSpace, PcVector};
use crate::failure_capture::FailureCapture;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp;
use rustycad_tkmath::math::{Function, FunctionWithDerivative};

/// The point-curve distance function (`ExtremaPC2d_DistanceFunction`,
/// `ExtremaPC_DistanceFunction`), borrowing the curve adaptor as natively.
///
/// A native evaluation always returns true; an adaptor exception becomes the `Err`
/// of the Rust members. At a singular parameter (`|C'| <= gp::Resolution()`) the
/// function and its derivative are 0;
/// `DistanceFunction::new(&adaptor, Point2::new(1.0, 0.0)).value(0.5)?`.
pub struct PcDistanceFunction<'a, S: PcSpace> {
    curve: &'a S::Curve,
    point: S::Point,
}

impl<'a, S: PcSpace> PcDistanceFunction<'a, S> {
    /// The function of `point` against `curve` (the constructor);
    /// `DistanceFunction::new(&adaptor, point)`.
    pub fn new(curve: &'a S::Curve, point: S::Point) -> Self {
        Self { curve, point }
    }
    /// `F(u)` (`Value`); `function.value(0.5)?`.
    pub fn value(&self, u: f64) -> Result<f64, Failure> {
        let (curve_point, d1) = S::d1(self.curve, u)?;
        let vector = self.point.vector_to(&curve_point);
        let speed: f64 = d1.length();
        if speed <= gp::resolution() {
            // The normalized function has no defined direction at a singular parameter.
            // Its continuous one-sided signs are used later for extrema classification.
            return Ok(0.0);
        }
        Ok(vector.dot_with(&d1) / speed)
    }
    /// `F(u)` and `F'(u)` (`Values`); `let (f, df) = function.values(0.5)?;`.
    pub fn values(&self, u: f64) -> Result<(f64, f64), Failure> {
        let (curve_point, d1, d2) = S::d2(self.curve, u)?;
        let vector = self.point.vector_to(&curve_point);
        let speed: f64 = d1.length();
        if speed <= gp::resolution() {
            return Ok((0.0, 0.0));
        }
        let raw_f: f64 = vector.dot_with(&d1);
        let f: f64 = raw_f / speed;
        let df: f64 = (d1.dot_with(&d1) + vector.dot_with(&d2)) / speed
            - raw_f * d1.dot_with(&d2) / (speed * speed * speed);
        Ok((f, df))
    }
    /// `F'(u)` (`Derivative`); `function.derivative(0.5)?`.
    pub fn derivative(&self, u: f64) -> Result<f64, Failure> {
        Ok(self.values(u)?.1)
    }
    /// The first curve parameter (`FirstParameter`); `function.first_parameter()?`.
    pub fn first_parameter(&self) -> Result<f64, Failure> {
        S::first_parameter(self.curve)
    }
    /// The last curve parameter (`LastParameter`); `function.last_parameter()?`.
    pub fn last_parameter(&self) -> Result<f64, Failure> {
        S::last_parameter(self.curve)
    }
    /// The query point (`Point`); `function.point()`.
    pub fn point(&self) -> &S::Point {
        &self.point
    }
    /// The curve (`Curve`); `function.curve()`.
    pub fn curve(&self) -> &'a S::Curve {
        self.curve
    }
}

/// The distance function as a MathRoot functor: an adaptor failure is recorded and the
/// evaluation answers false, so the caller can report it as the native `catch` does.
pub(crate) struct CapturingDistance<'f, 'a, S: PcSpace> {
    pub function: &'f PcDistanceFunction<'a, S>,
    pub capture: FailureCapture,
}

impl<'f, 'a, S: PcSpace> CapturingDistance<'f, 'a, S> {
    pub fn new(function: &'f PcDistanceFunction<'a, S>) -> Self {
        Self {
            function,
            capture: FailureCapture::default(),
        }
    }
}

impl<S: PcSpace> Function for CapturingDistance<'_, '_, S> {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        let Some(value) = self.capture.keep(self.function.value(x)) else {
            return false;
        };
        *result = value;
        true
    }
}

impl<S: PcSpace> FunctionWithDerivative for CapturingDistance<'_, '_, S> {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        let Some(value) = self.capture.keep(self.function.derivative(x)) else {
            return false;
        };
        *derivative = value;
        true
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        let Some((f, df)) = self.capture.keep(self.function.values(x)) else {
            return false;
        };
        (*value, *derivative) = (f, df);
        true
    }
}
