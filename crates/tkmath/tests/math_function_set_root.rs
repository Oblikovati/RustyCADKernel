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

//! Direct math_FunctionSetRoot_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case, unused_mut, unused_variables)]
use rustycad_tkmath::math::{
    FunctionSet, FunctionSetRoot, FunctionSetWithDerivatives, Matrix, Vector,
};
const TOLERANCE: f64 = 1e-6;
fn assert_near(left: f64, right: f64, tolerance: f64) {
    assert!((left - right).abs() <= tolerance, "{left} != {right}");
}
struct LinearSystem2D;
impl FunctionSet for LinearSystem2D {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        2
    }
    fn value(&mut self, X: &Vector<'_>, F: &mut Vector<'_>) -> bool {
        F[1] = 2.0 * X[1] + X[2] - 5.0; // 2x + y - 5 = 0
        F[2] = X[1] + 2.0 * X[2] - 4.0; // x + 2y - 4 = 0
        true
    }
}
impl FunctionSetWithDerivatives for LinearSystem2D {
    fn derivatives(&mut self, X: &Vector<'_>, D: &mut Matrix<'_>) -> bool {
        D[(1, 1)] = 2.0;
        D[(1, 2)] = 1.0;
        D[(2, 1)] = 1.0;
        D[(2, 2)] = 2.0;
        true
    }
    fn values(&mut self, X: &Vector<'_>, F: &mut Vector<'_>, D: &mut Matrix<'_>) -> bool {
        self.value(X, F);
        self.derivatives(X, D);
        true
    }
}
struct NonlinearSystem;
impl FunctionSet for NonlinearSystem {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        2
    }
    fn value(&mut self, X: &Vector<'_>, F: &mut Vector<'_>) -> bool {
        F[1] = X[1] * X[1] + X[2] * X[2] - 5.0; // x^2 + y^2 - 5 = 0
        F[2] = X[1] * X[2] - 2.0; // xy - 2 = 0
        true
    }
}
impl FunctionSetWithDerivatives for NonlinearSystem {
    fn derivatives(&mut self, X: &Vector<'_>, D: &mut Matrix<'_>) -> bool {
        D[(1, 1)] = 2.0 * X[1];
        D[(1, 2)] = 2.0 * X[2];
        D[(2, 1)] = X[2];
        D[(2, 2)] = X[1];
        true
    }
    fn values(&mut self, X: &Vector<'_>, F: &mut Vector<'_>, D: &mut Matrix<'_>) -> bool {
        self.value(X, F);
        self.derivatives(X, D);
        true
    }
}
struct SingleVariableSystem;
impl FunctionSet for SingleVariableSystem {
    fn variables(&self) -> i32 {
        1
    }
    fn equations(&self) -> i32 {
        1
    }
    fn value(&mut self, X: &Vector<'_>, F: &mut Vector<'_>) -> bool {
        F[1] = X[1] * X[1] - 4.0;
        true
    }
}
impl FunctionSetWithDerivatives for SingleVariableSystem {
    fn derivatives(&mut self, X: &Vector<'_>, D: &mut Matrix<'_>) -> bool {
        D[(1, 1)] = 2.0 * X[1];
        true
    }
    fn values(&mut self, X: &Vector<'_>, F: &mut Vector<'_>, D: &mut Matrix<'_>) -> bool {
        self.value(X, F);
        self.derivatives(X, D);
        true
    }
}
struct OverdeterminedSystem;
impl FunctionSet for OverdeterminedSystem {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        3
    }
    fn value(&mut self, X: &Vector<'_>, F: &mut Vector<'_>) -> bool {
        F[1] = X[1] + X[2] - 3.0; // x + y - 3 = 0
        F[2] = X[1] - X[2] - 1.0; // x - y - 1 = 0
        F[3] = 2.0 * X[1] - 4.0; // 2x - 4 = 0
        true
    }
}
impl FunctionSetWithDerivatives for OverdeterminedSystem {
    fn derivatives(&mut self, X: &Vector<'_>, D: &mut Matrix<'_>) -> bool {
        D[(1, 1)] = 1.0;
        D[(1, 2)] = 1.0;
        D[(2, 1)] = 1.0;
        D[(2, 2)] = -1.0;
        D[(3, 1)] = 2.0;
        D[(3, 2)] = 0.0;
        true
    }
    fn values(&mut self, X: &Vector<'_>, F: &mut Vector<'_>, D: &mut Matrix<'_>) -> bool {
        self.value(X, F);
        self.derivatives(X, D);
        true
    }
}
struct ThreeVariableSystem;
impl FunctionSet for ThreeVariableSystem {
    fn variables(&self) -> i32 {
        3
    }
    fn equations(&self) -> i32 {
        3
    }
    fn value(&mut self, X: &Vector<'_>, F: &mut Vector<'_>) -> bool {
        F[1] = X[1] + X[2] + X[3] - 6.0; // x + y + z - 6 = 0
        F[2] = 2.0 * X[1] - X[2] - 2.0; // 2x - y - 2 = 0
        F[3] = X[3] - 2.0; // z - 2 = 0
        true
    }
}
impl FunctionSetWithDerivatives for ThreeVariableSystem {
    fn derivatives(&mut self, X: &Vector<'_>, D: &mut Matrix<'_>) -> bool {
        D[(1, 1)] = 1.0;
        D[(1, 2)] = 1.0;
        D[(1, 3)] = 1.0;
        D[(2, 1)] = 2.0;
        D[(2, 2)] = -1.0;
        D[(2, 3)] = 0.0;
        D[(3, 1)] = 0.0;
        D[(3, 2)] = 0.0;
        D[(3, 3)] = 1.0;
        true
    }
    fn values(&mut self, X: &Vector<'_>, F: &mut Vector<'_>, D: &mut Matrix<'_>) -> bool {
        self.value(X, F);
        self.derivatives(X, D);
        true
    }
}
include!("math_function_set_root/suites_0.rs");
include!("math_function_set_root/suites_1.rs");
include!("math_function_set_root/suites_2.rs");
include!("math_function_set_root/suites_3.rs");
include!("math_function_set_root/suites_4.rs");
include!("math_function_set_root/suites_5.rs");
