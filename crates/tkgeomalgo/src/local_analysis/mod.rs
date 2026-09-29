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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/LocalAnalysis at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package LocalAnalysis: the C0/C1/C2/G1/G2 continuity between two curves or two
//! surfaces at a point, and the `LocalAnalysis::Dump` reports.
//!
//! The constructors' tolerance arguments (`EpsNul` to `Maxlen`, with native defaults)
//! are gathered in [`CurveContinuityTolerances`] and [`SurfaceContinuityTolerances`].
mod curve;
mod dump;
mod surface;

pub use curve::{CurveContinuityTolerances, LocalAnalysisCurveContinuity};
pub use dump::{dump_curve, dump_surface};
pub use surface::{LocalAnalysisSurfaceContinuity, SurfaceContinuityTolerances};

use rustycad_tkernel::standard::Failure;
use std::f64::consts::PI;

/// Why an analysis failed (`LocalAnalysis_StatusErrorType`);
/// `LocalAnalysisStatusErrorType::CurvatureNotDefined as i32 == 4`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum LocalAnalysisStatusErrorType {
    /// `LocalAnalysis_NullFirstDerivative`.
    NullFirstDerivative,
    /// `LocalAnalysis_NullSecondDerivative`.
    NullSecondDerivative,
    /// `LocalAnalysis_TangentNotDefined`.
    TangentNotDefined,
    /// `LocalAnalysis_NormalNotDefined`.
    NormalNotDefined,
    /// `LocalAnalysis_CurvatureNotDefined`.
    CurvatureNotDefined,
}

/// An angle folded into `[0, PI/2]`: `PI - ang` above `PI/2`, as every test computes it.
fn folded_angle(ang: f64) -> f64 {
    if ang > PI / 2.0 { PI - ang } else { ang }
}

/// The StdFail_NotDone of the queries of a failed analysis.
fn not_done(class: &str, member: &str) -> Failure {
    Failure::with_message(
        "StdFail_NotDone",
        &format!("{class}::{member}: the analysis failed; expected IsDone"),
    )
}
