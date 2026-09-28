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

//! Periodic root counts preserving MathRoot_Utils.hxx long-double arithmetic.
use crate::math_poly::extended::Extended as E;
fn valid_periodic(first: f64, upper: f64, period: f64, tolerance: f64) -> bool {
    first.is_finite()
        && upper.is_finite()
        && period.is_finite()
        && period > 0.0
        && tolerance.is_finite()
        && tolerance >= 0.0
}
fn inside(value: f64, limit: f64, exclude: bool) -> bool {
    if exclude {
        value < limit
    } else {
        value <= limit
    }
}
fn initial_count(first: f64, limit: f64, period: f64, exclude: bool) -> Option<usize> {
    let quotient: E = (E::new(limit) - E::new(first)) / E::new(period);
    let count: E = if exclude {
        quotient.ceil()
    } else {
        quotient.floor() + E::new(1.0)
    };
    if count < E::new(1.0) || count >= E::from_usize(usize::MAX) {
        return None;
    }
    Some(count.to_usize())
}
fn adjust_count(first: f64, limit: f64, period: f64, exclude: bool, count: &mut usize) {
    let last: f64 = first + (*count - 1) as f64 * period;
    if !inside(last, limit, exclude) {
        *count -= 1;
    }
    let next: f64 = first + *count as f64 * period;
    if *count < usize::MAX - 1 && inside(next, limit, exclude) {
        *count += 1;
    }
}
fn representable_steps(first: f64, period: f64, count: usize) -> bool {
    if count == 0 {
        return true;
    }
    let last: f64 = first + (count - 1) as f64 * period;
    if count < 2 || first + period <= first {
        return count < 2;
    }
    let previous: f64 = first + (count - 2) as f64 * period;
    last.is_finite() && last > previous
}
/// Counts safely representable periodic roots; `compute_periodic_root_count(0.0,6.0,2.0,0.0,false,&mut count)`.
pub fn compute_periodic_root_count(
    first: f64,
    upper: f64,
    period: f64,
    tolerance: f64,
    exclude: bool,
    count: &mut usize,
) -> bool {
    *count = 0;
    if !valid_periodic(first, upper, period, tolerance) {
        return false;
    }
    let limit: f64 = if exclude {
        upper - tolerance
    } else {
        upper + tolerance
    };
    if !limit.is_finite() {
        return false;
    }
    if !inside(first, limit, exclude) {
        return true;
    }
    let Some(initial) = initial_count(first, limit, period, exclude) else {
        return false;
    };
    *count = initial;
    adjust_count(first, limit, period, exclude, count);
    representable_steps(first, period, *count)
}
