// Copyright (c) 1996-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/AppBlend at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package AppBlend: the B-spline approximation of a surface swept by
//! sections (fillets, sweeps, skinned surfaces).
//!
//! `AppBlend_Approx` is the abstract result ([`AppBlendApprox`]).
//! `AppBlend_AppSurf.gxx` is a template over `TheLine` and `TheSectionGenerator`
//! (docs/geomalgo-design.md, "AppBlend template"): since the class stores neither,
//! [`AppBlendAppSurf`] is one type whose `perform` methods are generic over the two
//! traits of `tools`, and each native instance (`GeomFill_AppSurf`,
//! `GeomFill_AppSweep`, `BRepBlend_AppSurf`) is an alias of it.
//! `AppBlend_Debug.cxx` holds the two process-wide switches of `context`.
mod app_surf;
mod approx;
mod context;
mod internal_perform;
mod sections;
mod solution;
mod tools;
mod tronc_perform;

pub use app_surf::AppBlendAppSurf;
pub use approx::{AppBlendApprox, AppBlendCurve2d, AppBlendSurface};
pub use context::{
    get_context_approx_with_no_tgt, get_context_spline_approx, set_context_approx_with_no_tgt,
    set_context_spline_approx,
};
pub use tools::{AppBlendSection, TheLine, TheSectionGenerator};
