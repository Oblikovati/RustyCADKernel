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

//! The public `MathSys::Utils` namespace at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//!
//! Upstream declares these helpers in the solver headers (LinearAlgebra, Newton,
//! LevenbergMarquardt, NewtonTypes, Newton2D/3D/4D); each is implemented next to the solver
//! that uses it and re-exported here under one path, like `math_poly::utils`.
//! `MathSys::Utils::IsFinite` is the same all-entries `std::isfinite` scan as MathLin's, so the
//! MathLin implementation is re-exported rather than duplicated.
pub use super::linear::{
    LinearStep, RANK_TOLERANCE, safe_norm, scaled_rank, solve_linearized, solve_qr,
};
pub use super::lm::{LM_DAMPING_MAX, finish_lm, lm_impl};
pub use super::newton::{finalize_system_result, newton_impl};
pub use super::small::{
    ARMIJO_C1, CRITICAL_GRAD, CRITICAL_GRAD_SQ, LINE_SEARCH_MAX, NEWTON_PIVOT_TOL, clamp_2d,
    clamp_3d, clamp_4d, is_armijo_accepted, is_bounds_valid_2d, is_bounds_valid_3d,
    is_bounds_valid_4d, is_finite_array, is_finite_system_n, is_options_valid, is_options_valid_3d,
    is_options_valid_4d, is_step_within_tolerance, max_step_2d, max_step_3d, max_step_4d,
    merit_directional_derivative, safe_distance_n, safe_norm_n, solve_2x2, solve_3x3, solve_4x4,
    solve_symmetric_2x2_svd,
};
pub use crate::math_lin::utils::{is_finite_matrix, is_finite_vector};
