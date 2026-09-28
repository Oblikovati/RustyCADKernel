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

//! Trigonometric equations preserving pinned MathRoot_Trig.hxx branches.
use crate::math_poly::{PolyResult, quartic};
use crate::math_utils::Status;
use std::f64::consts::{PI, TAU};
mod refine;
mod special;
use special::special_roots;
/// Native four-root storage; `TrigResult::default()`.
#[derive(Default)]
pub struct TrigResult {
    /// Native completion status; `result.status`.
    pub status: Status,
    /// Sorted representatives in the working period; `result.roots`.
    pub roots: [f64; 4],
    /// Number of stored representatives; `result.nb_roots`.
    pub nb_roots: usize,
    /// Identity equation flag; `result.infinite_roots`.
    pub infinite_roots: bool,
}
impl TrigResult {
    /// Reports native success; `result.is_done()`.
    pub fn is_done(&self) -> bool {
        self.status == Status::Ok
    }
    fn failed(status: Status) -> Self {
        Self {
            status,
            ..Default::default()
        }
    }
    fn insert(&mut self, root: f64, tolerance: f64) {
        let mut position: usize = 0;
        while position < self.nb_roots && self.roots[position] < root {
            if (root - self.roots[position]).abs() <= tolerance {
                return;
            }
            position += 1;
        }
        if (position < self.nb_roots && (root - self.roots[position]).abs() <= tolerance)
            || self.nb_roots == 4
        {
            return;
        }
        for i in (position + 1..=self.nb_roots).rev() {
            self.roots[i] = self.roots[i - 1];
        }
        self.roots[position] = root;
        self.nb_roots += 1;
    }
}
impl From<&TrigResult> for bool {
    fn from(result: &TrigResult) -> bool {
        result.is_done()
    }
}
struct Interval {
    lower: f64,
    upper: f64,
    delta: f64,
    tolerance: f64,
}
impl Interval {
    fn new(lower: f64, upper: f64, epsilon: f64) -> Self {
        let (lower, delta): (f64, f64) = if lower <= -1e100 && upper >= 1e100 {
            (0.0, TAU)
        } else if upper >= 1e100 {
            (lower, TAU)
        } else if lower <= -1e100 {
            (upper - TAU, TAU)
        } else {
            (lower, (upper - lower).min(TAU))
        };
        Self {
            lower,
            upper: lower + delta,
            delta,
            tolerance: epsilon.max(f64::EPSILON * delta.abs()),
        }
    }
    fn mapped(&self, mut root: f64) -> Option<f64> {
        root += ((self.lower - root) / TAU).floor() * TAU;
        if root < self.lower - self.tolerance {
            root += TAU;
        }
        if root < self.lower {
            root = self.lower;
        }
        if root > self.upper + self.tolerance {
            return None;
        }
        if root > self.upper {
            root = self.upper;
        }
        let mut count: usize = 0;
        if !super::utils::compute_periodic_root_count(
            root,
            self.upper,
            TAU,
            self.tolerance,
            false,
            &mut count,
        ) || count == 0
        {
            return None;
        }
        Some(root)
    }
    fn add(&self, out: &mut TrigResult, root: f64) {
        if let Some(root) = self.mapped(root) {
            out.insert(root, self.tolerance);
        }
    }
    fn add_all(&self, out: &mut TrigResult, roots: &[f64]) {
        for &root in roots {
            self.add(out, root);
        }
    }
}
fn valid_input(c: &[f64; 5], lower: f64, upper: f64, epsilon: f64) -> bool {
    c.iter().all(|x| x.is_finite())
        && lower.is_finite()
        && upper.is_finite()
        && lower < upper
        && lower < 1e100
        && upper > -1e100
        && epsilon.is_finite()
        && epsilon > 0.0
}
fn normalized(mut c: [f64; 5]) -> [f64; 5] {
    let scale: f64 = c.iter().fold(0.0_f64, |s, x| s.max(x.abs()));
    if scale > 0.0 {
        for x in &mut c {
            *x /= scale;
        }
    }
    c
}
fn polynomial(c: &[f64; 5]) -> PolyResult {
    let [a, b, c, d, e] = *c;
    quartic(
        a - c + e,
        2.0 * d - 4.0 * b,
        2.0 * e - 2.0 * a,
        4.0 * b + 2.0 * d,
        a + c + e,
    )
}
fn general_roots(c: &[f64; 5], interval: &Interval, epsilon: f64, out: &mut TrigResult) {
    let p: PolyResult = polynomial(c);
    if !p.is_done() {
        if p.status == Status::InfiniteSolutions {
            out.infinite_roots = true;
        } else {
            out.status = p.status;
        }
        return;
    }
    let mut roots: [f64; 4] = [0.0; 4];
    let count: usize = p.nb_roots.min(4);
    roots[..count].copy_from_slice(&p.roots[..count]);
    roots[..count].sort_by(|a, b| a.partial_cmp(b).unwrap());
    for &root in &roots[..count] {
        refine::add_polynomial_root(c, interval, epsilon, out, root);
    }
    if out.nb_roots < 4 && (c[0] - c[2] + c[4]).abs() <= epsilon {
        interval.add(out, PI);
    }
}
/// Solves the native quadratic trigonometric form; `trigonometric([0.0,0.0,0.0,1.0,0.0],0.0,std::f64::consts::TAU,1.5e-12)`.
pub fn trigonometric(coefficients: [f64; 5], lower: f64, upper: f64, epsilon: f64) -> TrigResult {
    if !valid_input(&coefficients, lower, upper, epsilon) {
        return TrigResult::failed(Status::InvalidInput);
    }
    let coefficients: [f64; 5] = normalized(coefficients);
    let interval: Interval = Interval::new(lower, upper, epsilon);
    let mut out: TrigResult = TrigResult::failed(Status::Ok);
    if special_roots(&coefficients, &interval, epsilon, &mut out) {
        return out;
    }
    general_roots(&coefficients, &interval, epsilon, &mut out);
    out
}
/// Solves a sine equation; `trigonometric_linear(1.0,0.0,0.0,std::f64::consts::TAU)`.
pub fn trigonometric_linear(d: f64, e: f64, lower: f64, upper: f64) -> TrigResult {
    trigonometric([0.0, 0.0, 0.0, d, e], lower, upper, 1.5e-12)
}
/// Solves a linear sine/cosine combination; `trigonometric_cde(1.0,1.0,0.0,0.0,std::f64::consts::TAU)`.
pub fn trigonometric_cde(c: f64, d: f64, e: f64, lower: f64, upper: f64) -> TrigResult {
    trigonometric([0.0, 0.0, c, d, e], lower, upper, 1.5e-12)
}
