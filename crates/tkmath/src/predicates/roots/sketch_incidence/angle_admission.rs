// SPDX-License-Identifier: GPL-2.0-only
//! Admission of angle-bounded conic arcs.
use super::angle::{MAX_ANGLE, pi};
use super::failure::SketchIncidenceCause as Cause;
use super::input::SketchConicAngles;
use super::rational_polynomial::ratio;
use super::validate::{finite, nonzero};
use crate::predicates::{OrientationSign, orient2d};
use std::cmp::Ordering;

/// Finite bounds `start < end` of magnitude at most 64 whose span is below one full
/// turn, and for an ellipse an axis parallel to its major axis.
pub(super) fn angles(
    angles: &SketchConicAngles,
    major_axis: Option<[f64; 2]>,
) -> Result<(), Cause> {
    nonzero("angles.axis", angles.axis)?;
    finite("angles.bounds", &angles.bounds)?;
    let [start, end] = angles.bounds;
    let refused = Cause::AngleBounds {
        bounds: angles.bounds,
    };
    if start.abs() > MAX_ANGLE || end.abs() > MAX_ANGLE || start >= end {
        return Err(refused);
    }
    if !below_full_turn(angles.bounds) {
        return Err(refused);
    }
    let Some(major) = major_axis else {
        return Ok(());
    };
    if orient2d([0., 0.], angles.axis, major).expect("finite axes") != OrientationSign::Zero {
        return Err(Cause::AngleAxis {
            axis: angles.axis,
            major_axis: major,
        });
    }
    Ok(())
}
// The exact rational span is compared with 2 pi, refining pi until they separate;
// they never coincide because pi is irrational.
fn below_full_turn([start, end]: [f64; 2]) -> bool {
    let span = ratio(end).subtract(&ratio(start));
    let two = ratio(2.);
    let mut bits = 64;
    loop {
        let turn = pi(bits);
        if span < turn.lower.multiply(&two) {
            return true;
        }
        if span > turn.upper.multiply(&two) {
            return false;
        }
        bits *= 2;
    }
}
/// Exact order of a rational span with one half turn.
pub(super) fn compare_half_turn([start, end]: [f64; 2]) -> Ordering {
    let span = ratio(end).subtract(&ratio(start));
    let mut bits = 64;
    loop {
        let half = pi(bits);
        if span < half.lower {
            return Ordering::Less;
        }
        if span > half.upper {
            return Ordering::Greater;
        }
        bits *= 2;
    }
}
