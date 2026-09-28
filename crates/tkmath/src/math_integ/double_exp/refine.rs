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

//! MathInteg_DoubleExp.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{DoubleExpConfig, transform::Transform};
use crate::math_integ::{adaptive::converged, gauss::finite_value};
use crate::{
    math::Function,
    math_utils::{IntegResult, Status},
};
struct Refinement {
    previous: f64,
    error: Option<f64>,
    points: usize,
    level: u32,
    step: f64,
}
pub(super) fn refine(
    f: &mut impl Function,
    transform: Transform,
    config: &DoubleExpConfig,
) -> IntegResult {
    let mut state: Refinement = Refinement {
        previous: 0.0,
        error: None,
        points: 0,
        level: 0,
        step: config.step_factor,
    };
    for level in 0..config.nb_levels.min(config.integration.max_iterations) {
        state.level = level;
        match state.next_level(f, transform, config) {
            Err(status) => return state.failure(status),
            Ok(true) => return state.estimate(Status::Ok),
            Ok(false) => state.step *= 0.5,
        }
    }
    state.estimate(Status::MaxIterations)
}
impl Refinement {
    fn next_level(
        &mut self,
        f: &mut impl Function,
        transform: Transform,
        config: &DoubleExpConfig,
    ) -> Result<bool, Status> {
        let before: usize = self.points;
        let mut sum: f64 = 0.0;
        for sign in transform.signs() {
            self.direction(f, transform, config, sign, before, &mut sum)?;
        }
        let level_sum: f64 = self.step * sum;
        if !level_sum.is_finite() {
            return Err(Status::NumericalError);
        }
        let new_sum: f64 = if self.level == 0 {
            level_sum
        } else {
            0.5 * self.previous + level_sum
        };
        let mut done: bool = false;
        if self.level > 0 && self.points > before {
            let error: f64 = (new_sum - self.previous).abs();
            self.error = Some(error);
            done = converged(error, new_sum, config.integration.tolerance);
        }
        self.previous = new_sum;
        Ok(done)
    }
    fn direction(
        &mut self,
        f: &mut impl Function,
        transform: Transform,
        config: &DoubleExpConfig,
        sign: f64,
        before: usize,
        sum: &mut f64,
    ) -> Result<(), Status> {
        let mut k: usize = usize::from(self.level > 0 || sign < 0.0);
        let stride: usize = if self.level == 0 { 1 } else { 2 };
        while let Some((x, weight)) = transform.sample(sign * k as f64 * self.step) {
            if self.points - before >= config.max_points_per_level {
                return Err(Status::MaxIterations);
            }
            self.points += 1;
            let value: f64 = finite_value(f, x)?;
            *sum += weight * value;
            if !sum.is_finite() {
                return Err(Status::NumericalError);
            }
            k += stride;
        }
        Ok(())
    }
    fn estimate(&self, status: Status) -> IntegResult {
        IntegResult {
            status,
            value: Some(self.previous),
            absolute_error: self.error,
            relative_error: self
                .error
                .filter(|_| self.previous.abs() > f64::EPSILON)
                .map(|error| error / self.previous.abs()),
            number_of_points: self.points,
            iterations: self.level + 1,
        }
    }
    fn failure(&self, status: Status) -> IntegResult {
        let mut result: IntegResult = IntegResult {
            status,
            number_of_points: self.points,
            iterations: self.level + 1,
            ..Default::default()
        };
        if status == Status::MaxIterations && self.level > 0 {
            result.value = Some(self.previous);
            result.absolute_error = self.error;
        }
        result
    }
}
