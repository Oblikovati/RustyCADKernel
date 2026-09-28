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
// Rust translation of the point-surface classes of Extrema at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The extrema of the distance between a point and a surface: the functions
//! (FuncPSNorm, FuncPSDist), the local search (GenLocateExtPS), the sampled search
//! (GenExtPS), the swept surfaces (ExtPExtS, ExtPRevS) and the general class ExtPS.
mod ext_p_ext_s;
mod ext_p_rev_s;
mod ext_ps;
mod func_ps_dist;
mod func_ps_norm;
mod gen_ext_ps;
mod gen_locate_ext_ps;

pub use ext_p_ext_s::ExtPExtS;
pub use ext_p_rev_s::ExtPRevS;
pub use ext_ps::ExtPS;
pub use func_ps_dist::FuncPSDist;
pub use func_ps_norm::FuncPSNorm;
pub use gen_ext_ps::{GenExtPS, Sampling};
pub use gen_locate_ext_ps::GenLocateExtPS;
