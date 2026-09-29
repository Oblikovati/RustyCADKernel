// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of the curve-surface classes of Extrema at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The extrema of the distance between a curve and a surface: the function
//! FuncExtCS, the global-optimization functions (GlobOptFuncCS, GlobOptFuncConicS,
//! GlobOptFuncCQuadric), the global search GenExtCS and the general class ExtCS.
mod ext_cs;
mod func_ext_cs;
mod gen_ext_cs;
mod glob_opt_func_c_quadric;
mod glob_opt_func_conic_s;
mod glob_opt_func_cs;

pub use ext_cs::ExtCS;
pub use func_ext_cs::FuncExtCS;
pub use gen_ext_cs::GenExtCS;
pub use glob_opt_func_c_quadric::GlobOptFuncCQuadric;
pub use glob_opt_func_conic_s::GlobOptFuncConicS;
pub use glob_opt_func_cs::GlobOptFuncCS;
