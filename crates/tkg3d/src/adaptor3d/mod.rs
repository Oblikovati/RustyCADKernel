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
// Rust translation of ModelingData/TKG3d/Adaptor3d at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The Adaptor3d package: the curve and surface interfaces of geometric
//! algorithms, with the curve-on-surface and iso-curve adaptors and the tools that
//! work on adapted surfaces.
//!
//! `Adaptor3d_Curve` and `Adaptor3d_Surface` are traits reached through
//! `Handle<dyn Adaptor3dCurve>` and `Handle<dyn Adaptor3dSurface>`, as the Adaptor2d
//! and Geom classes are (docs/geom-design.md); `Adaptor3dCurveBase` and
//! `Adaptor3dSurfaceBase` are instances of the non-abstract native base classes.
mod curve;
pub use curve::{Adaptor3dCurve, Adaptor3dCurveBase, Adaptor3dD3};
mod surface;
pub use surface::{Adaptor3dSurface, Adaptor3dSurfaceBase};
mod iso_curve;
pub use iso_curve::Adaptor3dIsoCurve;
pub mod h_surface_tool;
mod inter_func;
pub use inter_func::Adaptor3dInterFunc;
mod curve_on_surface;
pub use curve_on_surface::Adaptor3dCurveOnSurface;
