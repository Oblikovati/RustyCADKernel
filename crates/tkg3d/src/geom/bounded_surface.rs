// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKG3d/Geom/Geom_BoundedSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_BoundedSurface`: surfaces limited by finite parameters.
use super::surface::GeomSurface;
use rustycad_tkernel::impl_transient_interface;

/// A surface with finite bounds; the native class adds no member;
/// `surface.cast::<dyn GeomBoundedSurface>()`.
pub trait GeomBoundedSurface: GeomSurface {}
impl_transient_interface!(
    dyn GeomBoundedSurface,
    "Geom_BoundedSurface",
    dyn GeomSurface
);
