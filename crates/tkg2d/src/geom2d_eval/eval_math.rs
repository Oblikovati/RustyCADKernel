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
// Rust translation of the local frame and failures of ModelingData/TKG2d/Geom2dEval/*.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The local frame and the failures that every Geom2dEval curve shares; the
//! numeric helpers of the anonymous namespaces (`powInt`, `trigShiftByQuarterTurns`...)
//! live in `eval_basis`, which GeomEval shares.
use crate::geom2d::errors;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis2d, Point2, Vector2, Xy};

/// The `anO`, `aXD` and `aYD = (-aXD.Y(), aXD.X())` that every analytic eval curve
/// reads from its `gp_Ax2d`; `LocalFrame::new(axis).point(1.0, 0.0)`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LocalFrame {
    pub(crate) origin: Xy,
    pub(crate) x_dir: Xy,
    pub(crate) y_dir: Xy,
}
impl LocalFrame {
    /// The frame of `position`; `LocalFrame::new(Axis2d::default())`.
    pub(crate) fn new(position: Axis2d) -> Self {
        let x_dir: Xy = position.direction().xy();
        Self {
            origin: position.location().xy(),
            x_dir,
            y_dir: Xy::new(-x_dir.y(), x_dir.x()),
        }
    }
    /// `gp_Pnt2d(anO + a * aXD + b * aYD)`, summed left to right; `frame.point(r * c, r * s)`.
    pub(crate) fn point(&self, a: f64, b: f64) -> Point2 {
        Point2::from(self.origin + a * self.x_dir + b * self.y_dir)
    }
    /// `gp_Vec2d(a * aXD + b * aYD)`; `frame.vector(a, b)`.
    pub(crate) fn vector(&self, a: f64, b: f64) -> Vector2 {
        Vector2::from(a * self.x_dir + b * self.y_dir)
    }
}

/// Standard_NotImplemented of the members an eval curve refuses (`Reverse`,
/// `ReversedParameter`, `Transform`); `unsupported("Geom2dEval_SineWaveCurve::Reverse", "")`.
pub(crate) fn unsupported(member: &str, detail: &str) -> Failure {
    errors::not_implemented(&format!(
        "{member}{detail}: eval curves keep their defining parameterization; expected no reversal or transformation"
    ))
}

/// Standard_RangeError of `EvalDN` below order 1; `check_derivative_order("X::EvalDN", 0)?`.
pub(crate) fn check_derivative_order(member: &str, n: i32) -> Result<(), Failure> {
    if n < 1 {
        return Err(errors::range(&format!(
            "{member}: N = {n}; expected N >= 1"
        )));
    }
    Ok(())
}
