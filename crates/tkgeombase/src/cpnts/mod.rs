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
// Rust translation of ModelingData/TKGeomBase/CPnts at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! CPnts: the curve length, the point at a curvilinear abscissa and the uniform
//! deflection iterator on an `Adaptor3d_Curve` or an `Adaptor2d_Curve2d`.
//!
//! Each native member has a 3D and a 2D overload with the same body; the port has one
//! member taking a [`PntsCurve`] (convertible from either adaptor). Native exceptions,
//! including the ones the adaptors raise while they are evaluated, are `Err` values.
mod abscissa_point;
mod curve;
mod my_gauss_function;
mod my_root_function;
mod uniform_deflection;

pub use abscissa_point::AbscissaPoint;
pub use curve::PntsCurve;
pub use my_gauss_function::{MyGaussFunction, RealFunction};
pub use my_root_function::MyRootFunction;
pub use uniform_deflection::UniformDeflection;
