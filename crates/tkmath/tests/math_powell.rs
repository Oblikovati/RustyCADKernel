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

//! Direct math_Powell_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case, unused_mut)]
use rustycad_tkmath::math::{Matrix, MultipleVarFunction, Powell, Vector};
fn assert_near(left: f64, right: f64, tolerance: f64) {
    assert!((left - right).abs() <= tolerance, "{left} != {right}");
}
struct QuadraticFunction;
impl MultipleVarFunction for QuadraticFunction {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        let x: f64 = point[1] - 1.0;
        let y: f64 = point[2] - 2.0;
        *value = x * x + y * y;
        true
    }
}
struct RosenbrockFunction;
impl MultipleVarFunction for RosenbrockFunction {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        let x: f64 = point[1];
        let y: f64 = point[2];
        let term1: f64 = y - x * x;
        let term2: f64 = 1.0 - x;
        *value = 100.0 * term1 * term1 + term2 * term2;
        true
    }
}
struct Simple1DFunction;
impl MultipleVarFunction for Simple1DFunction {
    fn variables(&self) -> i32 {
        1
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        let x: f64 = point[1] - 3.0;
        *value = x * x;
        true
    }
}
struct MultiDimensionalQuadratic(i32);
impl MultipleVarFunction for MultiDimensionalQuadratic {
    fn variables(&self) -> i32 {
        self.0
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        *value = 0.0;
        for i in 1..=self.0 {
            let diff: f64 = point[i] - f64::from(i);
            *value += diff * diff;
        }
        true
    }
}
include!("math_powell/suites_0.rs");
include!("math_powell/suites_1.rs");
include!("math_powell/suites_2.rs");
include!("math_powell/suites_3.rs");
