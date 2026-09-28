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
// Rust translation of ModelingData/TKG2d/Geom2dEval/Geom2dEval_RepCurveDesc.hxx, Geom2dEval_RepUtils.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parts of Geom2dEval that Geom2d includes: evaluation representation
//! descriptors and their evaluation helpers.
mod rep_curve_desc;
pub use rep_curve_desc::{
    Domain1d, Map1d, MappedCurveState, RepCurveDesc, RepCurveDescDerivBounded, RepCurveDescFull,
    RepCurveDescMapped, RepCurveKind,
};
pub mod rep_utils;
