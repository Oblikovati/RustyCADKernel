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

use super::{FunctionRoots, FunctionWithDerivative, Search};
impl FunctionRoots {
    pub(super) fn sample(&mut self, f: &mut impl FunctionWithDerivative, s: Search) {
        let mut samples: Vec<f64> = Vec::with_capacity(s.count + 1);
        let mut x: f64 = s.lower;
        let mut value: f64 = 0.0;
        for _ in 0..=s.count {
            if x > s.upper {
                x = s.upper;
            }
            if f.value(x, &mut value) {
                samples.push(value - s.target);
            }
            x += s.dx;
        }
        self.done = samples.len() == s.count + 1;
        if !self.done {
            return;
        }
        self.all_null = !samples.iter().any(|&v| v > s.null || v < -s.null);
        if self.all_null {
            return;
        }
        self.crossings(f, s, &samples);
        self.zeros(f, s, &samples);
        self.endpoints(f, s, &samples);
        self.extrema(f, s, &samples);
    }
    fn crossings(&mut self, f: &mut impl FunctionWithDerivative, s: Search, values: &[f64]) {
        let mut x: f64 = s.lower;
        for pair in values.windows(2) {
            let mut next: f64 = x + s.dx;
            if next > s.upper {
                next = s.upper;
            }
            let crosses: bool = if pair[0] < 0.0 {
                pair[1] > 0.0
            } else {
                pair[1] < 0.0
            };
            if crosses {
                self.solve(f, s, [x, pair[0]], [next, pair[1]]);
            }
            x += s.dx;
        }
    }
    fn zeros(&mut self, f: &mut impl FunctionWithDerivative, s: Search, values: &[f64]) {
        for (index, &value) in values.iter().enumerate() {
            if value != 0.0 {
                continue;
            }
            let x: f64 = s.clamp(s.lower + index as f64 * s.dx);
            self.sample_zero(f, s, x);
        }
    }
    fn sample_zero(&mut self, f: &mut impl FunctionWithDerivative, s: Search, x: f64) {
        // OCCT 3d097a0 evaluates two points on the same side of the sample.
        let offset: f64 = s.dx * 0.5;
        let u0: f64 = s.clamp(offset + x);
        let u1: f64 = s.clamp(x + offset);
        let (mut y0, mut y1): (f64, f64) = (0.0, 0.0);
        f.value(u0, &mut y0);
        y0 -= s.target;
        f.value(u1, &mut y1);
        y1 -= s.target;
        if y0 * y1 < 0.0 {
            self.solve(f, s, [u0, y0], [u1, y1]);
        } else if y0 != 0.0 || y1 != 0.0 {
            self.append(f, x, s.duplicate);
        }
    }
    fn endpoints(&mut self, f: &mut impl FunctionWithDerivative, s: Search, values: &[f64]) {
        if values[0] <= s.value && values[0] >= -s.value {
            self.append(f, s.lower, s.duplicate);
        }
        if values[s.count] <= s.value && values[s.count] >= -s.value {
            self.append(f, s.upper, s.duplicate);
        }
    }
}
