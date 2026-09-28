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

use super::{Matrix, Svd, Vector, pythag, sign};
impl Svd {
    pub(super) fn diagonalize(&mut self, residual: &mut Vector<'_>, norm: f64) -> bool {
        for index in (1..=self.diagonal.length()).rev() {
            if !self.converge(residual, norm, index) {
                return false;
            }
        }
        true
    }
    fn converge(&mut self, residual: &mut Vector<'_>, norm: f64, index: i32) -> bool {
        for iteration in 1..=30 {
            let (start, cancel): (i32, bool) = self.split(residual, norm, index);
            if cancel {
                self.cancel(residual, norm, start, index);
            }
            if start == index {
                self.positive_value(index);
                return true;
            }
            if iteration == 30 {
                return false;
            }
            self.qr_step(residual, start, index);
        }
        false
    }
    fn split(&self, residual: &Vector<'_>, norm: f64, index: i32) -> (i32, bool) {
        for start in (1..=index).rev() {
            if residual[start].abs() + norm == norm {
                return (start, false);
            }
            if start == 1 || self.diagonal[start - 1].abs() + norm == norm {
                return (start, true);
            }
        }
        (1, true)
    }
    fn cancel(&mut self, residual: &Vector<'_>, norm: f64, start: i32, end: i32) {
        let mut sine: f64 = 1.0;
        for index in start..=end {
            let f: f64 = sine * residual[index];
            if f.abs() + norm == norm {
                continue;
            }
            let g: f64 = self.diagonal[index];
            let h: f64 = pythag(f, g);
            self.diagonal[index] = h;
            let reciprocal: f64 = 1.0 / h;
            let cosine: f64 = g * reciprocal;
            sine = -f * reciprocal;
            rotate_columns(&mut self.u, start - 1, index, cosine, sine);
        }
    }
    fn positive_value(&mut self, index: i32) {
        if self.diagonal[index] >= 0.0 {
            return;
        }
        // Native uses a strict negative comparison, leaving NaN unchanged.
        if self.diagonal[index] < 0.0 {
            self.diagonal[index] = -self.diagonal[index];
            for row in 1..=self.diagonal.length() {
                self.v[(row, index)] = -self.v[(row, index)];
            }
        }
    }
    fn shift(&self, residual: &Vector<'_>, start: i32, end: i32) -> f64 {
        let x: f64 = self.diagonal[start];
        let y: f64 = self.diagonal[end - 1];
        let z: f64 = self.diagonal[end];
        let g: f64 = residual[end - 1];
        let h: f64 = residual[end];
        let f: f64 = ((y - z) * (y + z) + (g - h) * (g + h)) / (2.0 * h * y);
        let magnitude: f64 = pythag(f, 1.0);
        ((x - z) * (x + z) + h * ((y / (f + sign(magnitude, f))) - h)) / x
    }
    fn qr_step(&mut self, residual: &mut Vector<'_>, start: i32, end: i32) {
        let mut state: QrState = QrState {
            x: self.diagonal[start],
            f: self.shift(residual, start, end),
            cosine: 1.0,
            sine: 1.0,
        };
        for index in start..end {
            self.qr_pair(residual, index, &mut state);
        }
        residual[start] = 0.0;
        residual[end] = state.f;
        self.diagonal[end] = state.x;
    }
    fn qr_pair(&mut self, residual: &mut Vector<'_>, index: i32, state: &mut QrState) {
        let next: i32 = index + 1;
        let mut g: f64 = residual[next];
        let mut y: f64 = self.diagonal[next];
        let mut h: f64 = state.sine * g;
        g *= state.cosine;
        let mut z: f64 = pythag(state.f, h);
        residual[index] = z;
        state.cosine = state.f / z;
        state.sine = h / z;
        state.f = state.x * state.cosine + g * state.sine;
        g = g * state.cosine - state.x * state.sine;
        h = y * state.sine;
        y *= state.cosine;
        rotate_columns(&mut self.v, index, next, state.cosine, state.sine);
        z = pythag(state.f, h);
        self.diagonal[index] = z;
        if z != 0.0 {
            z = 1.0 / z;
            state.cosine = state.f * z;
            state.sine = h * z;
        }
        state.f = (state.cosine * g) + (state.sine * y);
        state.x = (state.cosine * y) - (state.sine * g);
        rotate_columns(&mut self.u, index, next, state.cosine, state.sine);
    }
}
struct QrState {
    x: f64,
    f: f64,
    cosine: f64,
    sine: f64,
}
fn rotate_columns(matrix: &mut Matrix<'_>, left: i32, right: i32, cosine: f64, sine: f64) {
    for row in 1..=matrix.row_number() {
        let y: f64 = matrix[(row, left)];
        let z: f64 = matrix[(row, right)];
        matrix[(row, left)] = y * cosine + z * sine;
        matrix[(row, right)] = z * cosine - y * sine;
    }
}
