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

//! Bounded Newton with native projected gradients and callback ordering.
use super::super::bounds::*;
use super::*;
impl NewtonState {
    fn bounded_search(
        &mut self,
        f: &mut impl MultipleVarFunction,
        bounds: &BoxBounds<'_>,
        cfg: &Config,
    ) -> Result<(), Status> {
        let mut callback = BoundedFunction::new(f, bounds.lower, bounds.upper);
        let alpha: f64 = bounds.alpha(&self.x, &self.dir);
        let mut line: LineSearchResult = LineSearchResult::default();
        if alpha > 0.0 {
            line = backtrack(
                &mut callback,
                &self.x,
                &self.dir,
                &self.g,
                self.fx,
                alpha,
                cfg.step_min,
            );
        }
        if !line.is_valid {
            let first: Status = line.status;
            self.steepest();
            let alpha: f64 = bounds.alpha(&self.x, &self.dir);
            if alpha > 0.0 {
                line = backtrack(
                    &mut callback,
                    &self.x,
                    &self.dir,
                    &self.g,
                    self.fx,
                    alpha,
                    cfg.step_min,
                );
            }
            if !line.is_valid {
                return Err(failure_after_fallback(first, line.status));
            }
        }
        self.advance(line.alpha);
        Ok(())
    }
    fn bounded_step(
        &mut self,
        f: &mut impl MultipleVarFunction,
        bounds: &BoxBounds<'_>,
        cfg: &NewtonConfig,
    ) -> Status {
        bounds.project(&self.x, &mut self.dir, false);
        if cfg.use_line_search {
            if let Err(status) = self.bounded_search(f, bounds, &cfg.base) {
                return status;
            }
        } else {
            self.advance(1.0);
        }
        bounds.clamp(&mut self.next);
        let mut fx: f64 = 0.0;
        let status: Status = value_status(f, &self.next, &mut fx);
        if status == Status::Ok {
            self.fx = fx;
        }
        status
    }
    fn run_bounded(
        mut self,
        f: &mut impl HessianFunction,
        bounds: &BoxBounds<'_>,
        cfg: &NewtonConfig,
    ) -> VectorResult {
        bounds.project(&self.x, &mut self.g, true);
        if norm(&self.g) < cfg.base.f_tolerance {
            return self.finish(Status::Ok);
        }
        for iteration in 1..=cfg.base.max_iterations {
            self.iterations = iteration;
            let status: Status = hessian_status(f, &self.x, &mut self.h);
            if status != Status::Ok {
                return self.finish(status);
            }
            if !self.bounded_direction(bounds, cfg.regularization) {
                return VectorResult {
                    status: Status::NumericalError,
                    iterations: iteration,
                    ..Default::default()
                };
            }
            let status: Status = self.bounded_step(f, bounds, cfg);
            if status != Status::Ok {
                return self.finish(status);
            }
            let diff: f64 = max_difference(&self.next, &self.x);
            let status: Status = gradient_status(f, &self.next, &mut self.next_g);
            if status != Status::Ok {
                return VectorResult {
                    status,
                    iterations: iteration,
                    solution: Some(self.next),
                    value: Some(self.fx),
                    ..Default::default()
                };
            }
            bounds.project(&self.next, &mut self.next_g, true);
            self.accept(self.fx);
            if norm(&self.g) < cfg.base.f_tolerance {
                return self.finish(Status::Ok);
            }
            if diff < cfg.base.x_tolerance {
                return self.finish(Status::NotConverged);
            }
        }
        self.finish(Status::MaxIterations)
    }
    fn bounded_direction(&mut self, bounds: &BoxBounds<'_>, minimum: f64) -> bool {
        self.steepest();
        if !regularize_hessian(&mut self.h, minimum) {
            return false;
        }
        let linear: LinearResult = crate::math_lin::solve(&self.h, &self.dir, 1e-15);
        if !linear.is_done() {
            return true;
        }
        self.dir = linear.solution.unwrap();
        bounds.project(&self.x, &mut self.dir, false);
        let mut derivative: f64 = 0.0;
        for i in 0..self.g.length() {
            derivative += self.g[i] * self.dir[i];
        }
        if derivative >= 0.0 {
            self.steepest();
        }
        true
    }
}
/// Constrains every objective callback and projects active gradients; `newton_bounded(&mut f,&start,&lo,&hi,&config)`.
pub fn newton_bounded(
    f: &mut impl HessianFunction,
    start: &Vector<'_>,
    lower: &Vector<'_>,
    upper: &Vector<'_>,
    cfg: &NewtonConfig,
) -> VectorResult {
    let bounds: BoxBounds<'_> = BoxBounds { lower, upper };
    if !cfg.valid(start) || !bounds.valid(start.length()) {
        return VectorResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    let mut point: Vector<'static> = zero_based(start);
    bounds.clamp(&mut point);
    match NewtonState::initialize(f, &point) {
        Ok(state) => state.run_bounded(f, &bounds, cfg),
        Err(status) => VectorResult {
            status,
            ..Default::default()
        },
    }
}
