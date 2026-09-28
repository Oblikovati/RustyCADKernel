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

//! Coordinate and diagonal stationarity probes from MathOpt_Utils.hxx at the pinned revision.
use super::utils::{value_status, zero_based};
use super::*;
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
        Ok(value >= self.threshold)
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
pub(super) fn stationary_status(
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
