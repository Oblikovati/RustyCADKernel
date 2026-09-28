// Created on: 2005-12-21
// Created by: Julia GERASIMOVA
// Copyright (c) 2005-2014 OPEN CASCADE SAS
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

use rustycad_tkernel::ncollection::Array1;
struct Recurrence {
    n: i32,
    a: Array1<'static, f64>,
    b: Array1<'static, f64>,
    s: Array1<'static, f64>,
    t: Array1<'static, f64>,
}
pub(super) fn coefficients(n: i32, size: i32) -> (Array1<'static, f64>, Array1<'static, f64>) {
    let mut recurrence: Recurrence = Recurrence::new(n, size);
    recurrence.eastward();
    for index in (0..=n / 2).rev() {
        recurrence.s[index] = recurrence.s[index - 1];
    }
    recurrence.southward();
    recurrence.a[2 * n] =
        recurrence.a[n - 1] - recurrence.b[2 * n] * recurrence.s[0] / recurrence.t[0];
    let mut diagonal: Array1<'static, f64> = Array1::with_bounds(1, size).unwrap();
    let mut subdiagonal: Array1<'static, f64> = Array1::with_bounds(1, size).unwrap();
    for index in 1..=size {
        diagonal[index] = recurrence.a[index - 1];
        subdiagonal[index] = recurrence.b[index - 1].sqrt();
    }
    (diagonal, subdiagonal)
}
impl Recurrence {
    fn new(n: i32, size: i32) -> Self {
        let mut result: Self = Self {
            n,
            a: Array1::with_bounds(0, size - 1).unwrap(),
            b: Array1::with_bounds(0, size - 1).unwrap(),
            s: Array1::with_bounds(-1, n / 2).unwrap(),
            t: Array1::with_bounds(-1, n / 2).unwrap(),
        };
        let end: i32 = (3 * (n + 1) / 2 + 1).min(size);
        for index in 2..=end {
            let square: i32 = (index - 1) * (index - 1);
            result.b[index - 1] = f64::from(square) / (4.0 * f64::from(square) - 1.0);
        }
        result.t[0] = result.b[n + 1];
        result
    }
    fn eastward(&mut self) {
        for m in 0..=self.n - 2 {
            let mut sum: f64 = 0.0;
            for k in (0..=(m + 1) / 2).rev() {
                sum += self.east_term(m, k);
                self.s[k] = sum;
            }
            std::mem::swap(&mut self.s, &mut self.t);
        }
    }
    fn east_term(&self, m: i32, k: i32) -> f64 {
        let l: i32 = m - k;
        let offset: i32 = k + self.n + 1;
        (self.a[offset] - self.a[l]) * self.t[k] + self.b[offset] * self.s[k - 1]
            - self.b[l] * self.s[k]
    }
    fn southward(&mut self) {
        let mut j: i32 = -1;
        for m in self.n - 1..=2 * self.n - 3 {
            let mut sum: f64 = 0.0;
            for k in m + 1 - self.n..=(m - 1) / 2 {
                let l: i32 = m - k;
                j = self.n - 1 - l;
                sum += self.south_term(k, l, j);
                self.s[j] = sum;
            }
            self.update_coefficient(m, j);
            std::mem::swap(&mut self.s, &mut self.t);
        }
    }
    fn south_term(&self, k: i32, l: i32, j: i32) -> f64 {
        let offset: i32 = k + self.n + 1;
        -(self.a[offset] - self.a[l]) * self.t[j] - self.b[offset] * self.s[j]
            + self.b[l] * self.s[j + 1]
    }
    fn update_coefficient(&mut self, m: i32, j: i32) {
        if m % 2 == 0 {
            let k: i32 = m / 2;
            let offset: i32 = k + self.n + 1;
            self.a[offset] =
                self.a[k] + (self.s[j] - self.b[offset] * self.s[j + 1]) / self.t[j + 1];
        } else {
            let k: i32 = (m + 1) / 2;
            self.b[k + self.n + 1] = self.s[j] / self.s[j + 1];
        }
    }
}
