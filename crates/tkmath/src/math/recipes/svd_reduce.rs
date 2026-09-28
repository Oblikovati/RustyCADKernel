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

//! Householder reduction to bidiagonal form, the first SVD_Decompose loop.
use super::svd::{SvdWork, sign};
impl SvdWork<'_, '_, '_, '_, '_> {
    /// Returns the native `anorm` bound used by the convergence tests.
    pub(super) fn bidiagonalize(&mut self) -> f64 {
        let mut norm: f64 = 0.0;
        let mut scale: f64 = 0.0;
        let mut g: f64 = 0.0;
        for index in 1..=self.n {
            self.rv1[index] = scale * g;
            self.w[index] = self.reduce_column(index);
            (scale, g) = self.reduce_row(index);
            let mut aw: f64 = self.w[index];
            if aw < 0.0 {
                aw = -aw;
            }
            let ar: f64 = if self.rv1[index] > 0.0 {
                aw + self.rv1[index]
            } else {
                aw - self.rv1[index]
            };
            if norm < ar {
                norm = ar;
            }
        }
        norm
    }
    /// Left Householder step on column `index`; returns `scale * g`, the new `w(index)`.
    fn reduce_column(&mut self, index: i32) -> f64 {
        let mut scale: f64 = 0.0;
        for row in index..=self.m {
            let entry: f64 = self.a[(row, index)];
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
        for row in index..=self.m {
            self.a[(row, index)] /= scale;
            sum += self.a[(row, index)] * self.a[(row, index)];
        }
        let f: f64 = self.a[(index, index)];
        let g: f64 = -sign(sum.sqrt(), f);
        let h: f64 = f * g - sum;
        self.a[(index, index)] = f - g;
        self.project_columns(index, h);
        for row in index..=self.m {
            self.a[(row, index)] *= scale;
        }
        scale * g
    }
    fn project_columns(&mut self, index: i32, h: f64) {
        for col in index + 1..=self.n {
            let f: f64 = self.column_dot(index, col, index) / h;
            for row in index..=self.m {
                self.a[(row, col)] += f * self.a[(row, index)];
            }
        }
    }
    /// Sum over rows `start..=m` of `a(row, left) * a(row, right)`.
    pub(super) fn column_dot(&self, left: i32, right: i32, start: i32) -> f64 {
        let mut sum: f64 = 0.0;
        for row in start..=self.m {
            sum += self.a[(row, left)] * self.a[(row, right)];
        }
        sum
    }
    /// Right Householder step on row `index`; returns the `(scale, g)` carried into `rv1`.
    /// With `index == n` the native `i != n` guard skips it, which the empty ranges mirror.
    fn reduce_row(&mut self, index: i32) -> (f64, f64) {
        let mut scale: f64 = 0.0;
        for col in index + 1..=self.n {
            let entry: f64 = self.a[(index, col)];
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
        for col in index + 1..=self.n {
            self.a[(index, col)] /= scale;
            sum += self.a[(index, col)] * self.a[(index, col)];
        }
        let f: f64 = self.a[(index, index + 1)];
        let g: f64 = -sign(sum.sqrt(), f);
        let h: f64 = f * g - sum;
        self.a[(index, index + 1)] = f - g;
        for col in index + 1..=self.n {
            self.rv1[col] = self.a[(index, col)] / h;
        }
        self.project_rows(index);
        for col in index + 1..=self.n {
            self.a[(index, col)] *= scale;
        }
        (scale, g)
    }
    fn project_rows(&mut self, index: i32) {
        // The native `i != m` guard only skips this empty range.
        for row in index + 1..=self.m {
            let sum: f64 = self.row_dot(row, index, index + 1);
            for col in index + 1..=self.n {
                self.a[(row, col)] += sum * self.rv1[col];
            }
        }
    }
    fn row_dot(&self, left: i32, right: i32, start: i32) -> f64 {
        let mut sum: f64 = 0.0;
        for col in start..=self.n {
            sum += self.a[(left, col)] * self.a[(right, col)];
        }
        sum
    }
}
