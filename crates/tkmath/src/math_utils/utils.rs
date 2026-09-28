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

//! Public `MathUtils::Utils` helpers of MathUtils_Bracket.hxx and MathUtils_Deriv.hxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. The bracketing and differencing algorithms call
//! these, so the public entry points are the implementation, not copies of it.
use super::{MinBracketOptions, Status};
use crate::math::{Function, Vector};

/// Evaluates `f(x)` and classifies a callback failure or a non-finite value.
///
/// A failed callback sets `CallbackError`; a non-finite value is still written to `value`
/// (the native code evaluates into the caller's variable) and sets `NumericalError`.
///
/// ```
/// use rustycad_tkmath::math_utils::{Status, utils::evaluate_finite};
/// # struct Square; impl rustycad_tkmath::math::Function for Square {
/// #     fn value(&mut self, x: f64, f: &mut f64) -> bool { *f = x * x; true } }
/// let (mut value, mut status) = (0.0, Status::Ok);
/// assert!(evaluate_finite(&mut Square, 3.0, &mut value, &mut status) && value == 9.0);
/// ```
pub fn evaluate_finite(
    f: &mut impl Function,
    x: f64,
    value: &mut f64,
    status: &mut Status,
) -> bool {
    if !f.value(x, value) {
        *status = Status::CallbackError;
        return false;
    }
    if !value.is_finite() {
        *status = Status::NumericalError;
        return false;
    }
    true
}

/// True when a zero endpoint value or differing sign bits bracket a root.
///
/// The native test reads `std::signbit`, so `-0.0` and `+0.0` differ but a zero always counts.
///
/// ```
/// use rustycad_tkmath::math_utils::utils::opposite_signs;
/// assert!(opposite_signs(-1.0, 2.0) && opposite_signs(0.0, 5.0) && !opposite_signs(1.0, 2.0));
/// ```
pub fn opposite_signs(a: f64, b: f64) -> bool {
    a == 0.0 || b == 0.0 || a.is_sign_negative() != b.is_sign_negative()
}

/// Clamps `value` to the option limits when `use_limits` is set.
///
/// `std::max(Left, std::min(Right, value))` returns its first argument on an unordered
/// comparison, so a NaN value maps to the right limit, and a NaN limit is kept as written.
///
/// ```
/// use rustycad_tkmath::math_utils::{MinBracketOptions, utils::limited};
/// let options = MinBracketOptions { use_limits: true, left_limit: 0.0, right_limit: 1.0, ..Default::default() };
/// assert_eq!(limited(3.0, &options), 1.0);
/// ```
pub fn limited(value: f64, options: &MinBracketOptions) -> f64 {
    if !options.use_limits {
        return value;
    }
    let upper: f64 = if value < options.right_limit {
        value
    } else {
        options.right_limit
    };
    if options.left_limit < upper {
        upper
    } else {
        options.left_limit
    }
}

/// True when the iteration budget is positive, the limits are finite and ordered (if used),
/// and every supplied cached endpoint value is finite.
///
/// ```
/// use rustycad_tkmath::math_utils::{MinBracketOptions, utils::is_valid};
/// assert!(is_valid(&MinBracketOptions::default()));
/// ```
pub fn is_valid(options: &MinBracketOptions) -> bool {
    options.max_iterations > 0
        && (!options.use_limits
            || (options.left_limit.is_finite()
                && options.right_limit.is_finite()
                && options.left_limit < options.right_limit))
        && (!options.has_fa || options.fa.is_finite())
        && (!options.has_fb || options.fb.is_finite())
}

/// Writes `x - step` and `x + step`, widened to the adjacent double when the step vanishes.
///
/// Returns false for a non-finite `x`/`step` or a non-positive step (outputs untouched), and
/// when the written points are not finite and strictly around `x` (outputs already written).
///
/// ```
/// use rustycad_tkmath::math_utils::utils::perturbations;
/// let (mut minus, mut plus) = (0.0, 0.0);
/// assert!(perturbations(1.0, 0.5, &mut minus, &mut plus) && (minus, plus) == (0.5, 1.5));
/// ```
pub fn perturbations(x: f64, step: f64, minus: &mut f64, plus: &mut f64) -> bool {
    if !x.is_finite() || !step.is_finite() || step <= 0.0 {
        return false;
    }
    *plus = x + step;
    *minus = x - step;
    // std::nextafter toward +/-infinity when the step is below half an ulp of x.
    if *plus == x {
        *plus = x.next_up();
    }
    if *minus == x {
        *minus = x.next_down();
    }
    minus.is_finite() && plus.is_finite() && *minus < x && x < *plus
}

/// True when every component of `vector` is finite (an empty vector is finite).
///
/// ```
/// use rustycad_tkmath::{math::Vector, math_utils::utils::is_finite_vector};
/// assert!(is_finite_vector(&Vector::with_size(3)));
/// ```
pub fn is_finite_vector(vector: &Vector<'_>) -> bool {
    (0..vector.length() as usize).all(|index| vector.at(index).unwrap().is_finite())
}
