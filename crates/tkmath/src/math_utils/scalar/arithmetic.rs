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
/// Evaluates the inner callback before the outer; `Composite::new(outer,inner)`.
#[derive(Clone)]
pub struct Composite<F, G> {
    outer: F,
    inner: G,
}
impl<F: Function, G: Function> Composite<F, G> {
    /// Owns both functions; `Composite::new(outer,inner)`.
    pub fn new(outer: F, inner: G) -> Self {
        Self { outer, inner }
    }
}
impl<F: Function, G: Function> Function for Composite<F, G> {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        let mut inner: f64 = 0.0;
        self.inner.value(x, &mut inner) && self.outer.value(inner, value)
    }
}
/// Deduces composition types; `make_composite(outer,inner)`.
pub fn make_composite<F: Function, G: Function>(outer: F, inner: G) -> Composite<F, G> {
    Composite::new(outer, inner)
}
fn pair<F: Function, G: Function>(first: &mut F, second: &mut G, x: f64) -> Option<(f64, f64)> {
    let (mut a, mut b): (f64, f64) = (0.0, 0.0);
    if !first.value(x, &mut a) || !second.value(x, &mut b) {
        return None;
    }
    Some((a, b))
}

/// Native sum callback composition; `Sum::new(first,second)`.
#[derive(Clone)]
pub struct Sum<F, G> {
    first: F,
    second: G,
}
impl<F: Function, G: Function> Sum<F, G> {
    /// Retains left-before-right callback order; `Sum::new(first,second)`.
    pub fn new(first: F, second: G) -> Self {
        Self { first, second }
    }
}
impl<F: Function, G: Function> Function for Sum<F, G> {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        let Some((a, b)) = pair(&mut self.first, &mut self.second, x) else {
            return false;
        };

        *value = a + b;
        true
    }
}
/// Deduces sum callback types; `make_sum(first,second)`.
pub fn make_sum<F: Function, G: Function>(first: F, second: G) -> Sum<F, G> {
    Sum::new(first, second)
}

/// Native difference callback composition; `Difference::new(first,second)`.
#[derive(Clone)]
pub struct Difference<F, G> {
    first: F,
    second: G,
}
impl<F: Function, G: Function> Difference<F, G> {
    /// Retains left-before-right callback order; `Difference::new(first,second)`.
    pub fn new(first: F, second: G) -> Self {
        Self { first, second }
    }
}
impl<F: Function, G: Function> Function for Difference<F, G> {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        let Some((a, b)) = pair(&mut self.first, &mut self.second, x) else {
            return false;
        };

        *value = a - b;
        true
    }
}
/// Deduces difference callback types; `make_difference(first,second)`.
pub fn make_difference<F: Function, G: Function>(first: F, second: G) -> Difference<F, G> {
    Difference::new(first, second)
}

/// Native product callback composition; `Product::new(first,second)`.
#[derive(Clone)]
pub struct Product<F, G> {
    first: F,
    second: G,
}
impl<F: Function, G: Function> Product<F, G> {
    /// Retains left-before-right callback order; `Product::new(first,second)`.
    pub fn new(first: F, second: G) -> Self {
        Self { first, second }
    }
}
impl<F: Function, G: Function> Function for Product<F, G> {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        let Some((a, b)) = pair(&mut self.first, &mut self.second, x) else {
            return false;
        };

        *value = a * b;
        true
    }
}
/// Deduces product callback types; `make_product(first,second)`.
pub fn make_product<F: Function, G: Function>(first: F, second: G) -> Product<F, G> {
    Product::new(first, second)
}

/// Native quotient callback composition; `Quotient::new(first,second)`.
#[derive(Clone)]
pub struct Quotient<F, G> {
    first: F,
    second: G,
}
impl<F: Function, G: Function> Quotient<F, G> {
    /// Retains left-before-right callback order; `Quotient::new(first,second)`.
    pub fn new(first: F, second: G) -> Self {
        Self { first, second }
    }
}
impl<F: Function, G: Function> Function for Quotient<F, G> {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        let Some((a, b)) = pair(&mut self.first, &mut self.second, x) else {
            return false;
        };
        if !a.is_finite() || !b.is_finite() || b == 0.0 {
            return false;
        }
        *value = a / b;
        value.is_finite()
    }
}
/// Deduces quotient callback types; `make_quotient(first,second)`.
pub fn make_quotient<F: Function, G: Function>(first: F, second: G) -> Quotient<F, G> {
    Quotient::new(first, second)
}

/// Native scaled transformation; `Scaled::new(function,amount)`.
#[derive(Clone)]
pub struct Scaled<F> {
    function: F,
    amount: f64,
}
impl<F: Function> Scaled<F> {
    /// Owns the wrapped function; `Scaled::new(function,amount)`.
    pub fn new(function: F, amount: f64) -> Self {
        Self { function, amount }
    }
}
impl<F: Function> Function for Scaled<F> {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        let mut a: f64 = 0.0;
        if !self.function.value(x, &mut a) {
            return false;
        }
        *value = self.amount * a;
        true
    }
}
/// Deduces the wrapped function type; `make_scaled(function,amount)`.
pub fn make_scaled<F: Function>(function: F, amount: f64) -> Scaled<F> {
    Scaled::new(function, amount)
}

/// Native shifted transformation; `Shifted::new(function,amount)`.
#[derive(Clone)]
pub struct Shifted<F> {
    function: F,
    amount: f64,
}
impl<F: Function> Shifted<F> {
    /// Owns the wrapped function; `Shifted::new(function,amount)`.
    pub fn new(function: F, amount: f64) -> Self {
        Self { function, amount }
    }
}
impl<F: Function> Function for Shifted<F> {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        let mut a: f64 = 0.0;
        if !self.function.value(x, &mut a) {
            return false;
        }
        *value = a + self.amount;
        true
    }
}
/// Deduces the wrapped function type; `make_shifted(function,amount)`.
pub fn make_shifted<F: Function>(function: F, amount: f64) -> Shifted<F> {
    Shifted::new(function, amount)
}

/// Native negated transformation; `Negated::new(function)`.
#[derive(Clone)]
pub struct Negated<F> {
    function: F,
}
impl<F: Function> Negated<F> {
    /// Owns the wrapped function; `Negated::new(function)`.
    pub fn new(function: F) -> Self {
        Self { function }
    }
}
impl<F: Function> Function for Negated<F> {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        let mut a: f64 = 0.0;
        if !self.function.value(x, &mut a) {
            return false;
        }
        *value = -a;
        true
    }
}
/// Deduces the wrapped function type; `make_negated(function)`.
pub fn make_negated<F: Function>(function: F) -> Negated<F> {
    Negated::new(function)
}
