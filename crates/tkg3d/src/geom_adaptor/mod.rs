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
// Rust translation of ModelingData/TKG3d/GeomAdaptor at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The GeomAdaptor package: Geom curves and surfaces behind the Adaptor3d
//! interfaces, the swept and transformed adaptors, and the way back from an adapted
//! curve or surface to a Geom one.
mod curve;
mod curve_eval;
mod curve_eval_order;
mod curve_query;
mod curve_shape;
mod curve_state;
pub use curve::GeomAdaptorCurve;
mod surface;
mod surface_eval;
mod surface_eval_order;
mod surface_query;
mod surface_shape;
mod surface_span;
mod surface_state;
pub use surface::GeomAdaptorSurface;
pub use surface_state::LoadRange as GeomAdaptorSurfaceRange;
