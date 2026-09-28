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

use super::{Householder, Matrix};
struct Reflection {
    column: i32,
    g: f64,
    h: f64,
    alpha: f64,
}
impl Householder {
    pub(super) fn reduce(&mut self, rhs: &mut Matrix<'_>, epsilon: f64) {
        for column in 1..=self.q.col_number() {
            let reflection = self.reflection(column);
            if reflection.g.abs() <= epsilon {
                return;
            }
            for next in column + 1..=self.q.col_number() {
                self.reflect_column(next, &reflection);
            }
            for next in 1..=rhs.col_number() {
                self.reflect_rhs(rhs, next, &reflection);
            }
            self.q[(column, column)] = reflection.g;
        }
        self.back_substitute(rhs);
        self.done = true;
    }
    fn reflection(&self, column: i32) -> Reflection {
        let mut h = 0.0;
        for row in column..=self.q.row_number() {
            let value = self.q[(row, column)];
            h += value * value;
        }
        let f = self.q[(column, column)];
        let g = if f < 1e-15 { h.sqrt() } else { -h.sqrt() };
        h -= f * g;
        Reflection {
            column,
            g,
            h,
            alpha: g - f,
        }
    }
    fn reflect_column(&mut self, next: i32, r: &Reflection) {
        let mut scale = 0.0;
        for row in r.column..=self.q.row_number() {
            scale += self.q[(row, r.column)] * self.q[(row, next)];
        }
        let coefficient = (r.g * self.q[(r.column, next)] - scale) / r.h;
        self.q[(r.column, next)] -= r.alpha * coefficient;
        for row in r.column + 1..=self.q.row_number() {
            self.q[(row, next)] += coefficient * self.q[(row, r.column)];
        }
    }
    fn reflect_rhs(&self, rhs: &mut Matrix<'_>, next: i32, r: &Reflection) {
        let mut scale = self.q[(r.column, r.column)] * rhs[(r.column, next)];
        for row in r.column + 1..=self.q.row_number() {
            scale += self.q[(row, r.column)] * rhs[(row, next)];
        }
        let coefficient = (r.g * rhs[(r.column, next)] - scale) / r.h;
        rhs[(r.column, next)] -= coefficient * r.alpha;
        for row in r.column + 1..=self.q.row_number() {
            rhs[(row, next)] += coefficient * self.q[(row, r.column)];
        }
    }
    fn back_substitute(&mut self, rhs: &Matrix<'_>) {
        let n = self.q.col_number();
        for col in 1..=rhs.col_number() {
            self.solution[(n, col)] = rhs[(n, col)] / self.q[(n, n)];
            for row in (1..n).rev() {
                self.solution[(row, col)] = self.back_component(rhs, row, col);
            }
        }
    }
    fn back_component(&self, rhs: &Matrix<'_>, row: i32, col: i32) -> f64 {
        let mut scale = 0.0;
        for k in row + 1..=self.q.col_number() {
            scale += self.q[(row, k)] * self.solution[(k, col)];
        }
        (rhs[(row, col)] - scale) / self.q[(row, row)]
    }
}
