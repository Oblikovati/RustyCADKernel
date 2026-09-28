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
// Rust translation of ModelingData/TKG2d/Adaptor2d at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The Adaptor2d package: the 2D curve interface of geometric algorithms, with a
//! line and an offset-curve implementation.
//!
//! `Adaptor2d_Curve2d` is a trait reached through `Handle<dyn Adaptor2dCurve2d>`, as
//! the Geom2d classes are (docs/geom-design.md); `Adaptor2dCurve2dBase` is an
//! instance of the non-abstract native base class.
mod curve2d;
pub use curve2d::{Adaptor2dCurve2d, Adaptor2dCurve2dBase, Adaptor2dD3};
mod line2d;
pub use line2d::Adaptor2dLine2d;
mod offset_curve;
mod offset_curve_kind;
mod offset_curve_shape;
pub use offset_curve::Adaptor2dOffsetCurve;
