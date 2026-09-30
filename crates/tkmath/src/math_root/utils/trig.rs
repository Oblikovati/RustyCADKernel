//! Root bookkeeping of MathRoot_Trig.hxx (`MathRoot::Utils` part).
use super::compute_periodic_root_count;
use crate::math_root::TrigResult;
use std::f64::consts::TAU;

/// Iteration cap of the trigonometric Newton/Halley refinement
/// (`THE_TRIG_REFINEMENT_MAX_ITERATIONS`).
pub const TRIG_REFINEMENT_MAX_ITERATIONS: u32 = 20;

/// Step size below which the refinement stops (`THE_TRIG_REFINEMENT_STEP_TOLERANCE`).
pub const TRIG_REFINEMENT_STEP_TOLERANCE: f64 = 1.0e-14;

/// Largest refinement step, preventing overshoot (`THE_TRIG_REFINEMENT_MAX_STEP`).
pub const TRIG_REFINEMENT_MAX_STEP: f64 = 0.5;

/// Inserts a root in ascending order unless a root within `tolerance` is stored or the four
/// slots are full (`AddDistinctTrigRoot`); `add_distinct_trig_root(&mut result, 1.0, 1e-12)`.
pub fn add_distinct_trig_root(result: &mut TrigResult, root: f64, tolerance: f64) {
    let mut position: usize = 0;
    while position < result.nb_roots && result.roots[position] < root {
        if (root - result.roots[position]).abs() <= tolerance {
            return;
        }
        position += 1;
    }
    if (position < result.nb_roots && (root - result.roots[position]).abs() <= tolerance)
        || result.nb_roots == result.roots.len()
    {
        return;
    }
    for index in (position + 1..=result.nb_roots).rev() {
        result.roots[index] = result.roots[index - 1];
    }
    result.roots[position] = root;
    result.nb_roots += 1;
}

/// Maps a 2*PI-periodic root to its first representative in `[lower, upper]`, snapping
/// representatives within `tolerance` outside onto the bounds (`MapPeriodicTrigRoot`);
/// `map_periodic_trig_root(7.0, 0.0, 6.5, 1e-12, &mut mapped)` maps to `7 - 2*PI`.
pub fn map_periodic_trig_root(
    mut root: f64,
    lower: f64,
    upper: f64,
    tolerance: f64,
    mapped_root: &mut f64,
) -> bool {
    root += ((lower - root) / TAU).floor() * TAU;
    if root < lower - tolerance {
        root += TAU;
    }
    if root < lower {
        root = lower;
    }
    if root > upper + tolerance {
        return false;
    }
    if root > upper {
        root = upper;
    }
    let mut representatives: usize = 0;
    if !compute_periodic_root_count(root, upper, TAU, tolerance, false, &mut representatives)
        || representatives == 0
    {
        return false;
    }
    *mapped_root = root;
    true
}

/// Maps a periodic root into the interval and inserts it when present (`AddPeriodicTrigRoot`);
/// `add_periodic_trig_root(&mut result, 1.0, 0.0, 6.0, 1e-12)`.
pub fn add_periodic_trig_root(
    result: &mut TrigResult,
    root: f64,
    lower: f64,
    upper: f64,
    tolerance: f64,
) {
    let mut mapped: f64 = 0.0;
    if map_periodic_trig_root(root, lower, upper, tolerance, &mut mapped) {
        add_distinct_trig_root(result, mapped, tolerance);
    }
}
