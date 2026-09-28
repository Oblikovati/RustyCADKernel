// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKG2d/Geom2dEval at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The Geom2dEval package: evaluation representation descriptors (which Geom2d
//! includes) and the eval curves, which are defined by a closed-form
//! parameterization and refuse reversal and transformation.
mod rep_curve_desc;
pub use rep_curve_desc::{
    Domain1d, Map1d, RepCurveDesc, RepCurveDescDerivBounded, RepCurveDescFull, RepCurveDescMapped,
    RepCurveKind,
};
pub mod rep_utils;

mod analytic_dump;
mod basis_sums;
mod eval_math;

mod aht_bezier_curve;
pub use aht_bezier_curve::Geom2dEvalAHTBezierCurve;
mod archimedean_spiral_curve;
pub use archimedean_spiral_curve::Geom2dEvalArchimedeanSpiralCurve;
mod circle_involute_curve;
pub use circle_involute_curve::Geom2dEvalCircleInvoluteCurve;
mod logarithmic_spiral_curve;
pub use logarithmic_spiral_curve::Geom2dEvalLogarithmicSpiralCurve;
mod sine_wave_curve;
pub use sine_wave_curve::Geom2dEvalSineWaveCurve;
mod tbezier_curve;
pub use tbezier_curve::Geom2dEvalTBezierCurve;
