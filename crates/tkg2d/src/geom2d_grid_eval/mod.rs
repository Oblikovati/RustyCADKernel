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
// Rust translation of ModelingData/TKG2d/Geom2dGridEval at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The Geom2dGridEval package: batch evaluation of 2D curves on parameter grids.
//!
//! Each evaluator takes an array of parameters (any bounds) and returns the points or
//! derivatives in an array indexed from 1; empty parameters, a null curve or an
//! order below 1 give an empty array. `Geom2dGridEvalCurve` picks the specialized
//! evaluator from the curve class. Evaluators that can raise natively (through the
//! gp getters, BSplCLib, the offset utilities or an adaptor) return `Result`.
use crate::geom2d::{Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3};

/// Point and first derivative of a grid evaluation (`Geom2dGridEval::CurveD1`); the
/// same fields as `Geom2d_Curve::ResD1`, so the port shares the struct.
pub type Geom2dGridEvalCurveD1 = Geom2dCurveD1;
/// Point and two derivatives of a grid evaluation (`Geom2dGridEval::CurveD2`).
pub type Geom2dGridEvalCurveD2 = Geom2dCurveD2;
/// Point and three derivatives of a grid evaluation (`Geom2dGridEval::CurveD3`).
pub type Geom2dGridEvalCurveD3 = Geom2dCurveD3;

pub mod bezier_arrays;
mod grid;
pub mod grid_loops;

mod line;
pub use line::Geom2dGridEvalLine;
mod circle;
pub use circle::Geom2dGridEvalCircle;
mod ellipse;
pub use ellipse::Geom2dGridEvalEllipse;
mod hyperbola;
pub use hyperbola::Geom2dGridEvalHyperbola;
mod parabola;
pub use parabola::Geom2dGridEvalParabola;
mod bezier_curve;
pub use bezier_curve::Geom2dGridEvalBezierCurve;
mod bspline_curve;
mod bspline_spans;
pub use bspline_curve::Geom2dGridEvalBSplineCurve;
mod offset_curve;
pub use offset_curve::Geom2dGridEvalOffsetCurve;
mod other_curve;
pub use other_curve::Geom2dGridEvalOtherCurve;
mod curve;
pub use curve::{Geom2dGridEvalCurve, Geom2dGridEvalEvaluator};
