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

//! Modern polynomial roots from pinned OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::math_utils::Status;
mod direct;
pub(crate) mod extended;
pub use direct::{cubic, linear, quadratic, quartic};
/// Native maximum accepted polynomial degree; `MAX_POLY_DEGREE`.
pub const MAX_POLY_DEGREE: usize = 20;
/// Kernel-owned complex storage; `Complex::new(1.0,2.0)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Complex {
    value: num_complex::Complex64,
}
impl Complex {
    /// Builds Cartesian coordinates; `Complex::new(1.0,2.0)`.
    pub const fn new(real: f64, imag: f64) -> Self {
        Self {
            value: num_complex::Complex64::new(real, imag),
        }
    }
    /// Returns the real coordinate; `Complex::new(1.0,2.0).real()`.
    pub fn real(self) -> f64 {
        self.value.re
    }
    /// Returns the imaginary coordinate; `Complex::new(1.0,2.0).imag()`.
    pub fn imag(self) -> f64 {
        self.value.im
    }
}
/// Native root arrays retain partial results on failure; `PolyResult::default()`.
#[derive(Default)]
pub struct PolyResult {
    /// Completion status; `result.status`.
    pub status: Status,
    /// Real root storage; `result.roots`.
    pub roots: [f64; MAX_POLY_DEGREE],
    /// Real algebraic multiplicities; `result.multiplicities`.
    pub multiplicities: [usize; MAX_POLY_DEGREE],
    /// Complex root storage; `result.complex_roots`.
    pub complex_roots: [Complex; MAX_POLY_DEGREE],
    /// Complex algebraic multiplicities; `result.complex_multiplicities`.
    pub complex_multiplicities: [usize; MAX_POLY_DEGREE],
    /// Accepted real roots; `result.nb_roots`.
    pub nb_roots: usize,
    /// Accepted complex roots; `result.nb_complex_roots`.
    pub nb_complex_roots: usize,
}
/// Native general-result alias; `GeneralPolyResult::default()`.
pub type GeneralPolyResult = PolyResult;
impl PolyResult {
    /// Reports native success; `quadratic(1.0,-5.0,6.0).is_done()`.
    pub fn is_done(&self) -> bool {
        self.status == Status::Ok
    }
    /// Counts real roots with algebraic multiplicity; `linear(1.0,0.0).nb_real_roots_with_multiplicity()`.
    pub fn nb_real_roots_with_multiplicity(&self) -> usize {
        self.multiplicities[..self.nb_roots].iter().sum()
    }
    fn failed(status: Status) -> Self {
        Self {
            status,
            ..Default::default()
        }
    }
    fn append(&mut self, root: f64, multiplicity: usize) {
        self.roots[self.nb_roots] = root;
        self.multiplicities[self.nb_roots] = multiplicity;
        self.nb_roots += 1;
    }
}
impl From<&PolyResult> for bool {
    fn from(result: &PolyResult) -> bool {
        result.is_done()
    }
}
impl std::ops::Index<usize> for PolyResult {
    type Output = f64;
    fn index(&self, index: usize) -> &f64 {
        &self.roots[index]
    }
}
