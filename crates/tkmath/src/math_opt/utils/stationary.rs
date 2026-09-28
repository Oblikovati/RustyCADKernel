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

//! Coordinate and diagonal stationarity probes, `MathOpt::Utils::IsStationary`.
use super::{value_status, zero_based};
use crate::math::{MultipleVarFunction, Vector};
use crate::math_utils::{Config, Status};
struct Probe {
    point: Vector<'static>,
    steps: Vec<f64>,
    threshold: f64,
}
impl Probe {
    fn evaluate(&self, f: &mut impl MultipleVarFunction) -> Result<bool, Status> {
        let mut value: f64 = 0.0;
        let status: Status = value_status(f, &self.point, &mut value);
        if status != Status::Ok {
            return Err(status);
        }
        // Native rejects only on `value < fx - tol`; a NaN threshold therefore rejects nothing.
        Ok(value.partial_cmp(&self.threshold) != Some(std::cmp::Ordering::Less))
    }
    fn coordinate(
        &mut self,
        f: &mut impl MultipleVarFunction,
        x: &Vector<'_>,
        index: usize,
    ) -> Result<bool, Status> {
        for sign in [-1.0, 1.0] {
            *self.point.change_at(index).unwrap() = x.at(index).unwrap() + sign * self.steps[index];
            if !self.evaluate(f)? {
                return Ok(false);
            }
        }
        *self.point.change_at(index).unwrap() = *x.at(index).unwrap();
        Ok(true)
    }
    fn diagonal(
        &mut self,
        f: &mut impl MultipleVarFunction,
        x: &Vector<'_>,
        first: usize,
        second: usize,
    ) -> Result<bool, Status> {
        for (a, b) in [(-1.0, -1.0), (-1.0, 1.0), (1.0, -1.0), (1.0, 1.0)] {
            *self.point.change_at(first).unwrap() = x.at(first).unwrap() + a * self.steps[first];
            *self.point.change_at(second).unwrap() = x.at(second).unwrap() + b * self.steps[second];
            if !self.evaluate(f)? {
                return Ok(false);
            }
        }
        *self.point.change_at(first).unwrap() = *x.at(first).unwrap();
        *self.point.change_at(second).unwrap() = *x.at(second).unwrap();
        Ok(true)
    }
    fn run(
        &mut self,
        f: &mut impl MultipleVarFunction,
        x: &Vector<'_>,
        config: &Config,
    ) -> Result<bool, Status> {
        for index in 0..self.steps.len() {
            let step: f64 = 2.0 * config.f_tolerance.sqrt() * (1.0 + x.at(index).unwrap().abs());
            self.steps[index] = if config.x_tolerance < step {
                step
            } else {
                config.x_tolerance
            };
            if !self.coordinate(f, x, index)? {
                return Ok(false);
            }
        }
        for first in 0..self.steps.len() {
            for second in first + 1..self.steps.len() {
                if !self.diagonal(f, x, first, second)? {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
}
/// Powell's stationarity verdict: `Ok` when no probe decreases the function, `NotConverged`
/// when one does, else the failing evaluation status.
pub(crate) fn stationary_status(
    f: &mut impl MultipleVarFunction,
    x: &Vector<'_>,
    fx: f64,
    config: &Config,
) -> Status {
    let mut probe: Probe = Probe {
        point: zero_based(x),
        steps: vec![0.0; x.length() as usize],
        threshold: fx - config.f_tolerance * (1.0 + fx.abs()),
    };
    match probe.run(f, x, config) {
        Ok(true) => Status::Ok,
        Ok(false) => Status::NotConverged,
        Err(status) => status,
    }
}

/// True when no coordinate or pairwise-diagonal perturbation decreases `f` below
/// `fx - f_tolerance * (1 + |fx|)`; `evaluation_status` receives the last evaluation's status
/// and stays untouched for an empty point; `MathOpt::Utils::IsStationary`.
///
/// ```
/// use rustycad_tkmath::{math::Vector, math_opt::utils::is_stationary};
/// use rustycad_tkmath::math_utils::{vector::Sphere, Config, Status};
/// let mut status = Status::NotConverged;
/// let x = Vector::with_size(2);
/// assert!(is_stationary(&mut Sphere::new(2), &x, 0.0, &Config::default(), &mut status));
/// assert_eq!(status, Status::Ok);
/// ```
pub fn is_stationary(
    f: &mut impl MultipleVarFunction,
    x: &Vector<'_>,
    fx: f64,
    config: &Config,
    evaluation_status: &mut Status,
) -> bool {
    match stationary_status(f, x, fx, config) {
        Status::Ok => {
            if x.length() > 0 {
                *evaluation_status = Status::Ok;
            }
            true
        }
        Status::NotConverged => {
            *evaluation_status = Status::Ok;
            false
        }
        failure => {
            *evaluation_status = failure;
            false
        }
    }
}
