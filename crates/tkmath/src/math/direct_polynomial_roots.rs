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

//! Direct math_DirectPolynomialRoots.cxx at OCCT 3d097a0; preserves order and multiplicity.
use rustycad_tkernel::standard::Failure;
mod cubic;
mod cubic_roots;
mod ferrari;
mod quadratic;
mod quartic;
mod refine;
mod scaling;
const ZERO: f64 = 1e-30;
/// Algebraic real roots through degree four; `DirectPolynomialRoots::quadratic(1.0,-5.0,6.0)`.
pub struct DirectPolynomialRoots {
    roots: [f64; 4],
    count: i32,
    done: bool,
    infinite: bool,
}
impl DirectPolynomialRoots {
    fn empty() -> Self {
        Self {
            roots: [0.0; 4],
            count: 0,
            done: true,
            infinite: false,
        }
    }
    /// Solves a linear equation, including infinite solutions; `DirectPolynomialRoots::linear(2.0,-6.0)`.
    pub fn linear(a: f64, b: f64) -> Self {
        let mut solver: Self = Self::empty();
        solver.solve_linear(a, b);
        solver
    }
    /// Solves a quadratic with native stable root ordering; `DirectPolynomialRoots::quadratic(1.0,-5.0,6.0)`.
    pub fn quadratic(a: f64, b: f64, c: f64) -> Self {
        let mut solver: Self = Self::empty();
        solver.solve_quadratic(a, b, c);
        solver
    }
    /// Solves a cubic with Cardano's native branches; `DirectPolynomialRoots::cubic(1.0,-6.0,11.0,-6.0)`.
    pub fn cubic(a: f64, b: f64, c: f64, d: f64) -> Self {
        let mut solver: Self = Self::empty();
        solver.solve_cubic(a, b, c, d);
        solver
    }
    /// Solves a quartic using Ferrari factorization; `DirectPolynomialRoots::quartic(1.0,0.0,-5.0,0.0,4.0)`.
    pub fn quartic(a: f64, b: f64, c: f64, d: f64, e: f64) -> Self {
        let mut solver: Self = Self::empty();
        solver.solve_quartic(a, b, c, d, e);
        solver
    }
    /// Reports native computation status; `solver.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Reports the identity-equation flag; `solver.infinite_roots()`.
    pub fn infinite_roots(&self) -> bool {
        self.infinite
    }
    /// Counts roots with multiplicity; native has no Done guard here; `solver.solutions()?`.
    pub fn solutions(&self) -> Result<i32, Failure> {
        self.check_finite()?;
        Ok(self.count)
    }
    /// Reads a one-based root in native order; `solver.value(1)?`.
    pub fn value(&self, index: i32) -> Result<f64, Failure> {
        self.check_finite()?;
        if index < 1 || index > self.count {
            return Err(Failure::with_type("Standard_RangeError", " ", ""));
        }
        Ok(self.roots[(index - 1) as usize])
    }
    fn check_finite(&self) -> Result<(), Failure> {
        if self.infinite {
            return Err(Failure::with_type("StdFail_InfiniteSolutions", " ", ""));
        }
        Ok(())
    }
    fn scale_refine(&mut self, scale: f64, coefficients: &[f64]) {
        for root in &mut self.roots[..self.count as usize] {
            *root *= scale;
            *root = refine::refine(coefficients, *root);
        }
    }
    /// Repeats native protected linear Solve without clearing earlier flags; `solver.solve_linear(2.0,-6.0)`.
    pub fn solve_linear(&mut self, a: f64, b: f64) {
        if a.abs() <= ZERO {
            if b.abs() <= ZERO {
                self.infinite = true;
                return;
            }
            self.count = 0;
            return;
        }
        self.count = 1;
        self.roots[0] = -b / a;
    }
}
