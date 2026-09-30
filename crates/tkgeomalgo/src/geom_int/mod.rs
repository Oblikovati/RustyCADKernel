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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomInt at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package GeomInt: the intersection of two Geom surfaces as Geom curves
//! (docs/geomalgo-design.md section 20). `GeomInt_IntSS` runs `IntPatch_Intersection`,
//! splits its lines with `GeomInt_LineConstructor` and makes the curves, approximating
//! the walking lines with `GeomInt_WLApprox` (the ApproxInt instances of `instances`).
mod instances;

pub use instances::*;
