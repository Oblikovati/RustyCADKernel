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

//! Translation of MathUtils_Bracket.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::utils::{evaluate_finite, is_valid, limited};
use super::{GOLDEN_RATIO, Status};
use crate::math::Function;
mod minimum;
mod options;
mod root;
pub use minimum::{MinBracketResult, bracket_minimum, bracket_minimum_iterations};
pub use options::MinBracketOptions;
pub use root::{BracketResult, bracket_root};

fn expansion(point: f64, previous: f64) -> f64 {
    point + GOLDEN_RATIO * (point - previous)
}
