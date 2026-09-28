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

//! Modern scalar root solvers retaining OCCT status and optional result fields.
mod newton;
pub use newton::{newton, newton_bounded};

mod brent;
pub use brent::brent;

mod secant;
pub use secant::{secant, secant_auto};

mod bisection;
mod bracket;
pub use bisection::{bisection, bisection_newton};

mod trig;
pub mod utils;
pub use trig::{TrigResult, trigonometric, trigonometric_cde, trigonometric_linear};

mod multiple;
pub use multiple::{
    MultipleConfig, MultipleResult, find_all_roots, find_all_roots_samples,
    find_all_roots_with_derivative,
};
mod all;
pub use all::{
    AllRootsResult, NullInterval, find_all_roots_at_samples, find_all_roots_with_intervals,
};
