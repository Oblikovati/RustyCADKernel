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

//! Public `MathRoot::Utils` helpers (MathRoot_Utils.hxx and the Utils parts of the
//! Multiple, MultipleUtils, All and Trig headers) at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::math::Function;
use crate::math_root::{MultipleConfig, MultipleResult, find_all_roots};
use crate::math_utils::Config;
mod periodic;
pub use periodic::compute_periodic_root_count;
mod multiple;
pub use multiple::{
    MAX_NEWTON_POLISH_ITERATIONS, MULTIPLE_RELATIVE_INTERVAL_TOLERANCE, add_root,
    effective_x_tolerance, evaluate_shifted_value, evaluate_shifted_values, evaluate_value,
    is_valid_multiple_input, sort_roots,
};
mod adapters;
pub use adapters::{
    MultipleBrentValueWrapper, MultipleDerivativeValueWrapper, MultipleFunctionCell,
    MultipleGetRootValueFn, MultipleGetValueFn, MultipleIntervalHandler, MultipleNoExtraHandler,
    MultipleRootValueCallback, MultipleSampleCallback, MultipleSampleLookup, MultipleSampleValueFn,
};
mod refine;
pub use refine::refine_bracketed_root;
mod engine;
pub use engine::find_all_roots_impl;
pub(crate) use engine::multiple_sample_count;
mod derivative;
pub use derivative::find_all_roots_with_derivative_impl;
mod trig;
pub use trig::{
    TRIG_REFINEMENT_MAX_ITERATIONS, TRIG_REFINEMENT_MAX_STEP, TRIG_REFINEMENT_STEP_TOLERANCE,
    add_distinct_trig_root, add_periodic_trig_root, map_periodic_trig_root,
};

/// Finds the roots of `f - offset` on `[a, b]` with the given sampling and tolerances,
/// the building block of the null-interval search (`FindMultipleRoots` in MathRoot_All.hxx);
/// `find_multiple_roots(&mut f, 0.0, 1.0, 10, 1e-10, 1e-10, 0.0)`.
pub fn find_multiple_roots(
    function: &mut impl Function,
    a: f64,
    b: f64,
    nb_samples: usize,
    eps_x: f64,
    eps_f: f64,
    offset: f64,
) -> MultipleResult {
    let config: MultipleConfig = MultipleConfig {
        nb_samples,
        x_tolerance: eps_x,
        f_tolerance: eps_f,
        offset,
        ..Default::default()
    };
    find_all_roots(function, a, b, &config)
}

/// Reports whether a scalar root solver configuration is usable (`IsValidConfig`);
/// `is_valid_config(&Config::default())` is true.
pub fn is_valid_config(config: &Config) -> bool {
    config.max_iterations > 0
        && config.x_tolerance.is_finite()
        && config.x_tolerance > 0.0
        && config.f_tolerance.is_finite()
        && config.f_tolerance >= 0.0
}

/// Reports whether bounds form a finite, non-empty ordered interval (`IsValidBounds`);
/// `is_valid_bounds(0.0, 1.0)` is true.
pub fn is_valid_bounds(lower: f64, upper: f64) -> bool {
    lower.is_finite() && upper.is_finite() && lower < upper
}

/// Compares sign bits without multiplying, so signed zeros count as signs and nothing
/// overflows (`HaveOppositeSigns`); `have_opposite_signs(-0.0, 0.0)` is true.
pub fn have_opposite_signs(first: f64, second: f64) -> bool {
    first.is_sign_negative() != second.is_sign_negative()
}

/// Halves before adding so extreme endpoints cannot overflow (`Midpoint`);
/// `midpoint(f64::MAX, f64::MAX)` is `f64::MAX`.
pub fn midpoint(first: f64, second: f64) -> f64 {
    0.5 * first + 0.5 * second
}
