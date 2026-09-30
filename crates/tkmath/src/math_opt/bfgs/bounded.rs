//! Bounded BFGS from MathOpt_BFGS.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::super::bounds::*;
use super::*;
impl BfgsState {
    fn bounded_line(
        &mut self,
        f: &mut impl MultipleVarFunction,
        bounds: &BoxBounds<'_>,
        cfg: &Config,
    ) -> Result<LineSearchResult, Status> {
        self.search_direction();
        bounds.project(&self.point, &mut self.direction, false);
        let mut callback = BoundedFunction::new(f, bounds.lower, bounds.upper);
        let alpha: f64 = bounds.alpha(&self.point, &self.direction);
        let mut line: LineSearchResult = LineSearchResult::default();
        if alpha > 0.0 {
            line = backtrack(
                &mut callback,
                &self.point,
                &self.direction,
                &self.gradient,
                self.fx,
                alpha,
                cfg.step_min,
            );
        }
        if line.is_valid {
            return Ok(line);
        }
        let first: Status = line.status;
        for i in 0..self.point.length() {
            self.direction[i] = -self.gradient[i];
        }
        bounds.project(&self.point, &mut self.direction, false);
        let alpha: f64 = bounds.alpha(&self.point, &self.direction);
        if alpha > 0.0 {
            line = backtrack(
                &mut callback,
                &self.point,
                &self.direction,
                &self.gradient,
                self.fx,
                alpha,
                cfg.step_min,
            );
        }
        if !line.is_valid {
            return Err(failure_after_fallback(first, line.status));
        }
        self.inverse = identity(self.point.length() as usize);
        Ok(line)
    }
    fn bounded_iteration(
        &mut self,
        f: &mut impl MultipleVarFunctionWithGradient,
        bounds: &BoxBounds<'_>,
        cfg: &Config,
    ) -> Result<Option<Status>, Status> {
        let line: LineSearchResult = self.bounded_line(f, bounds, cfg)?;
        self.prepare_next(line.alpha);
        bounds.clamp(&mut self.next);
        for i in 0..self.point.length() {
            self.step[i] = self.next[i] - self.point[i];
        }
        let mut fx: f64 = 0.0;
        let status: Status = value_status(f, &self.next, &mut fx);
        if status != Status::Ok {
            return Err(status);
        }
        let status: Status = gradient_status(f, &self.next, &mut self.next_gradient);
        if status != Status::Ok {
            return Err(status);
        }
        bounds.project(&self.next, &mut self.next_gradient, true);
        if norm(&self.next_gradient) < cfg.f_tolerance {
            self.accept(fx);
            return Ok(Some(Status::Ok));
        }
        if max_difference(&self.next, &self.point) < cfg.x_tolerance {
            self.accept(fx);
            return Ok(Some(Status::NotConverged));
        }
        self.update_inverse();
        self.accept(fx);
        Ok(None)
    }
    fn run_bounded(
        mut self,
        f: &mut impl MultipleVarFunctionWithGradient,
        bounds: &BoxBounds<'_>,
        cfg: &Config,
    ) -> VectorResult {
        bounds.project(&self.point, &mut self.gradient, true);
        if norm(&self.gradient) < cfg.f_tolerance {
            return self.finish(Status::Ok);
        }
        for iteration in 1..=cfg.max_iterations {
            self.iterations = iteration;
            match self.bounded_iteration(f, bounds, cfg) {
                Ok(Some(status)) | Err(status) => return self.finish(status),
                Ok(None) => {}
            }
        }
        self.finish(Status::MaxIterations)
    }
}
/// Preserves native projected gradients and clamped callbacks; `bfgs_bounded(&mut f,&start,&lo,&hi,&config)`.
pub fn bfgs_bounded(
    f: &mut impl MultipleVarFunctionWithGradient,
    start: &Vector<'_>,
    lower: &Vector<'_>,
    upper: &Vector<'_>,
    cfg: &Config,
) -> VectorResult {
    let bounds: BoxBounds<'_> = BoxBounds { lower, upper };
    if !is_valid_config(cfg) || !is_finite_vector(start) || !bounds.valid(start.length()) {
        return VectorResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    let mut point: Vector<'static> = zero_based(start);
    bounds.clamp(&mut point);
    match BfgsState::initialize(f, &point) {
        Ok(state) => state.run_bounded(f, &bounds, cfg),
        Err(status) => VectorResult {
            status,
            ..Default::default()
        },
    }
}
