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

//! Direct math_Uzawa.cxx at OCCT 3d097a0: equality projection and projected dual iteration.
use super::{Crout, Matrix, Vector};
use rustycad_tkernel::standard::Failure;
mod direct;
mod iterate;
mod results;
/// Constrained least-distance projection; `Uzawa::new(&constraints,&rhs,&start)?`.
pub struct Uzawa {
    solution: Vector<'static>,
    error: Vector<'static>,
    initial: Vector<'static>,
    dual: Vector<'static>,
    inverse: Matrix<'static>,
    iterations: i32,
    done: bool,
}
impl Uzawa {
    /// Solves equality constraints with native defaults; `Uzawa::new(&matrix,&rhs,&start)?`.
    pub fn new(matrix: &Matrix<'_>, rhs: &Vector<'_>, start: &Vector<'_>) -> Result<Self, Failure> {
        Self::with_options(matrix, rhs, start, 1e-6, 1e-6, 500)
    }
    /// Configures equality-only projection; `Uzawa::with_options(&matrix,&rhs,&start,1e-6,1e-6,500)?`.
    pub fn with_options(
        matrix: &Matrix<'_>,
        rhs: &Vector<'_>,
        start: &Vector<'_>,
        epsilon_x: f64,
        epsilon_dual: f64,
        limit: i32,
    ) -> Result<Self, Failure> {
        Self::with_constraints(
            matrix,
            rhs,
            start,
            matrix.row_number(),
            0,
            [epsilon_x, epsilon_dual],
            limit,
        )
    }
    /// Uses implementation-order counts: equality first, inequality second; `Uzawa::with_constraints(&matrix,&rhs,&start,1,2,[1e-6,1e-6],500)?`.
    pub fn with_constraints(
        matrix: &Matrix<'_>,
        rhs: &Vector<'_>,
        start: &Vector<'_>,
        equalities: i32,
        inequalities: i32,
        tolerances: [f64; 2],
        limit: i32,
    ) -> Result<Self, Failure> {
        let mut solver: Self = Self::allocate(matrix.row_number(), matrix.col_number())?;
        solver.perform(
            matrix,
            rhs,
            start,
            [equalities, inequalities],
            tolerances,
            limit,
        )?;
        Ok(solver)
    }
    fn allocate(rows: i32, columns: i32) -> Result<Self, Failure> {
        Ok(Self {
            solution: Vector::with_bounds(1, columns)?,
            error: Vector::with_bounds(1, columns)?,
            initial: Vector::with_bounds(1, columns)?,
            dual: Vector::with_bounds(1, rows)?,
            inverse: Matrix::with_bounds(1, rows, 1, rows).unwrap(),
            iterations: 0,
            done: false,
        })
    }
    /// Repeats the native protected operation using existing storage; `solver.perform(&matrix,&rhs,&start,[1,2],[1e-6,1e-6],500)?`.
    pub fn perform(
        &mut self,
        matrix: &Matrix<'_>,
        rhs: &Vector<'_>,
        start: &Vector<'_>,
        counts: [i32; 2],
        tolerances: [f64; 2],
        limit: i32,
    ) -> Result<(), Failure> {
        self.check_dimensions(matrix, rhs, start, counts)?;
        self.initial_residual(matrix, rhs, start);
        if counts[1] == 0 {
            return self.direct(matrix, start);
        }
        self.iterate(matrix, start, counts[0], tolerances, limit);
        Ok(())
    }
    fn check_dimensions(
        &self,
        matrix: &Matrix<'_>,
        rhs: &Vector<'_>,
        start: &Vector<'_>,
        counts: [i32; 2],
    ) -> Result<(), Failure> {
        let (rows, columns): (i32, i32) = (matrix.row_number(), matrix.col_number());
        // Native Errinit has column count storage but receives row count writes; tall input is undefined.
        if rhs.length() != rows
            || counts[0] + counts[1] != rows
            || rows > columns
            || start.length() != columns
            || self.solution.length() != columns
            || self.dual.length() != rows
        {
            return Err(Failure::with_type(
                "Standard_DimensionError",
                &format!(
                    "Uzawa: matrix {rows}x{columns}, rhs {}, start {}, counts {counts:?}; expected matching lengths/counts and rows <= columns",
                    rhs.length(),
                    start.length()
                ),
                "",
            ));
        }
        Ok(())
    }
    fn initial_residual(&mut self, matrix: &Matrix<'_>, rhs: &Vector<'_>, start: &Vector<'_>) {
        for row in 1..=matrix.row_number() {
            self.initial[row] = matrix[(row, 1)] * start[1] - rhs[row];
            for column in 2..=matrix.col_number() {
                self.initial[row] += matrix[(row, column)] * start[column];
            }
        }
    }
}
impl Uzawa {
    /// Writes the native `Dump` text with the solution vector; `solver.dump(&mut text)?`.
    pub fn dump(&self, stream: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(stream, "math_Uzawa")?;
        if !self.done {
            return writeln!(stream, " Status = not Done ");
        }
        writeln!(stream, " Status = Done ")?;
        writeln!(stream, " Number of iterations = {}", self.iterations)?;
        write!(stream, " The solution vector is: ")?;
        self.solution.dump(stream)?;
        writeln!(stream)
    }
}
display_via_dump!(Uzawa);
