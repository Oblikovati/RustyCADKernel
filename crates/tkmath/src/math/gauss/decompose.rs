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

use super::Gauss;
impl Gauss {
    pub(super) fn decompose(&mut self, minimum: f64) -> bool {
        let Some(mut scales) = self.row_scales(minimum) else {
            return false;
        };
        for column in 1..=self.lu.row_number() {
            self.upper_column(column);
            let pivot = self.lower_column(column, &scales);
            self.exchange_pivot(column, pivot, &mut scales);
            self.pivots[(column - 1) as usize] = pivot;
            if self.lu[(column, column)].abs() <= minimum {
                return false;
            }
            self.scale_column(column);
        }
        true
    }
    fn row_scales(&self, minimum: f64) -> Option<Vec<f64>> {
        let mut scales = Vec::with_capacity(self.lu.row_number() as usize);
        for row in 1..=self.lu.row_number() {
            let mut biggest = 0.0;
            for col in 1..=self.lu.col_number() {
                let value = self.lu[(row, col)].abs();
                if value > biggest {
                    biggest = value;
                }
            }
            if biggest <= minimum {
                return None;
            }
            scales.push(1.0 / biggest);
        }
        Some(scales)
    }
    fn upper_column(&mut self, column: i32) {
        for row in 1..column {
            self.lu[(row, column)] = self.column_sum(row, column, row);
        }
    }
    fn column_sum(&self, row: i32, column: i32, end: i32) -> f64 {
        let mut sum = self.lu[(row, column)];
        for k in 1..end {
            sum -= self.lu[(row, k)] * self.lu[(k, column)];
        }
        sum
    }
    fn lower_column(&mut self, column: i32, scales: &[f64]) -> i32 {
        let mut biggest = 0.0;
        let mut pivot = column;
        for row in column..=self.lu.row_number() {
            let sum = self.column_sum(row, column, column);
            self.lu[(row, column)] = sum;
            let candidate = scales[(row - 1) as usize] * sum.abs();
            // OCC25559: unordered candidates must update the pivot, as in native code.
            if candidate < biggest {
                continue;
            }
            biggest = candidate;
            pivot = row;
        }
        pivot
    }
    fn exchange_pivot(&mut self, column: i32, pivot: i32, scales: &mut [f64]) {
        if column == pivot {
            return;
        }
        for k in 1..=self.lu.row_number() {
            let temporary = self.lu[(pivot, k)];
            self.lu[(pivot, k)] = self.lu[(column, k)];
            self.lu[(column, k)] = temporary;
        }
        self.sign = -self.sign;
        scales[(pivot - 1) as usize] = scales[(column - 1) as usize];
    }
    fn scale_column(&mut self, column: i32) {
        if column == self.lu.row_number() {
            return;
        }
        let factor = 1.0 / self.lu[(column, column)];
        for row in column + 1..=self.lu.row_number() {
            self.lu[(row, column)] *= factor;
        }
    }
}
