// Copyright (c) 2026 OPEN CASCADE SAS
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

//! MathSys native behavior at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::{
    math::{Matrix, Vector},
    math_utils::Status,
};
/// Native generic nonlinear solver result; `SystemResult::default()`.
#[derive(Default)]
pub struct SystemResult {
    /// Completion status; `result.status`.
    pub status: Status,
    /// Completed iteration count; `result.iterations`.
    pub iterations: u32,
    /// Last solution when available; `result.solution`.
    pub solution: Option<Vector<'static>>,
    /// Squared residual norm if representable; `result.value`.
    pub value: Option<f64>,
    /// Final gradient when computed; `result.gradient`.
    pub gradient: Option<Vector<'static>>,
    /// Final Jacobian when computed; `result.jacobian`.
    pub jacobian: Option<Matrix<'static>>,
    /// Final Euclidean residual norm; `result.residual_norm`.
    pub residual_norm: f64,
    /// Last accepted displacement norm; `result.step_norm`.
    pub step_norm: f64,
    /// Equilibrated Jacobian rank; `result.rank`.
    pub rank: usize,
    /// Meets function tolerance; `result.is_root`.
    pub is_root: bool,
    /// Meets least-squares stationarity criterion; `result.is_stationary`.
    pub is_stationary: bool,
}
impl SystemResult {
    /// Reports native success; `result.is_done()`.
    pub fn is_done(&self) -> bool {
        self.status == Status::Ok
    }
}
impl From<&SystemResult> for bool {
    fn from(result: &SystemResult) -> bool {
        result.is_done()
    }
}
