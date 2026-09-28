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

//! MathRoot_Brent.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::bracket::finite_value;
use crate::math::Function;
use crate::math_utils::{Config, ScalarResult, Status};
struct BrentState {
    a: f64,
    b: f64,
    c: f64,
    fa: f64,
    fb: f64,
    fc: f64,
    d: f64,
    e: f64,
}
impl BrentState {
    fn prepare(&mut self) {
        if self.fb.is_sign_negative() == self.fc.is_sign_negative() {
            self.c = self.a;
            self.fc = self.fa;
            self.d = self.b - self.a;
            self.e = self.d;
        }
        if self.fc.abs() < self.fb.abs() {
            self.a = self.b;
            self.b = self.c;
            self.c = self.a;
            self.fa = self.fb;
            self.fb = self.fc;
            self.fc = self.fa;
        }
    }
    fn interpolate(&mut self, mid: f64, tol: f64) {
        let s: f64 = self.fb / self.fa;
        let (mut p, mut q): (f64, f64) = if self.a == self.c {
            (2.0 * mid * s, 1.0 - s)
        } else {
            let q: f64 = self.fa / self.fc;
            let r: f64 = self.fb / self.fc;
            (
                s * (2.0 * mid * q * (q - r) - (self.b - self.a) * (r - 1.0)),
                (q - 1.0) * (r - 1.0) * (s - 1.0),
            )
        };
        if p > 0.0 {
            q = -q;
        }
        p = p.abs();
        let min1: f64 = 3.0 * mid * q - (tol * q).abs();
        let min2: f64 = (self.e * q).abs();
        if 2.0 * p < if min1 < min2 { min1 } else { min2 } {
            self.e = self.d;
            self.d = p / q;
        } else {
            self.d = mid;
            self.e = self.d;
        }
    }
    fn advance(&mut self, mid: f64, tol: f64) {
        if self.e.abs() >= tol && self.fa.abs() > self.fb.abs() {
            self.interpolate(mid, tol);
        } else {
            self.d = mid;
            self.e = self.d;
        }
        self.a = self.b;
        self.fa = self.fb;
        self.b += if self.d.abs() > tol {
            self.d
        } else if mid > 0.0 {
            tol.abs()
        } else {
            -tol.abs()
        };
    }
}
fn initialize(
    f: &mut impl Function,
    lower: f64,
    upper: f64,
    config: &Config,
) -> Result<BrentState, ScalarResult> {
    let [a, b, fa, fb]: [f64; 4] = super::bracket::initialize(f, lower, upper, config)?;
    Ok(BrentState {
        a,
        b,
        c: b,
        fa,
        fb,
        fc: fb,
        d: 0.0,
        e: 0.0,
    })
}
/// Finds a bracketed root using native interpolation and stopping rules; `brent(&mut f,0.0,2.0,&Config::default())`.
pub fn brent(f: &mut impl Function, lower: f64, upper: f64, config: &Config) -> ScalarResult {
    let mut state: BrentState = match initialize(f, lower, upper, config) {
        Ok(state) => state,
        Err(result) => return result,
    };
    let mut result: ScalarResult = ScalarResult::default();
    for iteration in 0..config.max_iterations {
        result.iterations = iteration + 1;
        state.prepare();
        let tol: f64 = 2.0 * f64::EPSILON * state.b.abs() + 0.5 * config.x_tolerance;
        let mid: f64 = 0.5 * state.c + 0.5 * (-state.b);
        if mid.abs() <= tol || state.fb.abs() <= config.f_tolerance {
            result.status = Status::Ok;
            result.root = Some(state.b);
            result.value = Some(state.fb);
            return result;
        }
        state.advance(mid, tol);
        if !state.b.is_finite() {
            result.status = Status::NumericalError;
            return result;
        }
        if !finite_value(f, state.b, &mut state.fb, &mut result) {
            return result;
        }
    }
    result.status = Status::MaxIterations;
    result.root = Some(state.b);
    result.value = Some(state.fb);
    result
}
