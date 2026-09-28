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

//! Direct math_NewtonMinimum_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case, unused_mut)]
use rustycad_tkmath::math::{
    Matrix, MultipleVarFunction, MultipleVarFunctionWithGradient, MultipleVarFunctionWithHessian,
    NewtonMinimum, Vector,
};
fn assert_near(left: f64, right: f64, tolerance: f64) {
    assert!((left - right).abs() <= tolerance, "{left} != {right}");
}
struct QuadraticBowlWithHessian;
impl MultipleVarFunction for QuadraticBowlWithHessian {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        let dx: f64 = point[1] - 1.0;
        let dy: f64 = point[2] - 2.0;
        *value = dx * dx + 2.0 * dy * dy;
        true
    }
}
impl MultipleVarFunctionWithGradient for QuadraticBowlWithHessian {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        gradient[1] = 2.0 * (point[1] - 1.0);
        gradient[2] = 4.0 * (point[2] - 2.0);
        true
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        self.value(point, value);
        self.gradient(point, gradient);
        true
    }
}
impl MultipleVarFunctionWithHessian for QuadraticBowlWithHessian {
    fn values_hessian(
        &mut self,
        point: &Vector<'_>,
        value: &mut f64,
        gradient: &mut Vector<'_>,
        hessian: &mut Matrix<'_>,
    ) -> bool {
        self.value(point, value);
        self.gradient(point, gradient);
        // Hessian matrix: [[2, 0], [0, 4]]
        hessian[(1, 1)] = 2.0;
        hessian[(1, 2)] = 0.0;
        hessian[(2, 1)] = 0.0;
        hessian[(2, 2)] = 4.0;
        true
    }
}
struct RosenbrockWithHessian;
impl MultipleVarFunction for RosenbrockWithHessian {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        let x: f64 = point[1];
        let y: f64 = point[2];
        let dx: f64 = 1.0 - x;
        let dy: f64 = y - x * x;
        *value = dx * dx + 100.0 * dy * dy;
        true
    }
}
impl MultipleVarFunctionWithGradient for RosenbrockWithHessian {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        let x: f64 = point[1];
        let y: f64 = point[2];
        gradient[1] = -2.0 * (1.0 - x) + 200.0 * (y - x * x) * (-2.0 * x);
        gradient[2] = 200.0 * (y - x * x);
        true
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        self.value(point, value);
        self.gradient(point, gradient);
        true
    }
}
impl MultipleVarFunctionWithHessian for RosenbrockWithHessian {
    fn values_hessian(
        &mut self,
        point: &Vector<'_>,
        value: &mut f64,
        gradient: &mut Vector<'_>,
        hessian: &mut Matrix<'_>,
    ) -> bool {
        let x: f64 = point[1];
        let y: f64 = point[2];

        self.value(point, value);
        self.gradient(point, gradient);

        // Hessian matrix computation
        hessian[(1, 1)] = 2.0 + 1200.0 * x * x - 400.0 * (y - x * x);
        hessian[(1, 2)] = -400.0 * x;
        hessian[(2, 1)] = -400.0 * x;
        hessian[(2, 2)] = 200.0;
        true
    }
}
struct Quadratic3DWithHessian;
impl MultipleVarFunction for Quadratic3DWithHessian {
    fn variables(&self) -> i32 {
        3
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        let dx: f64 = point[1] - 1.0;
        let dy: f64 = point[2] - 2.0;
        let dz: f64 = point[3] - 3.0;
        *value = dx * dx + 2.0 * dy * dy + 3.0 * dz * dz;
        true
    }
}
impl MultipleVarFunctionWithGradient for Quadratic3DWithHessian {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        gradient[1] = 2.0 * (point[1] - 1.0);
        gradient[2] = 4.0 * (point[2] - 2.0);
        gradient[3] = 6.0 * (point[3] - 3.0);
        true
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        self.value(point, value);
        self.gradient(point, gradient);
        true
    }
}
impl MultipleVarFunctionWithHessian for Quadratic3DWithHessian {
    fn values_hessian(
        &mut self,
        point: &Vector<'_>,
        value: &mut f64,
        gradient: &mut Vector<'_>,
        hessian: &mut Matrix<'_>,
    ) -> bool {
        self.value(point, value);
        self.gradient(point, gradient);

        // Diagonal Hessian matrix
        hessian.init(0.0);
        hessian[(1, 1)] = 2.0;
        hessian[(2, 2)] = 4.0;
        hessian[(3, 3)] = 6.0;
        true
    }
}
struct SaddleFunction;
impl MultipleVarFunction for SaddleFunction {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        let x: f64 = point[1];
        let y: f64 = point[2];
        *value = x * x - y * y;
        true
    }
}
impl MultipleVarFunctionWithGradient for SaddleFunction {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        gradient[1] = 2.0 * point[1];
        gradient[2] = -2.0 * point[2];
        true
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        self.value(point, value);
        self.gradient(point, gradient);
        true
    }
}
impl MultipleVarFunctionWithHessian for SaddleFunction {
    fn values_hessian(
        &mut self,
        point: &Vector<'_>,
        value: &mut f64,
        gradient: &mut Vector<'_>,
        hessian: &mut Matrix<'_>,
    ) -> bool {
        self.value(point, value);
        self.gradient(point, gradient);

        // Hessian matrix: [[2, 0], [0, -2]] (indefinite)
        hessian[(1, 1)] = 2.0;
        hessian[(1, 2)] = 0.0;
        hessian[(2, 1)] = 0.0;
        hessian[(2, 2)] = -2.0;
        true
    }
}
include!("math_newton_minimum/suites_0.rs");
include!("math_newton_minimum/suites_1.rs");
include!("math_newton_minimum/suites_2.rs");
include!("math_newton_minimum/suites_3.rs");
include!("math_newton_minimum/suites_4.rs");
include!("math_newton_minimum/suites_5.rs");
