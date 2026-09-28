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

use super::{
    DirectPolynomialRoots, ZERO, ferrari,
    scaling::{epsilon, scale_factor},
};
impl DirectPolynomialRoots {
    /// Repeats native protected quartic Solve and its degree-reduction rules; `solver.solve_quartic(1.0,0.0,-5.0,0.0,4.0)`.
    pub fn solve_quartic(&mut self, a: f64, b: f64, c: f64, d: f64, e: f64) {
        if reduce_degree(a, [b, c, d, e]) {
            self.solve_cubic(b, c, d, e);
            return;
        }
        let normalized: [f64; 4] = [b / a, c / a, d / a, e / a];
        let scale: f64 = scale_factor(normalized[3], 4);
        let square: f64 = scale * scale;
        let scaled: [f64; 4] = [
            normalized[0] / scale,
            normalized[1] / square,
            normalized[2] / (square * scale),
            normalized[3] / (square * square),
        ];
        let Some(resolvent) = ferrari::resolvent(scaled) else {
            self.done = false;
            return;
        };
        let [p1, q1, p2, q2]: [f64; 4] = ferrari::factor(scaled, resolvent);
        let first: Self = Self::quadratic(1.0, p1, q1);
        if !first.done {
            self.done = false;
            return;
        }
        let second: Self = Self::quadratic(1.0, p2, q2);
        if !second.done {
            self.done = false;
            return;
        }
        self.collect(&first, &second);
        self.scale_refine(scale, &[a, b, c, d, e]);
    }
    fn collect(&mut self, first: &Self, second: &Self) {
        self.count = first.count + second.count;
        for index in 0..first.count as usize {
            self.roots[index] = first.roots[index];
        }
        for index in 0..second.count as usize {
            self.roots[first.count as usize + index] = second.roots[index];
        }
    }
}
fn reduce_degree(a: f64, coefficients: [f64; 4]) -> bool {
    if a.abs() <= ZERO {
        return true;
    }
    let mut maximum: f64 = ZERO;
    for value in coefficients {
        if maximum < value.abs() {
            maximum = value.abs();
        }
    }
    if maximum > ZERO {
        maximum = epsilon(100.0 * maximum);
    }
    if a.abs()
        .partial_cmp(&maximum)
        .is_none_or(|order| order == std::cmp::Ordering::Greater)
    {
        return false;
    }
    let threshold: f64 = 1000.0 * maximum;
    !coefficients
        .iter()
        .any(|value| value.abs() > ZERO && value.abs() <= threshold)
}
