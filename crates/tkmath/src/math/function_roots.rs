// Copyright (c) 1997-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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

//! Sampled scalar root isolation, preserving the active OCCT FunctionRoots algorithm.
use super::FunctionWithDerivative;
mod brent;
mod extrema;
mod golden;
mod results;
mod sampling;
/// Roots and callback states in ascending order; `FunctionRoots::new(&mut f,-2.0,2.0,20)`.
pub struct FunctionRoots {
    roots: Vec<f64>,
    states: Vec<i32>,
    done: bool,
    all_null: bool,
}
#[derive(Clone, Copy)]
struct Search {
    lower: f64,
    upper: f64,
    count: usize,
    dx: f64,
    tolerance: f64,
    original: f64,
    duplicate: f64,
    value: f64,
    null: f64,
    target: f64,
}
impl Search {
    fn new(mut lower: f64, mut upper: f64, samples: i32, eps: [f64; 3], target: f64) -> Self {
        if upper < lower {
            std::mem::swap(&mut lower, &mut upper);
        }
        let count: usize = (i64::from(samples) * 2).max(20) as usize;
        let duplicate: f64 = 1e-10 * (lower.abs() + upper.abs());
        let tolerance: f64 = if eps[0] < duplicate {
            duplicate
        } else {
            eps[0]
        };
        Self {
            lower,
            upper,
            count,
            dx: (upper - lower) / count as f64,
            tolerance,
            original: eps[0],
            duplicate,
            value: eps[1],
            null: eps[2],
            target,
        }
    }
    fn clamp(self, mut x: f64) -> f64 {
        if x < self.lower {
            x = self.lower;
        }
        if x > self.upper {
            x = self.upper;
        }
        x
    }
}
impl FunctionRoots {
    /// Uses native zero default tolerances; `FunctionRoots::new(&mut f,-2.0,2.0,20)`.
    pub fn new(
        function: &mut impl FunctionWithDerivative,
        lower: f64,
        upper: f64,
        samples: i32,
    ) -> Self {
        Self::with_options(function, lower, upper, samples, [0.0; 3], 0.0)
    }
    /// Solves F(x)=target with `[EpsX,EpsF,EpsNull]`; `FunctionRoots::with_options(&mut f,-2.0,2.0,20,[1e-8;3],0.0)`.
    pub fn with_options(
        function: &mut impl FunctionWithDerivative,
        lower: f64,
        upper: f64,
        samples: i32,
        eps: [f64; 3],
        target: f64,
    ) -> Self {
        let search: Search = Search::new(lower, upper, samples, eps, target);
        let mut result: Self = Self {
            roots: Vec::new(),
            states: Vec::new(),
            done: true,
            all_null: false,
        };
        result.sample(function, search);
        result
    }
    fn append(&mut self, function: &mut impl FunctionWithDerivative, x: f64, duplicate: f64) {
        let mut position: usize = self.roots.len();
        for (index, &root) in self.roots.iter().enumerate() {
            if (x - root).abs() <= duplicate {
                return;
            }
            if root >= x {
                position = index;
                break;
            }
        }
        self.roots.insert(position, x);
        let mut value: f64 = 0.0;
        function.value(x, &mut value);
        self.states.insert(position, function.state_number());
    }
}
