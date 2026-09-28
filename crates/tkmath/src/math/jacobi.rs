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

//! math_Jacobi: symmetric eigenanalysis through the `Jacobi` recipe.
use super::recipes::{STATUS_OK, jacobi};
use super::{Matrix, Vector};
use rustycad_tkernel::standard::Failure;
use std::fmt;
/// Jacobi eigenpairs in descending order; `Jacobi::new(&matrix)?`.
pub struct Jacobi {
    values: Vector<'static>,
    vectors: Matrix<'static>,
    done: bool,
    rotations: i32,
}
impl Jacobi {
    /// Copies the matrix and diagonalizes its upper triangle; `Jacobi::new(&matrix)?`.
    pub fn new(matrix: &Matrix<'_>) -> Result<Self, Failure> {
        let n: i32 = matrix.row_number();
        if n != matrix.col_number() {
            return Err(super::not_square(" "));
        }
        let mut work: Matrix<'static> = Matrix::with_bounds(1, n, 1, n).unwrap();
        work.initialized(matrix)?;
        Self::diagonalized(&mut work)
    }
    fn diagonalized(work: &mut Matrix<'_>) -> Result<Self, Failure> {
        let n = work.row_number();
        let mut values = Vector::with_bounds(1, n)?;
        let mut vectors = Matrix::with_bounds(1, n, 1, n).unwrap();
        let mut rotations = 0;
        let done = jacobi(work, &mut values, &mut vectors, &mut rotations)? == STATUS_OK;
        Ok(Self {
            values,
            vectors,
            done,
            rotations,
        })
    }
    /// Runs the recipe on a caller-owned 1-based work matrix (used by MathLin).
    pub(crate) fn recipe(work: &mut Matrix<'_>) -> (bool, u32, Vector<'static>, Matrix<'static>) {
        let solver = Self::diagonalized(work).unwrap();
        (
            solver.done,
            solver.rotations as u32,
            solver.values,
            solver.vectors,
        )
    }
    /// Reports convergence within fifty sweeps; `solver.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// Borrows sorted eigenvalues; `solver.values()?`.
    pub fn values(&self) -> Result<&Vector<'static>, Failure> {
        self.check_done()?;
        Ok(&self.values)
    }
    /// Reads one eigenvalue using one-based indexing; `solver.value(1)?`.
    pub fn value(&self, index: i32) -> Result<f64, Failure> {
        self.check_done()?;
        self.values.value(index).copied()
    }
    /// Borrows eigenvectors stored in columns; `solver.vectors()?`.
    pub fn vectors(&self) -> Result<&Matrix<'static>, Failure> {
        self.check_done()?;
        Ok(&self.vectors)
    }
    /// Copies a column into an existing bounded vector; `solver.vector(1,&mut vector)?`.
    pub fn vector(&self, index: i32, vector: &mut Vector<'_>) -> Result<(), Failure> {
        self.check_done()?;
        vector.initialized(&self.vectors.col(index)?)
    }
    /// Writes the native status text with the eigenvalue vector's own dump;
    /// `solver.dump(&mut text)?`.
    pub fn dump(&self, stream: &mut impl fmt::Write) -> fmt::Result {
        write!(stream, "math_Jacobi ")?;
        if !self.done {
            return writeln!(stream, "Status = not Done ");
        }
        writeln!(stream, " Status = Done ")?;
        write!(stream, " The eigenvalues vector is: ")?;
        self.values.dump(stream)?;
        writeln!(stream)
    }
    fn check_done(&self) -> Result<(), Failure> {
        if !self.done {
            return Err(Failure::with_type("StdFail_NotDone", " ", ""));
        }
        Ok(())
    }
}
