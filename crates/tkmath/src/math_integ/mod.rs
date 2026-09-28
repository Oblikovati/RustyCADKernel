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

//! Numerical integration preserving native estimates and callback accounting.
mod gauss;
pub use gauss::{GAUSS_MAX_ORDER, gauss};
mod composite;
pub use composite::gauss_composite;

mod adaptive;
pub use adaptive::gauss_adaptive;

mod kronrod_rule;
pub use kronrod_rule::{KRONROD_MAX_GAUSS_ORDER, kronrod_rule};
mod kronrod;
pub use kronrod::{
    KRONROD_AUTO_INITIAL_ORDER, KRONROD_AUTO_MAX_ITERATIONS, KRONROD_AUTO_ORDER_STEP,
    KronrodConfig, kronrod, kronrod_auto,
};
mod kronrod_infinite;
pub use kronrod_infinite::{kronrod_infinite, kronrod_semi_infinite};

mod double_exp;
pub use double_exp::{
    DOUBLE_EXP_DEFAULT_MAX_POINTS_PER_LEVEL, DOUBLE_EXP_MAX_TRANSFORM_ARGUMENT, DoubleExpConfig,
    EXP_SINH_MIN_TRANSFORM_ARGUMENT, double_exponential, exp_sinh, sinh_sinh, tanh_sinh,
    tanh_sinh_singular, tanh_sinh_with_singularity,
};

mod multiple;
pub use multiple::{
    MULTIPLE_GAUSS_MAX_ORDER, MultipleConfig, gauss_multiple, gauss_multiple_uniform,
};
mod set;
pub use set::{SET_GAUSS_MAX_ORDER, SetResult, gauss_set, gauss_set_bounds};
