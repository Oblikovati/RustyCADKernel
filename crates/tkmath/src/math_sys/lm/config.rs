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

//! MathSys_LevenbergMarquardt.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::math_utils::Config;
/// Native scaled Levenberg–Marquardt settings; `LMConfig::default()`.
#[derive(Clone, Copy, Debug)]
pub struct LMConfig {
    /// Shared solver settings; `config.solver.tolerance`.
    pub solver: Config,
    /// Initial damping; `config.lambda_init`.
    pub lambda_init: f64,
    /// Damping growth factor; `config.lambda_increase`.
    pub lambda_increase: f64,
    /// Damping reduction factor; `config.lambda_decrease`.
    pub lambda_decrease: f64,
    /// Maximum damping; `config.lambda_max`.
    pub lambda_max: f64,
    /// Minimum damping; `config.lambda_min`.
    pub lambda_min: f64,
    /// Rank tolerance after column scaling; `config.rank_tolerance`.
    pub rank_tolerance: f64,
}
impl Default for LMConfig {
    fn default() -> Self {
        Self::new(1e-10, 100)
    }
}
impl LMConfig {
    /// Initializes native damping defaults; `LMConfig::new(1e-8,100)`.
    pub fn new(tolerance: f64, limit: u32) -> Self {
        Self {
            solver: Config::new(tolerance, limit),
            lambda_init: 1e-3,
            lambda_increase: 10.0,
            lambda_decrease: 0.1,
            lambda_max: 1e16,
            lambda_min: 1e-15,
            rank_tolerance: super::super::linear::RANK_TOLERANCE,
        }
    }
    pub(super) fn valid(&self) -> bool {
        let positive: [f64; 9] = [
            self.solver.tolerance,
            self.solver.x_tolerance,
            self.solver.f_tolerance,
            self.solver.step_min,
            self.lambda_init,
            self.lambda_increase,
            self.lambda_decrease,
            self.lambda_min,
            self.rank_tolerance,
        ];
        positive.iter().all(|v| v.is_finite() && *v > 0.0)
            && self.solver.max_iterations > 0
            && self.solver.relative_tolerance.is_finite()
            && self.solver.relative_tolerance >= 0.0
            && self.lambda_increase > 1.0
            && self.lambda_decrease < 1.0
            && self.lambda_min <= self.lambda_init
            && self.lambda_max.is_finite()
            && self.lambda_init <= self.lambda_max
    }
}
