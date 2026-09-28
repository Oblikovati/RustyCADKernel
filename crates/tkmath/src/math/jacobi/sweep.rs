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

use super::{Jacobi, Matrix, Vector};
impl Jacobi {
    pub(super) fn diagonalize(&mut self, work: &mut Matrix<'_>) {
        let n: i32 = self.values.length();
        let mut sums: Vector<'static> = Vector::with_bounds(1, n).unwrap();
        let mut shifts: Vector<'static> = Vector::with_bounds(1, n).unwrap();
        for index in 1..=n {
            self.vectors[(index, index)] = 1.0;
            sums[index] = work[(index, index)];
            self.values[index] = sums[index];
        }
        self.iterate(work, &mut sums, &mut shifts);
        self.sort_eigenpairs();
    }
    fn iterate(&mut self, work: &mut Matrix<'_>, sums: &mut Vector<'_>, shifts: &mut Vector<'_>) {
        let n: i32 = self.values.length();
        for iteration in 1..=50 {
            let sum: f64 = upper_sum(work);
            if sum == 0.0 {
                self.done = true;
                return;
            }
            let threshold: f64 = if iteration < 4 {
                0.2 * sum / f64::from(n * n)
            } else {
                0.0
            };
            self.sweep(work, shifts, iteration, threshold);
            accumulate(&mut self.values, sums, shifts);
        }
    }
    fn sweep(
        &mut self,
        work: &mut Matrix<'_>,
        shifts: &mut Vector<'_>,
        iteration: i32,
        threshold: f64,
    ) {
        for row in 1..self.values.length() {
            for col in row + 1..=self.values.length() {
                self.pivot(work, shifts, row, col, iteration, threshold);
            }
        }
    }
    fn sort_eigenpairs(&mut self) {
        for index in 1..self.values.length() {
            let selected: i32 = self.last_maximum(index);
            if selected == index {
                continue;
            }
            let value: f64 = self.values[index];
            self.values[index] = self.values[selected];
            self.values[selected] = value;
            self.vectors.swap_col(index, selected).unwrap();
        }
    }
    fn last_maximum(&self, start: i32) -> i32 {
        let mut selected: i32 = start;
        for index in start + 1..=self.values.length() {
            if self.values[index] >= self.values[selected] {
                selected = index;
            }
        }
        selected
    }
}
fn upper_sum(work: &Matrix<'_>) -> f64 {
    let mut sum: f64 = 0.0;
    for row in 1..work.row_number() {
        for col in row + 1..=work.col_number() {
            sum += work[(row, col)].abs();
        }
    }
    sum
}
fn accumulate(values: &mut Vector<'_>, sums: &mut Vector<'_>, shifts: &mut Vector<'_>) {
    for index in 1..=values.length() {
        sums[index] += shifts[index];
        values[index] = sums[index];
        shifts[index] = 0.0;
    }
}
