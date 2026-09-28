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

//! Direct math_BissecNewton_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case, unused_mut)]
use rustycad_tkmath::math::{BissecNewton, Function, FunctionWithDerivative};
fn assert_near(left: f64, right: f64, tolerance: f64) {
    assert!((left - right).abs() <= tolerance, "{left} != {right}");
}
struct QuadraticWithDerivative;
impl Function for QuadraticWithDerivative {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        *result = (x - 2.0) * (x - 2.0) - 1.0;
        true
    }
}
impl FunctionWithDerivative for QuadraticWithDerivative {
    fn derivative(&mut self, x: f64, result: &mut f64) -> bool {
        *result = 2.0 * (x - 2.0);
        true
    }
    fn values(&mut self, x: f64, result: &mut f64, derivative: &mut f64) -> bool {
        *result = (x - 2.0) * (x - 2.0) - 1.0;
        *derivative = 2.0 * (x - 2.0);
        true
    }
}
struct CubicWithDerivative;
impl Function for CubicWithDerivative {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        *result = x * x * x - x - 2.0;
        true
    }
}
impl FunctionWithDerivative for CubicWithDerivative {
    fn derivative(&mut self, x: f64, result: &mut f64) -> bool {
        *result = 3.0 * x * x - 1.0;
        true
    }
    fn values(&mut self, x: f64, result: &mut f64, derivative: &mut f64) -> bool {
        *result = x * x * x - x - 2.0;
        *derivative = 3.0 * x * x - 1.0;
        true
    }
}
struct SineWithDerivative;
impl Function for SineWithDerivative {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        *result = x.sin();
        true
    }
}
impl FunctionWithDerivative for SineWithDerivative {
    fn derivative(&mut self, x: f64, result: &mut f64) -> bool {
        *result = x.cos();
        true
    }
    fn values(&mut self, x: f64, result: &mut f64, derivative: &mut f64) -> bool {
        *result = x.sin();
        *derivative = x.cos();
        true
    }
}
struct CubicZeroDerivative;
impl Function for CubicZeroDerivative {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        *result = x * x * x;
        true
    }
}
impl FunctionWithDerivative for CubicZeroDerivative {
    fn derivative(&mut self, x: f64, result: &mut f64) -> bool {
        *result = 3.0 * x * x;
        true
    }
    fn values(&mut self, x: f64, result: &mut f64, derivative: &mut f64) -> bool {
        *result = x * x * x;
        *derivative = 3.0 * x * x;
        true
    }
}
include!("math_bissec_newton/suites_0.rs");
include!("math_bissec_newton/suites_1.rs");
include!("math_bissec_newton/suites_2.rs");
include!("math_bissec_newton/suites_3.rs");
include!("math_bissec_newton/suites_4.rs");
