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

//! Direct math_Integration_Test.cxx translation at OCCT 3d097a0.
#![allow(non_snake_case)]
use rustycad_tkmath::math::{
    Function, GaussSingleIntegration as Gauss, KronrodSingleIntegration as Kronrod,
};
use std::f64::consts::PI;
fn near(a: f64, b: f64, tolerance: f64) {
    assert!((a - b).abs() <= tolerance, "{a} != {b}");
}
struct ConstantFunction(f64);
impl Function for ConstantFunction {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        let _ = x;
        *value = self.0;
        true
    }
}
struct LinearFunction(f64, f64);
impl Function for LinearFunction {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        let _ = x;
        *value = self.0 * x + self.1;
        true
    }
}
struct QuadraticFunction(f64, f64, f64);
impl Function for QuadraticFunction {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        let _ = x;
        *value = self.0 * x * x + self.1 * x + self.2;
        true
    }
}
struct PowerFunction(i32);
impl Function for PowerFunction {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        let _ = x;
        *value = x.powf(f64::from(self.0));
        true
    }
}
struct SineFunction;
impl Function for SineFunction {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        let _ = x;
        *value = x.sin();
        true
    }
}
struct ExponentialFunction;
impl Function for ExponentialFunction {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        let _ = x;
        *value = x.exp();
        true
    }
}
#[test]
fn MathIntegrationTest__GaussConstantFunction() {
    let mut f = ConstantFunction(5.0);
    let s = Gauss::new(&mut f, 0.0, 2.0, 4);
    assert!(s.is_done());
    near(s.value().unwrap(), 5.0 * (2.0 - 0.0), 1e-12);
}
#[test]
fn MathIntegrationTest__GaussLinearFunction() {
    let mut f = LinearFunction(2.0, 3.0);
    let s = Gauss::new(&mut f, 1.0, 4.0, 2);
    assert!(s.is_done());
    near(s.value().unwrap(), 24.0, 1e-12);
}
#[test]
fn MathIntegrationTest__GaussQuadraticFunction() {
    let mut f = QuadraticFunction(1.0, -2.0, 1.0);
    let s = Gauss::new(&mut f, 0.0, 2.0, 3);
    assert!(s.is_done());
    near(s.value().unwrap(), 2.0 / 3.0, 1e-12);
}
#[test]
fn MathIntegrationTest__GaussSineFunction() {
    let mut f = SineFunction;
    let s = Gauss::new(&mut f, 0.0, PI, 10);
    assert!(s.is_done());
    near(s.value().unwrap(), 2.0, 1e-10);
}
#[test]
fn MathIntegrationTest__GaussExponentialFunction() {
    let mut f = ExponentialFunction;
    let s = Gauss::new(&mut f, 0.0, 1.0, 15);
    assert!(s.is_done());
    near(s.value().unwrap(), 1.0_f64.exp() - 1.0, 1e-8);
}
#[test]
fn MathIntegrationTest__GaussDifferentOrders() {
    let mut f = QuadraticFunction(1.0, 0.0, 0.0);
    for order in [2, 3, 5, 10, 20] {
        let s = Gauss::new(&mut f, 0.0, 1.0, order);
        assert!(s.is_done());
        near(
            s.value().unwrap(),
            1.0 / 3.0,
            if order >= 3 { 1e-12 } else { 1e-6 },
        );
    }
}
#[test]
fn MathIntegrationTest__GaussWithTolerance() {
    let mut f = QuadraticFunction(1.0, 0.0, 0.0);
    let tolerance: f64 = 1e-10;
    let s = Gauss::with_tolerance(&mut f, 0.0, 1.0, 5, tolerance);
    assert!(s.is_done());
    near(s.value().unwrap(), 1.0 / 3.0, tolerance * 10.0);
}
#[test]
fn MathIntegrationTest__GaussNegativeInterval() {
    let mut f = LinearFunction(1.0, 0.0);
    let s = Gauss::new(&mut f, -2.0, 2.0, 3);
    assert!(s.is_done());
    near(s.value().unwrap(), 0.0, 1e-12);
}
#[test]
fn MathIntegrationTest__KronrodConstantFunction() {
    let mut f = ConstantFunction(3.0);
    let s = Kronrod::new(&mut f, 1.0, 5.0, 15);
    assert!(s.is_done());
    near(s.value().unwrap(), 3.0 * (5.0 - 1.0), 1e-12);
}
#[test]
fn MathIntegrationTest__KronrodQuadraticFunction() {
    let mut f = QuadraticFunction(2.0, -1.0, 3.0);
    let s = Kronrod::new(&mut f, 0.0, 1.0, 15);
    assert!(s.is_done());
    near(s.value().unwrap(), 19.0 / 6.0, 1e-10);
}
#[test]
fn MathIntegrationTest__KronrodSineFunction() {
    let mut f = SineFunction;
    let s = Kronrod::new(&mut f, 0.0, PI / 2.0, 21);
    assert!(s.is_done());
    near(s.value().unwrap(), 1.0, 1e-12);
}
#[test]
fn MathIntegrationTest__GaussVsKronrodComparison() {
    let mut f = PowerFunction(4);
    let gauss = Gauss::new(&mut f, 0.0, 2.0, 10);
    assert!(gauss.is_done());
    let kronrod = Kronrod::new(&mut f, 0.0, 2.0, 21);
    assert!(kronrod.is_done());
    near(gauss.value().unwrap(), 32.0 / 5.0, 1e-10);
    near(kronrod.value().unwrap(), 32.0 / 5.0, 1e-10);
    near(gauss.value().unwrap(), kronrod.value().unwrap(), 1e-8);
}
#[test]
fn MathIntegrationTest__DefaultConstructorAndPerform() {
    let _unused = Gauss::default();
    let mut f = LinearFunction(1.0, 2.0);
    let s = Gauss::new(&mut f, 0.0, 3.0, 5);
    assert!(s.is_done());
    near(s.value().unwrap(), 10.5, 1e-12);
}
#[test]
fn MathIntegrationTest__ZeroLengthInterval() {
    let mut f = ConstantFunction(1.0);
    let s = Gauss::new(&mut f, 5.0, 5.0, 5);
    if s.is_done() {
        near(s.value().unwrap(), 0.0, 1e-7);
    }
}
#[test]
fn MathIntegrationTest__ReverseInterval() {
    let mut f = LinearFunction(1.0, 0.0);
    let s = Gauss::new(&mut f, 2.0, 0.0, 5);
    if s.is_done() {
        near(s.value().unwrap(), -2.0, 1e-12);
    }
}
#[test]
fn MathIntegrationTest__HighOrderIntegration() {
    let mut f = PowerFunction(10);
    let s = Gauss::new(&mut f, 0.0, 1.0, 30);
    assert!(s.is_done());
    near(s.value().unwrap(), 1.0 / 11.0, 1e-12);
}
