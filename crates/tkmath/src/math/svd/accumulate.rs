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

use super::{Svd, Vector};
impl Svd {
    pub(super) fn accumulate_right(&mut self, residual: &Vector<'_>) {
        let n: i32 = self.diagonal.length();
        for index in (1..=n).rev() {
            if index < n {
                self.right_column(index, residual[index + 1]);
            }
            self.v[(index, index)] = 1.0;
        }
    }
    fn right_column(&mut self, index: i32, g: f64) {
        if g != 0.0 {
            for row in index + 1..=self.diagonal.length() {
                self.v[(row, index)] = (self.u[(index, row)] / self.u[(index, index + 1)]) / g;
            }
            self.right_projection(index);
        }
        for col in index + 1..=self.diagonal.length() {
            self.v[(index, col)] = 0.0;
            self.v[(col, index)] = 0.0;
        }
    }
    fn right_projection(&mut self, index: i32) {
        for col in index + 1..=self.diagonal.length() {
            let sum: f64 = self.right_dot(index, col);
            for row in index + 1..=self.diagonal.length() {
                self.v[(row, col)] += sum * self.v[(row, index)];
            }
        }
    }
    fn right_dot(&self, index: i32, col: i32) -> f64 {
        let mut sum: f64 = 0.0;
        for row in index + 1..=self.diagonal.length() {
            sum += self.u[(index, row)] * self.v[(row, col)];
        }
        sum
    }
    pub(super) fn accumulate_left(&mut self) {
        for index in (1..=self.diagonal.length()).rev() {
            for col in index + 1..=self.diagonal.length() {
                self.u[(index, col)] = 0.0;
            }
            self.left_column(index);
            self.u[(index, index)] += 1.0;
        }
    }
    fn left_column(&mut self, index: i32) {
        let g: f64 = self.diagonal[index];
        if g == 0.0 {
            for row in index..=self.u.row_number() {
                self.u[(row, index)] = 0.0;
            }
            return;
        }
        let reciprocal: f64 = 1.0 / g;
        self.left_projection(index, reciprocal);
        for row in index..=self.u.row_number() {
            self.u[(row, index)] *= reciprocal;
        }
    }
    fn left_projection(&mut self, index: i32, reciprocal: f64) {
        for col in index + 1..=self.diagonal.length() {
            let sum: f64 = self.column_dot(index, col, index + 1);
            let f: f64 = (sum / self.u[(index, index)]) * reciprocal;
            for row in index..=self.u.row_number() {
                self.u[(row, col)] += f * self.u[(row, index)];
            }
        }
    }
}
