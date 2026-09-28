// Copyright (c) 2026 OPEN CASCADE SAS
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

//! Option and box-bound helpers of MathSys_Newton2D/3D/4D.hxx (`IsOptionsValid*`,
//! `IsBoundsValid*D`, `MaxStep*D`, `Clamp*D`) at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Upstream spells the three dimensions out separately with identical bodies; each public
//! wrapper keeps its native name and delegates to one const-generic implementation.
use super::checks::{is_finite_array, std_max};
use super::{NewtonBoundsN, NewtonOptions};

/// Validates the fields the 2D solvers read (`IsOptionsValid`); `is_options_valid(&NewtonOptions::default())`.
pub fn is_options_valid(options: &NewtonOptions) -> bool {
    let finite_positive = |value: f64| value.is_finite() && value > 0.0;
    finite_positive(options.solver.f_tolerance)
        && finite_positive(options.solver.x_tolerance)
        && options.solver.max_iterations > 0
        && finite_positive(options.max_step_ratio)
        && options.soft_bounds_extension.is_finite()
        && options.soft_bounds_extension >= 0.0
}
/// 3D spelling of [`is_options_valid`] (`IsOptionsValid3D`); `is_options_valid_3d(&options)`.
pub fn is_options_valid_3d(options: &NewtonOptions) -> bool {
    is_options_valid(options)
}
/// 4D spelling of [`is_options_valid`] (`IsOptionsValid4D`); `is_options_valid_4d(&options)`.
pub fn is_options_valid_4d(options: &NewtonOptions) -> bool {
    is_options_valid(options)
}

/// Inactive bounds are valid; active ones need finite `min <= max`; `is_bounds_valid_2d(&bounds)`.
pub fn is_bounds_valid_2d(bounds: &NewtonBoundsN<2>) -> bool {
    bounds_valid(bounds)
}
/// 3D bounds check (`IsBoundsValid3D`); `is_bounds_valid_3d(&bounds)`.
pub fn is_bounds_valid_3d(bounds: &NewtonBoundsN<3>) -> bool {
    bounds_valid(bounds)
}
/// 4D bounds check (`IsBoundsValid4D`); `is_bounds_valid_4d(&bounds)`.
pub fn is_bounds_valid_4d(bounds: &NewtonBoundsN<4>) -> bool {
    bounds_valid(bounds)
}
pub(super) fn bounds_valid<const N: usize>(bounds: &NewtonBoundsN<N>) -> bool {
    !bounds.has_bounds
        || (is_finite_array(&bounds.min)
            && is_finite_array(&bounds.max)
            && (0..N).all(|i| bounds.min[i] <= bounds.max[i]))
}

/// Step limit `ratio * max(1, widest side)`, none when unbounded; `max_step_2d(&bounds, 0.5)`.
pub fn max_step_2d(bounds: &NewtonBoundsN<2>, max_step_ratio: f64) -> Option<f64> {
    max_step(bounds, max_step_ratio)
}
/// 3D step limit (`MaxStep3D`); `max_step_3d(&bounds, 0.5)`.
pub fn max_step_3d(bounds: &NewtonBoundsN<3>, max_step_ratio: f64) -> Option<f64> {
    max_step(bounds, max_step_ratio)
}
/// 4D step limit (`MaxStep4D`); `max_step_4d(&bounds, 0.5)`.
pub fn max_step_4d(bounds: &NewtonBoundsN<4>, max_step_ratio: f64) -> Option<f64> {
    max_step(bounds, max_step_ratio)
}
pub(super) fn max_step<const N: usize>(bounds: &NewtonBoundsN<N>, ratio: f64) -> Option<f64> {
    if !bounds.has_bounds {
        return None;
    }
    // Native nesting `max(1, max(d0, max(d1, ...)))` folds from the last side, keeping
    // `std::max`'s NaN propagation order.
    let widest: f64 = (0..N - 1)
        .rev()
        .fold(bounds.max[N - 1] - bounds.min[N - 1], |acc, i| {
            std_max(bounds.max[i] - bounds.min[i], acc)
        });
    Some(ratio * std_max(1.0, widest))
}

/// Clamps into the box, widened by `ratio * side` when soft; `clamp_2d(&mut x, &bounds, false, 0.0)`.
pub fn clamp_2d(
    x: &mut [f64; 2],
    bounds: &NewtonBoundsN<2>,
    use_soft_bounds: bool,
    soft_ratio: f64,
) {
    clamp(x, bounds, use_soft_bounds, soft_ratio);
}
/// 3D clamp (`Clamp3D`); `clamp_3d(&mut x, &bounds, true, 1e-4)`.
pub fn clamp_3d(
    x: &mut [f64; 3],
    bounds: &NewtonBoundsN<3>,
    use_soft_bounds: bool,
    soft_ratio: f64,
) {
    clamp(x, bounds, use_soft_bounds, soft_ratio);
}
/// 4D clamp (`Clamp4D`); `clamp_4d(&mut x, &bounds, true, 1e-4)`.
pub fn clamp_4d(
    x: &mut [f64; 4],
    bounds: &NewtonBoundsN<4>,
    use_soft_bounds: bool,
    soft_ratio: f64,
) {
    clamp(x, bounds, use_soft_bounds, soft_ratio);
}
pub(super) fn clamp<const N: usize>(
    x: &mut [f64; N],
    bounds: &NewtonBoundsN<N>,
    use_soft_bounds: bool,
    soft_ratio: f64,
) {
    if !bounds.has_bounds {
        return;
    }
    for (i, value) in x.iter_mut().enumerate() {
        let extension: f64 = if use_soft_bounds {
            (bounds.max[i] - bounds.min[i]) * soft_ratio
        } else {
            0.0
        };
        *value = std_clamp(*value, bounds.min[i] - extension, bounds.max[i] + extension);
    }
}
/// GCC 13 libstdc++ `std::clamp(v, lo, hi)` is `std::min(std::max(v, lo), hi)`: an inverted
/// box yields `hi` and a NaN value stays NaN (Rust's `f64::clamp` would panic on `lo > hi`).
fn std_clamp(value: f64, lower: f64, upper: f64) -> f64 {
    let raised: f64 = if value < lower { lower } else { value };
    if upper < raised { upper } else { raised }
}
