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
    pub(super) fn pivot(
        &mut self,
        work: &mut Matrix<'_>,
        shifts: &mut Vector<'_>,
        row: i32,
        col: i32,
        iteration: i32,
        threshold: f64,
    ) {
        let magnitude: f64 = 100.0 * work[(row, col)].abs();
        let left: f64 = self.values[row].abs();
        let right: f64 = self.values[col].abs();
        if iteration > 4 && left + magnitude == left && right + magnitude == right {
            work[(row, col)] = 0.0;
            return;
        }
        if work[(row, col)].abs() > threshold {
            self.apply_rotation(work, shifts, row, col, magnitude);
            self.rotations += 1;
        }
    }
    fn apply_rotation(
        &mut self,
        work: &mut Matrix<'_>,
        shifts: &mut Vector<'_>,
        row: i32,
        col: i32,
        magnitude: f64,
    ) {
        let tangent: f64 = tangent(
            work[(row, col)],
            self.values[col] - self.values[row],
            magnitude,
        );
        let cosine: f64 = 1.0 / (1.0 + tangent * tangent).sqrt();
        let sine: f64 = tangent * cosine;
        let tau: f64 = sine / (1.0 + cosine);
        let shift: f64 = tangent * work[(row, col)];
        shifts[row] -= shift;
        shifts[col] += shift;
        self.values[row] -= shift;
        self.values[col] += shift;
        work[(row, col)] = 0.0;
        rotate_upper(work, row, col, sine, tau);
        for index in 1..=self.values.length() {
            rotate(&mut self.vectors, (index, row), (index, col), sine, tau);
        }
    }
}
fn tangent(pivot: f64, difference: f64, magnitude: f64) -> f64 {
    if difference.abs() + magnitude == difference.abs() {
        return pivot / difference;
    }
    let theta: f64 = 0.5 * difference / pivot;
    let result: f64 = 1.0 / (theta.abs() + (1.0 + theta * theta).sqrt());
    if theta < 0.0 { -result } else { result }
}
fn rotate_upper(work: &mut Matrix<'_>, row: i32, col: i32, sine: f64, tau: f64) {
    for index in 1..row {
        rotate(work, (index, row), (index, col), sine, tau);
    }
    for index in row + 1..col {
        rotate(work, (row, index), (index, col), sine, tau);
    }
    for index in col + 1..=work.row_number() {
        rotate(work, (row, index), (col, index), sine, tau);
    }
}
fn rotate(work: &mut Matrix<'_>, left: (i32, i32), right: (i32, i32), sine: f64, tau: f64) {
    let g: f64 = work[left];
    let h: f64 = work[right];
    work[left] = g - sine * (h + g * tau);
    work[right] = h + sine * (g - h * tau);
}
