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
// Rust translation of the curve-curve classes of ModelingData/TKGeomBase/Extrema
// (Extrema_GFuncExtCC, Extrema_GGenExtCC, Extrema_GenLocateExtCC, the
// Extrema_GlobOptFuncCC functions, Extrema_ExtCC, Extrema_ExtCC2d,
// Extrema_LocateExtCC and the Extrema_ECC, Extrema_ECC2d, Extrema_LocECC and
// Extrema_CCLocFOfLocECC aliases) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The curve-curve extrema.
mod ext_cc;
mod ext_cc2d;
mod func_ext_cc;
mod func_ext_cc_values;
mod gen_ext_cc;
mod gen_locate_ext_cc;
mod glob_opt_func_cc;

pub use ext_cc::ExtCC;
pub use ext_cc2d::ExtCC2d;
pub use func_ext_cc::GFuncExtCC;
pub use gen_ext_cc::GGenExtCC;
pub use gen_locate_ext_cc::{GenLocateExtCC, LocECC, LocateExtCC};
pub use glob_opt_func_cc::{GlobOptFuncCCC0, GlobOptFuncCCC1, GlobOptFuncCCC2};

use super::curve_tools::{Curve2dTool, CurveTool};

/// `Extrema_ECC`: `Extrema_GGenExtCC` over Adaptor3d_Curve.
pub type ECC<'a> = GGenExtCC<'a, CurveTool>;
/// `Extrema_ECC2d`: `Extrema_GGenExtCC` over Adaptor2d_Curve2d.
pub type ECC2d<'a> = GGenExtCC<'a, Curve2dTool>;
/// `Extrema_CCLocFOfLocECC`: `Extrema_GFuncExtCC` over Adaptor3d_Curve.
pub type CCLocFOfLocECC<'a> = GFuncExtCC<'a, CurveTool>;
