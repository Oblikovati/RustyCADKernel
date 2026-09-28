// Created on: 2005-12-15
// Created by: Julia GERASIMOVA
// Copyright (c) 2005-2014 OPEN CASCADE SAS
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

//! Symmetric tridiagonal eigenanalysis with OCCT's implicit QL iteration.
use super::{Matrix, Vector};
use rustycad_tkernel::{ncollection::Array1, standard::Failure};
mod ql;
/// Tridiagonal eigenpairs in native iteration order; `EigenValuesSearcher::new(&diagonal,&subdiagonal)?`.
pub struct EigenValuesSearcher {
    diagonal: Vector<'static>,
    subdiagonal: Vector<'static>,
    vectors: Matrix<'static>,
    values: Vector<'static>,
    done: bool,
}
impl EigenValuesSearcher {
    /// Copies bounded arrays; the first subdiagonal entry is unused; `EigenValuesSearcher::new(&diagonal,&subdiagonal)?`.
    pub fn new(diagonal: &Array1<'_, f64>, subdiagonal: &Array1<'_, f64>) -> Result<Self, Failure> {
        let n: i32 = diagonal.len() as i32;
        let mut solver: Self = Self::allocate(n)?;
        if diagonal.len() != subdiagonal.len() {
            return Ok(solver);
        }
        solver.initialize(diagonal, subdiagonal);
        solver.done = solver.iterate();
        if solver.done {
            solver.values.initialized(&solver.diagonal)?;
        }
        Ok(solver)
    }
    fn allocate(n: i32) -> Result<Self, Failure> {
        Ok(Self {
            diagonal: Vector::with_bounds(1, n)?,
            subdiagonal: Vector::with_bounds(1, n)?,
            vectors: Matrix::with_bounds(1, n, 1, n).unwrap(),
            values: Vector::with_bounds(1, n)?,
            done: false,
        })
    }
    fn initialize(&mut self, diagonal: &Array1<'_, f64>, subdiagonal: &Array1<'_, f64>) {
        for index in 1..=self.dimension() {
            self.diagonal[index] = diagonal.as_slice()[(index - 1) as usize];
            self.subdiagonal[index] = subdiagonal
                .as_slice()
                .get(index as usize)
                .copied()
                .unwrap_or(0.0);
            self.vectors[(index, index)] = 1.0;
        }
    }
    /// Reports convergence, or false on mismatched input lengths; `solver.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Reports the diagonal length independently of convergence; `solver.dimension()`.
    pub fn dimension(&self) -> i32 {
        self.diagonal.length()
    }
    /// Reads a one-based result without adding a native-absent NotDone check; `solver.eigen_value(1)?`.
    pub fn eigen_value(&self, index: i32) -> Result<f64, Failure> {
        self.values.value(index).copied()
    }
    /// Copies a one-based eigenvector column; `solver.eigen_vector(1)?`.
    pub fn eigen_vector(&self, index: i32) -> Result<Vector<'static>, Failure> {
        self.vectors.col(index)
    }
    fn iterate(&mut self) -> bool {
        for start in 1..=self.dimension() {
            if !self.converge(start) {
                return false;
            }
        }
        true
    }
    fn converge(&mut self, start: i32) -> bool {
        for iteration in 0..=30 {
            let end: i32 = self.submatrix_end(start);
            if end == start {
                return true;
            }
            if iteration == 30 {
                return false;
            }
            self.ql_step(start, end, self.wilkinson_shift(start, end));
        }
        false
    }
    fn submatrix_end(&self, start: i32) -> i32 {
        for end in start..self.dimension() {
            let sum: f64 = self.diagonal[end].abs() + self.diagonal[end + 1].abs();
            if self.subdiagonal[end].abs() + sum == sum {
                return end;
            }
        }
        self.dimension()
    }
    fn wilkinson_shift(&self, start: i32, end: i32) -> f64 {
        let shift: f64 =
            (self.diagonal[start + 1] - self.diagonal[start]) / (2.0 * self.subdiagonal[start]);
        let radius: f64 = (1.0 + shift * shift).sqrt();
        let denominator: f64 = if shift < 0.0 {
            shift - radius
        } else {
            shift + radius
        };
        self.diagonal[end] - self.diagonal[start] + self.subdiagonal[start] / denominator
    }
}
