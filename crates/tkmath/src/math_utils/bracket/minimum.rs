// Copyright (c) 2026 OPEN CASCADE SAS
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

use super::{Function, MinBracketOptions, Status, evaluate_finite, expansion, is_valid, limited};
mod initialize;
/// Successful fields satisfy a < b < c and fb < fa,fc; `MinBracketResult::default()`.
#[derive(Clone, Debug)]
pub struct MinBracketResult {
    /// Native `status` field; access with `result.status`.
    pub status: Status,
    /// Native `is_valid` field; access with `result.is_valid`.
    pub is_valid: bool,
    /// Native `iterations` field; access with `result.iterations`.
    pub iterations: u32,
    /// Native `a` field; access with `result.a`.
    pub a: f64,
    /// Native `b` field; access with `result.b`.
    pub b: f64,
    /// Native `c` field; access with `result.c`.
    pub c: f64,
    /// Native `fa` field; access with `result.fa`.
    pub fa: f64,
    /// Native `fb` field; access with `result.fb`.
    pub fb: f64,
    /// Native `fc` field; access with `result.fc`.
    pub fc: f64,
}
impl Default for MinBracketResult {
    fn default() -> Self {
        Self {
            status: Status::NotConverged,
            is_valid: false,
            iterations: 0,
            a: 0.0,
            b: 0.0,
            c: 0.0,
            fa: f64::NAN,
            fb: f64::NAN,
            fc: f64::NAN,
        }
    }
}
impl MinBracketResult {
    fn finish(&mut self) -> bool {
        if !(self.fb < self.fa && self.fb < self.fc) {
            return false;
        }
        if self.a > self.c {
            std::mem::swap(&mut self.a, &mut self.c);
            std::mem::swap(&mut self.fa, &mut self.fc);
        }
        self.is_valid = self.a < self.b && self.b < self.c;
        self.status = if self.is_valid {
            Status::Ok
        } else {
            Status::NumericalError
        };
        true
    }
    fn expand(&mut self, f: &mut impl Function, options: &MinBracketOptions) -> bool {
        let candidate: f64 = limited(expansion(self.c, self.b), options);
        if !self.valid_candidate(candidate, self.c) {
            return false;
        }
        let mut candidate_value: f64 = 0.0;
        if !evaluate_finite(f, candidate, &mut candidate_value, &mut self.status) {
            return false;
        }
        self.a = self.b;
        self.fa = self.fb;
        self.b = self.c;
        self.fb = self.fc;
        self.c = candidate;
        self.fc = candidate_value;
        self.iterations += 1;
        true
    }
    fn valid_candidate(&mut self, candidate: f64, previous: f64) -> bool {
        if !candidate.is_finite() {
            self.status = Status::NumericalError;
            return false;
        }
        if candidate == previous {
            self.status = Status::NoSolution;
            return false;
        }
        true
    }
}
/// Brackets a strict minimum using native expansion; `bracket_minimum(&mut f, 0.0, 1.0, &options)`.
pub fn bracket_minimum(
    f: &mut impl Function,
    a: f64,
    b: f64,
    options: &MinBracketOptions,
) -> MinBracketResult {
    let mut result: MinBracketResult = MinBracketResult::default();
    if !result.initialize(f, a, b, options) {
        return result;
    }
    for iteration in 0..=options.max_iterations {
        if result.finish() {
            return result;
        }
        if iteration == options.max_iterations {
            break;
        }
        if !result.expand(f, options) {
            return result;
        }
    }
    result.status = Status::MaxIterations;
    result
}
/// Native overload taking only an iteration budget (default options otherwise);
/// `bracket_minimum_iterations(&mut f, 0.0, 1.0, 50)`.
pub fn bracket_minimum_iterations(
    f: &mut impl Function,
    a: f64,
    b: f64,
    max_iterations: u32,
) -> MinBracketResult {
    let options: MinBracketOptions = MinBracketOptions {
        max_iterations,
        ..Default::default()
    };
    bracket_minimum(f, a, b, &options)
}
