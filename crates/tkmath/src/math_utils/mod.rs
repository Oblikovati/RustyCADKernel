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

//! OCCT modern solver utilities, separate from the legacy math package.
mod config;
mod convergence;
mod core;
mod domain;
mod extended;
mod glibc_cbrt;
mod types;
pub use config::*;
pub use convergence::*;
pub use core::*;
pub use domain::*;
pub use types::*;

mod bracket;
pub use bracket::*;

mod poly;
pub use poly::*;

mod deriv;
pub use deriv::*;

mod line_search;
pub use line_search::*;

mod functor_vector;
pub use functor_vector::*;

pub mod scalar;

pub mod vector;

pub(crate) use line_search::BrentState;

mod gauss;
pub use gauss::{
    get_gauss_points_and_weights, get_kronrod_points_and_weights,
    get_ordered_gauss_points_and_weights,
};

mod random;
pub use random::RandomGenerator;

/// `MathUtils::Utils` helpers (bracketing and finite-difference building blocks).
pub mod utils;
