// Copyright (c) 1992-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/TopTrans at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package TopTrans: the complex transitions at an intersection. A curve
//! transition (`CurveTransition`) accumulates the boundary curves that a curve crosses
//! at one point; a surface transition (`SurfaceTransition`) accumulates the boundary
//! faces that a surface crosses along one curve. Both answer the states just before
//! and just after the intersection.
mod curve_transition;
mod nearest;
mod surface_transition;

pub use curve_transition::CurveTransition;
pub use surface_transition::{SurfaceTransition, get_after, get_before};
