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

//! Direct math_TrigonometricFunctionRoots.cxx at OCCT 3d097a0, including native multiplicities.
use super::{DirectPolynomialRoots, NewtonFunctionRoot, TrigonometricEquation};
use rustycad_tkernel::standard::Failure;
use std::f64::consts::PI;
mod analytical;
mod interval;
mod polynomial;
mod refine;
mod results;
use interval::Interval;
const EPS: f64 = 1.5e-12;
const PERIOD: f64 = PI + PI;
/// Real trigonometric roots over the native one-period window; `TrigonometricRoots::sine(1.0,0.0,0.0,2.0*PI)`.
pub struct TrigonometricRoots {
    roots: [f64; 4],
    count: i32,
    done: bool,
    infinite: bool,
}
impl TrigonometricRoots {
    /// Solves d*sin(x)+e=0; `TrigonometricRoots::sine(1.0,-0.5,0.0,2.0*PI)`.
    pub fn sine(d: f64, e: f64, lower: f64, upper: f64) -> Self {
        Self::new(0.0, 0.0, 0.0, d, e, lower, upper)
    }
    /// Solves c*cos(x)+d*sin(x)+e=0; `TrigonometricRoots::linear(1.0,1.0,0.0,0.0,2.0*PI)`.
    pub fn linear(c: f64, d: f64, e: f64, lower: f64, upper: f64) -> Self {
        Self::new(0.0, 0.0, c, d, e, lower, upper)
    }
    /// Solves the full quadratic trigonometric equation; `TrigonometricRoots::new(1.0,0.0,0.0,0.0,-0.5,0.0,2.0*PI)`.
    pub fn new(a: f64, b: f64, c: f64, d: f64, e: f64, lower: f64, upper: f64) -> Self {
        let mut solver: Self = Self {
            roots: [0.0; 4],
            count: -1,
            done: false,
            infinite: false,
        };
        solver.perform([a, b, c, d, e], [lower, upper]);
        solver
    }
    /// Repeats native protected Perform, resetting native status flags; `solver.perform([1.0,0.0,0.0,0.0,-0.5],[0.0,2.0*PI])`.
    pub fn perform(&mut self, coefficients: [f64; 5], bounds: [f64; 2]) {
        self.infinite = false;
        self.done = true;
        let interval: Interval = Interval::new(bounds);
        if self.analytical(coefficients, &interval) {
            return;
        }
        let Some(zeros) = self.polynomial_roots(coefficients) else {
            return;
        };
        self.refine_roots(&zeros, coefficients, &interval);
        self.add_pi(coefficients, &interval);
    }
    fn append(&mut self, root: f64) {
        self.roots[self.count as usize] = root;
        self.count += 1;
    }
    fn insert(&mut self, index: usize, root: f64) {
        for position in (index..self.count as usize).rev() {
            self.roots[position + 1] = self.roots[position];
        }
        self.roots[index] = root;
        self.count += 1;
    }
}
impl TrigonometricRoots {
    /// Writes the native `Dump` text; `roots.dump(&mut text)?`.
    pub fn dump(&self, stream: &mut impl std::fmt::Write) -> std::fmt::Result {
        writeln!(stream, " math_TrigonometricFunctionRoots: ")?;
        if !self.done {
            return writeln!(stream, "Not Done ");
        }
        if self.infinite {
            return writeln!(stream, " There is an infinity of roots");
        }
        writeln!(stream, " Number of solutions = {}", self.count)?;
        for index in 1..=self.count {
            let root = crate::math::stream_real(self.roots[(index - 1) as usize]);
            writeln!(stream, " Value number {index}= {root}")?;
        }
        Ok(())
    }
}
display_via_dump!(TrigonometricRoots);
