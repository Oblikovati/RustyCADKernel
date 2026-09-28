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

//! Modern optimization algorithms with native status and callback semantics.
use crate::math::{Matrix, MultipleVarFunction, MultipleVarFunctionWithGradient, Vector};
use crate::math_utils::{
    Config, LineSearchResult, Status, VectorResult, ZERO_TOL, armijo_backtrack, exact_line_search,
};
mod bfgs;
mod powell;
mod stationary;
mod utils;
pub use bfgs::{bfgs, bfgs_numerical};
pub use powell::{powell, powell_with_directions};

mod scalar;
pub use scalar::{brent, brent_with_bracket, golden};

mod frpr;
pub use frpr::{ConjugateGradientFormula, FrprConfig, frpr, frpr_numerical};

mod bounds;
mod newton;
pub use bfgs::bfgs_bounded;
pub use newton::{
    HessianFunction, NewtonConfig, newton, newton_bounded, newton_modified, newton_numerical,
    newton_numerical_hessian,
};

mod lbfgs;
pub use lbfgs::lbfgs;

mod pso;
pub use pso::*;

mod global;
pub use global::*;
