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
// Rust translation of ModelingData/TKGeomBase at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT TKGeomBase: construction of elementary geometry and the geometry
//! algorithms of ModelingData.
//!
//! TKGeomBase depends on TKernel, TKMath, TKG2d and TKG3d natively; this crate
//! depends on their ports. Each OCCT package is one module, added when it is
//! ported; docs/tkgeombase-progress.md lists the ported packages.
pub mod app_cont;
pub mod app_par_curves;
pub mod bnd_lib;
pub mod cpnts;
mod defl_curv_intervals;
pub mod extrema_pc;
pub mod extrema_pc2d;
pub mod extrema_pc_core;
mod failure_capture;
pub mod fem_tool;
pub mod gc;
pub mod gce;
pub mod gce2d;
pub mod gcpnts;
pub mod geom_bnd_lib;
pub mod geom_lprop;
pub mod geom_tools;
pub mod gprop;
pub mod hermit;
pub mod int_ana;
pub mod int_ana2d;
mod line_extrema;
pub mod lprop;
mod native_failure;
mod native_text;
mod plane_projection;
#[doc(hidden)]
pub mod probe_support;
