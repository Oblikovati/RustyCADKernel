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

use super::{DirectPolynomialRoots, ZERO, scaling::scale_factor};
pub(super) struct Cubic {
    pub beta: f64,
    pub gamma: f64,
    pub delta: f64,
    pub p: f64,
    pub q: f64,
    pub discriminant: f64,
}
impl DirectPolynomialRoots {
    /// Repeats native protected cubic Solve with scaling; `solver.solve_cubic(1.0,-6.0,11.0,-6.0)`.
    pub fn solve_cubic(&mut self, a: f64, b: f64, c: f64, d: f64) {
        if a.abs() <= ZERO {
            self.solve_quadratic(b, c, d);
            return;
        }
        let (beta, gamma, delta): (f64, f64, f64) = (b / a, c / a, d / a);
        let scale: f64 = scale_factor(delta, 3);
        let square: f64 = scale * scale;
        let cubic: Cubic = Cubic::new(beta / scale, gamma / square, delta / (square * scale));
        if cubic.p.abs() > 1e80 {
            self.done = false;
            return;
        }
        self.count = if cubic.discriminant > 0.0 { 1 } else { 3 };
        cubic.solve(&mut self.roots);
        self.scale_refine(scale, &[a, b, c, d]);
    }
}
impl Cubic {
    fn new(beta: f64, gamma: f64, delta: f64) -> Self {
        let p1: f64 = gamma;
        let p2: f64 = -(beta * beta) / 3.0;
        let mut p: f64 = p1 + p2;
        let ep: f64 = 5.0 * f64::EPSILON * (p1.abs() + p2.abs());
        if p.abs() <= ep {
            p = 0.0;
        }
        let q1: f64 = delta;
        let q2: f64 = -beta * gamma / 3.0;
        let q3: f64 = 2.0 * (beta * beta * beta) / 27.0;
        let mut q: f64 = q1 + q2 + q3;
        let eq: f64 = 10.0 * f64::EPSILON * (q1.abs() + q2.abs() + q3.abs());
        if q.abs() <= eq {
            q = 0.0;
        }
        let a1: f64 = (p * p * p) / 27.0;
        let a2: f64 = (q * q) / 4.0;
        let discriminant: f64 = if p < 0.0 {
            special_discriminant(beta, gamma, delta, a1)
        } else {
            a1 + a2
        };
        Self {
            beta,
            gamma,
            delta,
            p,
            q,
            discriminant,
        }
    }
}
fn special_discriminant(beta: f64, gamma: f64, delta: f64, a1: f64) -> f64 {
    let sigma: f64 = beta * gamma / 3.0 - 2.0 * beta * beta * beta / 27.0;
    let psi: f64 = gamma * gamma * (4.0 * gamma - beta * beta) / 27.0;
    let d1: f64 = if sigma >= 0.0 {
        sigma + 2.0 * (-a1).sqrt()
    } else {
        sigma - 2.0 * (-a1).sqrt()
    };
    let d2: f64 = psi / d1;
    if (delta - d1).abs() >= 18.0 * f64::EPSILON * (delta.abs() + d1.abs())
        && (delta - d2).abs() >= 24.0 * f64::EPSILON * (delta.abs() + d2.abs())
    {
        return (delta - d1) * (delta - d2) / 4.0;
    }
    0.0
}
