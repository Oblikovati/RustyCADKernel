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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntImp at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package IntImp: the implicit-function toolkit of the walking and
//! curve-surface intersections.
//!
//! Besides `IntImp_ComputeTangence` and the isoparametric choice, the package is
//! generic code (docs/geomalgo-design.md, "IntImp templates"): `IntImp_ZerParFunc`,
//! `IntImp_ZerImpFunc`, `IntImp_ZerCSParFunc`, `IntImp_Int2S` and `IntImp_IntCS` are
//! ported once as generics over the traits of `tools` (`ThePSurfaceTool`,
//! `TheISurfaceTool`, `TheCurveTool`, `TheFunction`). The packages that instantiate them natively
//! (IntWalk, IntPatch, GeomInt, IntCurveSurface) declare the instances as aliases.
mod captured;
mod compute_tangence;
mod int2s;
mod int_cs;
mod tools;
mod zer_cs_par_func;
mod zer_imp_func;
mod zer_par_func;

pub use compute_tangence::{choix_ref, compute_tangence};
pub use int_cs::IntImpIntCS;
pub use int2s::IntImpInt2S;
pub use tools::{HSurfaceTool, TheCurveTool, TheFunction, TheISurfaceTool, ThePSurfaceTool};
pub use zer_cs_par_func::IntImpZerCSParFunc;
pub use zer_imp_func::IntImpZerImpFunc;
pub use zer_par_func::IntImpZerParFunc;

/// `IntImp_ConstIsoparametric`: the isoparametric line a walking step fixes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum IntImpConstIsoparametric {
    /// U constant on the first patch.
    #[default]
    UIsoparametricOnCaro1,
    /// V constant on the first patch.
    VIsoparametricOnCaro1,
    /// U constant on the second patch.
    UIsoparametricOnCaro2,
    /// V constant on the second patch.
    VIsoparametricOnCaro2,
}
