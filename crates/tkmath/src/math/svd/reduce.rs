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

use super::{Svd, Vector, sign};
impl Svd {
    pub(super) fn bidiagonalize(&mut self, residual: &mut Vector<'_>) -> f64 {
        let mut norm: f64 = 0.0;
        let mut scale: f64 = 0.0;
        let mut g: f64 = 0.0;
        for index in 1..=self.diagonal.length() {
            residual[index] = scale * g;
            self.diagonal[index] = self.reduce_column(index);
            (scale, g) = self.reduce_row(index, residual);
            let mut aw: f64 = self.diagonal[index];
            if aw < 0.0 {
                aw = -aw;
            }
            let ar: f64 = if residual[index] > 0.0 {
                aw + residual[index]
            } else {
                aw - residual[index]
            };
            if norm < ar {
                norm = ar;
            }
        }
        norm
    }
    fn reduce_column(&mut self, index: i32) -> f64 {
        let mut scale: f64 = 0.0;
        for row in index..=self.u.row_number() {
            let entry: f64 = self.u[(row, index)];
            if entry > 0.0 {
                scale += entry;
            } else {
                scale -= entry;
            }
        }
        if scale == 0.0 {
            return scale * 0.0;
        }
        let mut sum: f64 = 0.0;
        for row in index..=self.u.row_number() {
            self.u[(row, index)] /= scale;
            sum += self.u[(row, index)] * self.u[(row, index)];
        }
        let f: f64 = self.u[(index, index)];
        let g: f64 = -sign(sum.sqrt(), f);
        let h: f64 = f * g - sum;
        self.u[(index, index)] = f - g;
        self.project_columns(index, h);
        for row in index..=self.u.row_number() {
            self.u[(row, index)] *= scale;
        }
        scale * g
    }
    fn project_columns(&mut self, index: i32, h: f64) {
        for col in index + 1..=self.diagonal.length() {
            let f: f64 = self.column_dot(index, col, index) / h;
            for row in index..=self.u.row_number() {
                self.u[(row, col)] += f * self.u[(row, index)];
            }
        }
    }
    pub(super) fn column_dot(&self, left: i32, right: i32, start: i32) -> f64 {
        let mut sum: f64 = 0.0;
        for row in start..=self.u.row_number() {
            sum += self.u[(row, left)] * self.u[(row, right)];
        }
        sum
    }
    fn reduce_row(&mut self, index: i32, residual: &mut Vector<'_>) -> (f64, f64) {
        let mut scale: f64 = 0.0;
        for col in index + 1..=self.diagonal.length() {
            let entry: f64 = self.u[(index, col)];
            if entry > 0.0 {
                scale += entry;
            } else {
                scale -= entry;
            }
        }
        if scale == 0.0 {
            return (scale, 0.0);
        }
        let mut sum: f64 = 0.0;
        for col in index + 1..=self.diagonal.length() {
            self.u[(index, col)] /= scale;
            sum += self.u[(index, col)] * self.u[(index, col)];
        }
        let f: f64 = self.u[(index, index + 1)];
        let g: f64 = -sign(sum.sqrt(), f);
        let h: f64 = f * g - sum;
        self.u[(index, index + 1)] = f - g;
        for col in index + 1..=self.diagonal.length() {
            residual[col] = self.u[(index, col)] / h;
        }
        self.project_rows(index, residual);
        for col in index + 1..=self.diagonal.length() {
            self.u[(index, col)] *= scale;
        }
        (scale, g)
    }
    fn project_rows(&mut self, index: i32, residual: &Vector<'_>) {
        for row in index + 1..=self.u.row_number() {
            let sum: f64 = self.row_dot(row, index, index + 1);
            for col in index + 1..=self.diagonal.length() {
                self.u[(row, col)] += sum * residual[col];
            }
        }
    }
    fn row_dot(&self, left: i32, right: i32, start: i32) -> f64 {
        let mut sum: f64 = 0.0;
        for col in start..=self.diagonal.length() {
            sum += self.u[(left, col)] * self.u[(right, col)];
        }
        sum
    }
}
