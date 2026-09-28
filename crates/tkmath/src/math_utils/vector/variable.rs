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

use super::*;
/// Native sphere objective with adapter dimension metadata; `Sphere::new(3)`.
#[derive(Clone)]
pub struct Sphere {
    dimension: i32,
}
impl Sphere {
    /// Declares the objective dimension; `Sphere::new(3)`.
    pub fn new(dimension: i32) -> Self {
        Self { dimension }
    }
}
impl MultipleVarFunction for Sphere {
    fn variables(&self) -> i32 {
        self.dimension
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        *value = 0.0;
        for x in point.values() {
            *value += x * x;
        }
        true
    }
}
impl MultipleVarFunctionWithGradient for Sphere {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        for (index, x) in point.values().iter().enumerate() {
            *gradient.change_at(index).unwrap() = 2.0 * x;
        }
        true
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        self.value(point, value) && self.gradient(point, gradient)
    }
}
/// Native Rastrigin objective; `Rastrigin::new(3,10.0)`.
#[derive(Clone)]
pub struct Rastrigin {
    dimension: i32,
    a: f64,
}
impl Rastrigin {
    /// Declares dimension and native amplitude; `Rastrigin::new(3,10.0)`.
    pub fn new(dimension: i32, a: f64) -> Self {
        Self { dimension, a }
    }
}
impl MultipleVarFunction for Rastrigin {
    fn variables(&self) -> i32 {
        self.dimension
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        *value = self.a * f64::from(point.length());
        for x in point.values() {
            *value += x * x - self.a * (TWO_PI * x).cos();
        }
        true
    }
}
impl MultipleVarFunctionWithGradient for Rastrigin {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        for (index, x) in point.values().iter().enumerate() {
            *gradient.change_at(index).unwrap() = 2.0 * x + self.a * TWO_PI * (TWO_PI * x).sin();
        }
        true
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        self.value(point, value) && self.gradient(point, gradient)
    }
}
/// Native Ackley objective rejects empty and nonfinite inputs; `Ackley::new(3,20.0,0.2,TWO_PI)`.
#[derive(Clone)]
pub struct Ackley {
    dimension: i32,
    a: f64,
    b: f64,
    c: f64,
}
impl Ackley {
    /// Declares dimension and native parameters; `Ackley::new(3,20.0,0.2,TWO_PI)`.
    pub fn new(dimension: i32, a: f64, b: f64, c: f64) -> Self {
        Self { dimension, a, b, c }
    }
    fn sums(&self, point: &Vector<'_>) -> Option<(f64, f64)> {
        let (mut squared, mut cosine): (f64, f64) = (0.0, 0.0);
        for x in point.values() {
            if !x.is_finite() || !(self.c * x).is_finite() {
                return None;
            }
            squared += x * x;
            cosine += (self.c * x).cos();
            if !squared.is_finite() || !cosine.is_finite() {
                return None;
            }
        }
        Some((squared, cosine))
    }
}
impl MultipleVarFunction for Ackley {
    fn variables(&self) -> i32 {
        self.dimension
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        if point.length() == 0 || !self.a.is_finite() || !self.b.is_finite() || !self.c.is_finite()
        {
            return false;
        }
        let Some((squared, cosine)) = self.sums(point) else {
            return false;
        };
        let divisor: f64 = f64::from(point.length());
        *value = -self.a * (-self.b * (squared / divisor).sqrt()).exp() - (cosine / divisor).exp()
            + self.a
            + EULER_NUMBER;
        value.is_finite()
    }
}
