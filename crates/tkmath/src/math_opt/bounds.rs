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

//! Shared native box-constraint operations for BFGS and Newton.
use super::*;
pub(super) struct BoxBounds<'a> {
    pub(super) lower: &'a Vector<'a>,
    pub(super) upper: &'a Vector<'a>,
}
impl BoxBounds<'_> {
    pub(super) fn valid(&self, n: i32) -> bool {
        if self.lower.length() != n || self.upper.length() != n || n == 0 {
            return false;
        }
        for i in 0..n as usize {
            let lo: f64 = *self.lower.at(i).unwrap();
            let hi: f64 = *self.upper.at(i).unwrap();
            if !lo.is_finite() || !hi.is_finite() || lo >= hi || !(hi - lo).is_finite() {
                return false;
            }
        }
        true
    }
    pub(super) fn clamp(&self, x: &mut Vector<'_>) {
        for i in 0..x.length() as usize {
            let lo: f64 = *self.lower.at(i).unwrap();
            let hi: f64 = *self.upper.at(i).unwrap();
            let v: &mut f64 = x.change_at(i).unwrap();
            if *v < lo {
                *v = lo;
            }
            if *v > hi {
                *v = hi;
            }
        }
    }
    pub(super) fn project(&self, x: &Vector<'_>, v: &mut Vector<'_>, gradient: bool) {
        for i in 0..x.length() as usize {
            let value: f64 = *x.at(i).unwrap();
            let tol: f64 = f64::EPSILON * value.abs().max(1.0);
            let component: &mut f64 = v.change_at(i).unwrap();
            let lo: f64 = *self.lower.at(i).unwrap();
            let hi: f64 = *self.upper.at(i).unwrap();
            let sign: f64 = if gradient { 1.0 } else { -1.0 };
            if value - lo < tol && *component * sign > 0.0 {
                *component = 0.0;
            }
            if hi - value < tol && *component * sign < 0.0 {
                *component = 0.0;
            }
        }
    }
    pub(super) fn alpha(&self, x: &Vector<'_>, dir: &Vector<'_>) -> f64 {
        let mut alpha: f64 = 1.0;
        for i in 0..x.length() as usize {
            let d: f64 = *dir.at(i).unwrap();
            let value: f64 = *x.at(i).unwrap();
            if d < -f64::EPSILON {
                alpha = alpha.min((self.lower.at(i).unwrap() - value) / d);
            } else if d > f64::EPSILON {
                alpha = alpha.min((self.upper.at(i).unwrap() - value) / d);
            }
        }
        alpha
    }
}
