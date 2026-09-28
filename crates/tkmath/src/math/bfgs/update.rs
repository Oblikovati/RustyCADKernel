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

use super::{Matrix, Vector};
pub(super) fn update_hessian(
    hessian: &mut Matrix<'static>,
    step: &Vector<'static>,
    difference: &mut Vector<'static>,
) {
    let size: i32 = difference.upper();
    let mut product: Vector<'static> = Vector::filled(1, size, 0.0).unwrap();
    for row in 1..=size {
        for column in 1..=size {
            product[row] += hessian[(row, column)] * difference[column];
        }
    }
    let (fac, fae): (f64, f64) = curvature(difference, step, &product);
    let fad: f64 = 1.0 / fae;
    for index in 1..=size {
        difference[index] = fac * step[index] - fad * product[index];
    }
    for row in 1..=size {
        update_row(hessian, step, difference, &product, row, (fac, fad, fae));
    }
}
fn curvature(difference: &Vector<'_>, step: &Vector<'_>, product: &Vector<'_>) -> (f64, f64) {
    let (mut fac, mut fae): (f64, f64) = (0.0, 0.0);
    for index in 1..=difference.upper() {
        fac += difference[index] * step[index];
        fae += difference[index] * product[index];
    }
    (1.0 / fac, fae)
}
fn update_row(
    hessian: &mut Matrix<'static>,
    step: &Vector<'_>,
    difference: &Vector<'_>,
    product: &Vector<'_>,
    row: i32,
    (fac, fad, fae): (f64, f64, f64),
) {
    for column in 1..=difference.upper() {
        hessian[(row, column)] += fac * step[row] * step[column]
            - fad * product[row] * product[column]
            + fae * difference[row] * difference[column];
    }
}
pub(super) fn next_direction(
    hessian: &Matrix<'_>,
    gradient: &Vector<'_>,
    direction: &mut Vector<'_>,
) {
    for row in 1..=direction.upper() {
        direction[row] = 0.0;
        for column in 1..=direction.upper() {
            direction[row] -= hessian[(row, column)] * gradient[column];
        }
    }
}
