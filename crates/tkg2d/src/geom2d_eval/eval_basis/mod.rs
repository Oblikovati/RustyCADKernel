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
// Rust translation of the anonymous-namespace helpers of ModelingData/TKG2d/Geom2dEval/*.cxx and ModelingData/TKG3d/GeomEval/*.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Basis evaluation shared by the Geom2dEval (TKG2d) and GeomEval (TKG3d) curves.
//!
//! OCCT copies these helpers into each source file (`powInt`, `fallingFactorial`,
//! `evalTrigPairNthDeriv`, `CurveBasisEval`, `PolyEvalData`...), and the GeomEval
//! T-Bezier and AHT-Bezier curves are the Geom2dEval ones with `gp_XYZ` in place of
//! `gp_XY`. The port writes each algorithm once, generic over the coordinate type
//! (`EvalCoord`) and the pole type (`EvalPole`), so both toolkits evaluate with the
//! same operations in the same order.
use rustycad_tkmath::gp::{Point2, Point3, Xy, Xyz};
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

pub mod aht;
pub mod aht_dn;
pub mod sums;
pub mod tbezier;

/// The coordinate triple or pair (`gp_XY`, `gp_XYZ`) that the evaluators accumulate.
///
/// Only the component-wise operators that the native code uses are required; a
/// scalar on the left (`w * xyz`) is written `xyz * w`, which rounds identically
/// because IEEE multiplication is commutative; `fn zero<C: EvalCoord>() -> C { C::default() }`.
pub trait EvalCoord:
    Copy
    + Default
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<f64, Output = Self>
    + Div<f64, Output = Self>
    + AddAssign
    + SubAssign
{
}
impl EvalCoord for Xy {}
impl EvalCoord for Xyz {}

/// A stored pole (`gp_Pnt2d`, `gp_Pnt`) and the coordinates the evaluators read
/// from it (`XY()`, `XYZ()`); `Point3::origin().coord()`.
pub trait EvalPole: Copy {
    /// The coordinate type of the pole.
    type Coord: EvalCoord;
    /// The coordinates of the pole; `pole.coord()`.
    fn coord(&self) -> Self::Coord;
}
impl EvalPole for Point2 {
    type Coord = Xy;
    fn coord(&self) -> Xy {
        self.xy()
    }
}
impl EvalPole for Point3 {
    type Coord = Xyz;
    fn coord(&self) -> Xyz {
        self.xyz()
    }
}

/// `(sin, cos)` of `t + n * Pi / 2` from `sin t` and `cos t`
/// (`trigShiftByQuarterTurns`, `evalTrigPairNthDeriv`, the inline `switch (N & 3)`);
/// `trig_shift(0.0, 1.0, 1) == (1.0, -0.0)`.
pub fn trig_shift(sin: f64, cos: f64, n: i32) -> (f64, f64) {
    match n & 3 {
        0 => (sin, cos),
        1 => (cos, -sin),
        2 => (-sin, -cos),
        _ => (-cos, sin),
    }
}

/// `base` multiplied `exponent` times into 1.0, left to right (`powInt`); a negative
/// exponent gives 1.0 as the native loop does; `pow_int(2.0, 3) == 8.0`.
pub fn pow_int(base: f64, exponent: i32) -> f64 {
    let mut result: f64 = 1.0;
    for _ in 0..exponent {
        result *= base;
    }
    result
}

/// `n * (n - 1) * ... * (n - k + 1)` in doubles (`fallingFactorial`);
/// `falling_factorial(5, 2) == 20.0`.
pub fn falling_factorial(n: i32, k: i32) -> f64 {
    let mut result: f64 = 1.0;
    for i in 0..k {
        result *= f64::from(n - i);
    }
    result
}
