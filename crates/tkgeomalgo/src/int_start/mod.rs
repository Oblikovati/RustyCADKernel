// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntStart at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package IntStart: the two templates that find the start points of a walking
//! on a surface domain, on its restrictions (`IntStart_SearchOnBoundaries.gxx`) and
//! inside it (`IntStart_SearchInside.gxx`). The package has no header of its own:
//! IntPatch and Contap instantiate the templates. Each template is one Rust generic
//! over the tool traits of [`tools`] (docs/geomalgo-design.md section 12).
pub(crate) mod failures;
mod search_inside;
mod search_on_boundaries;
pub mod tools;

pub use search_inside::SearchInside;
pub use search_on_boundaries::SearchOnBoundaries;
pub use tools::{
    SearchInsideTypes, SearchOnBoundariesTypes, TheArc, TheArcFunction, TheInsideFunction,
    ThePathPoint, TheSOBTool, TheSegment, TheTopolTool, TheVertex,
};
