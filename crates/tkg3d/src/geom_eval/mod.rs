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
// Rust translation of ModelingData/TKG3d/GeomEval/GeomEval_RepCurveDesc.hxx, GeomEval_RepSurfaceDesc.hxx, GeomEval_RepUtils.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parts of GeomEval that Geom includes: evaluation representation
//! descriptors of curves and surfaces and their evaluation helpers.
//!
//! `GeomEval_RepCurveDesc` is identical to `Geom2dEval_RepCurveDesc` except for
//! the curve class, so both expand `rustycad_tkg2d::define_rep_curve_descriptors`
//! and share `Map1d`, `Domain1d` and the kind enumeration.
mod rep_curve_desc;
pub use rep_curve_desc::{
    RepCurveDesc, RepCurveDescDerivBounded, RepCurveDescFull, RepCurveDescMapped,
};
pub use rustycad_tkg2d::geom2d_eval::{Domain1d, Map1d, RepCurveKind};
mod rep_surface_desc;
pub use rep_surface_desc::{
    Domain2d, Map2d, RepSurfaceDesc, RepSurfaceDescDerivBounded, RepSurfaceDescFull,
    RepSurfaceDescMapped, RepSurfaceKind,
};
mod eval_failures;
pub mod rep_utils;

// GeomEval curves.
mod aht_bezier_curve;
mod bezier_curve_data;
mod curve_support;
pub use aht_bezier_curve::GeomEvalAHTBezierCurve;
mod circular_helix_curve;
pub use circular_helix_curve::GeomEvalCircularHelixCurve;
mod sine_wave_curve;
pub use sine_wave_curve::GeomEvalSineWaveCurve;
mod tbezier_curve;
pub use tbezier_curve::GeomEvalTBezierCurve;

// GeomEval elementary surfaces.

// GeomEval Bezier surfaces.
