// Copyright (c) 2025 OPEN CASCADE SAS
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

use super::*;
fn owned_matrix(source: &Matrix<'_>) -> Matrix<'static> {
    let mut result: Matrix<'static> = Matrix::with_bounds(
        source.lower_row(),
        source.upper_row(),
        source.lower_col(),
        source.upper_col(),
    )
    .unwrap();
    result.initialized(source).unwrap();
    result
}
fn owned_vector(source: &Vector<'_>) -> Vector<'static> {
    let mut result: Vector<'static> = Vector::with_bounds(source.lower(), source.upper()).unwrap();
    result.initialized(source).unwrap();
    result
}
/// Native quadratic form copies its coefficients; `QuadraticForm::new(&a,&b,c)`.
#[derive(Clone)]
pub struct QuadraticForm {
    a: Matrix<'static>,
    b: Vector<'static>,
    c: f64,
}
impl QuadraticForm {
    /// Owns copies including original bounds; `QuadraticForm::new(&a,&b,0.0)`.
    pub fn new(a: &Matrix<'_>, b: &Vector<'_>, c: f64) -> Self {
        Self {
            a: owned_matrix(a),
            b: owned_vector(b),
            c,
        }
    }
}
impl MultipleVarFunction for QuadraticForm {
    fn variables(&self) -> i32 {
        self.a.col_number()
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        *value = self.c;
        for i in 0..point.length() as usize {
            for j in 0..point.length() as usize {
                *value += point.at(i).unwrap() * self.a.at(i, j).unwrap() * point.at(j).unwrap();
            }
        }
        for i in 0..point.length() as usize {
            *value += self.b.at(i).unwrap() * point.at(i).unwrap();
        }
        true
    }
}
impl MultipleVarFunctionWithGradient for QuadraticForm {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        for i in 0..point.length() as usize {
            *gradient.change_at(i).unwrap() = *self.b.at(i).unwrap();
            for j in 0..point.length() as usize {
                *gradient.change_at(i).unwrap() +=
                    (self.a.at(i, j).unwrap() + self.a.at(j, i).unwrap()) * point.at(j).unwrap();
            }
        }
        true
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        self.value(point, value) && self.gradient(point, gradient)
    }
}
/// Native squared linear residual; `LinearResidual::new(&a,&b)`.
#[derive(Clone)]
pub struct LinearResidual {
    a: Matrix<'static>,
    b: Vector<'static>,
}
impl LinearResidual {
    /// Copies the matrix and right-hand side; `LinearResidual::new(&a,&b)`.
    pub fn new(a: &Matrix<'_>, b: &Vector<'_>) -> Self {
        Self {
            a: owned_matrix(a),
            b: owned_vector(b),
        }
    }
    fn residual(&self, point: &Vector<'_>, row: usize) -> f64 {
        let mut residual: f64 = -self.b.at(row).unwrap();
        for column in 0..self.a.col_number() as usize {
            residual += self.a.at(row, column).unwrap() * point.at(column).unwrap();
        }
        residual
    }
}
impl MultipleVarFunction for LinearResidual {
    fn variables(&self) -> i32 {
        self.a.col_number()
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        *value = 0.0;
        for row in 0..self.a.row_number() as usize {
            let residual: f64 = self.residual(point, row);
            *value += residual * residual;
        }
        true
    }
}
impl MultipleVarFunctionWithGradient for LinearResidual {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        let residuals: Vec<f64> = (0..self.a.row_number() as usize)
            .map(|row| self.residual(point, row))
            .collect();
        for column in 0..self.a.col_number() as usize {
            *gradient.change_at(column).unwrap() = 0.0;
            for (row, residual) in residuals.iter().enumerate() {
                *gradient.change_at(column).unwrap() +=
                    2.0 * self.a.at(row, column).unwrap() * residual;
            }
        }
        true
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        self.value(point, value) && self.gradient(point, gradient)
    }
}
