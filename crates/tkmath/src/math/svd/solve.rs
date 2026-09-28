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

use super::{Failure, Matrix, Svd, Vector};
impl Svd {
    /// Applies a persistent singular-value cutoff and solves; `solver.solve(&rhs,&mut output,1e-6)?`.
    pub fn solve(
        &mut self,
        rhs: &Vector<'_>,
        output: &mut Vector<'_>,
        epsilon: f64,
    ) -> Result<(), Failure> {
        self.check_done()?;
        let mut padded: Vector<'static> = Vector::filled(1, self.u.row_number(), 0.0)?;
        for offset in 0..rhs.length() {
            padded[offset + 1] = *rhs.at(offset as usize)?;
        }
        self.truncate(epsilon);
        self.solve_reduced(&padded, output)
    }
    /// Writes the one-based pseudoinverse and retains the cutoff; `solver.pseudo_inverse(&mut inverse,1e-6)?`.
    pub fn pseudo_inverse(
        &mut self,
        inverse: &mut Matrix<'_>,
        epsilon: f64,
    ) -> Result<(), Failure> {
        self.check_done()?;
        self.truncate(epsilon);
        let mut basis: Vector<'static> = Vector::with_bounds(1, self.u.row_number())?;
        let mut column: Vector<'static> = Vector::with_bounds(1, self.diagonal.length())?;
        for col in 1..=self.rows {
            basis.init(0.0);
            basis[col] = 1.0;
            self.solve_reduced(&basis, &mut column)?;
            for row in 1..=self.diagonal.length() {
                *inverse.change_value(row, col)? = column[row];
            }
        }
        Ok(())
    }
    fn truncate(&mut self, epsilon: f64) {
        let minimum: f64 = epsilon * self.diagonal[self.diagonal.max_index()];
        for index in 1..=self.diagonal.length() {
            if self.diagonal[index] < minimum {
                self.diagonal[index] = 0.0;
            }
        }
    }
    fn solve_reduced(&self, rhs: &Vector<'_>, output: &mut Vector<'_>) -> Result<(), Failure> {
        let mut temporary: Vector<'static> = Vector::with_bounds(1, self.diagonal.length())?;
        for col in 1..=self.diagonal.length() {
            temporary[col] = self.project_rhs(rhs, col);
        }
        for row in 1..=self.diagonal.length() {
            let mut sum: f64 = 0.0;
            for col in 1..=self.diagonal.length() {
                sum += self.v[(row, col)] * temporary[col];
            }
            *output.change_at((row - 1) as usize)? = sum;
        }
        Ok(())
    }
    fn project_rhs(&self, rhs: &Vector<'_>, col: i32) -> f64 {
        if self.diagonal[col] == 0.0 {
            return 0.0;
        }
        let mut sum: f64 = 0.0;
        for row in 1..=self.u.row_number() {
            sum += self.u[(row, col)] * rhs[row];
        }
        sum / self.diagonal[col]
    }
}
