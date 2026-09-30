//! MathOpt_Powell.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::utils::*;
use super::*;
struct PowellState {
    point: Vector<'static>,
    old: Vector<'static>,
    direction: Vector<'static>,
    extrapolated: Vector<'static>,
    displacement: Vector<'static>,
    directions: Matrix<'static>,
    fx: f64,
    iterations: u32,
}
impl PowellState {
    fn initialize(
        f: &mut impl MultipleVarFunction,
        start: &Vector<'_>,
        initial: Option<&Matrix<'_>>,
    ) -> Result<Self, Status> {
        let point: Vector<'static> = zero_based(start);
        let mut fx: f64 = 0.0;
        let status: Status = value_status(f, &point, &mut fx);
        if status != Status::Ok {
            return Err(status);
        }
        let n: usize = point.length() as usize;
        let mut directions: Matrix<'static> = identity(n);
        if let Some(initial) = initial {
            directions.initialized(initial).unwrap();
        }
        Ok(Self {
            point,
            fx,
            directions,
            iterations: 0,
            old: Vector::with_size(n),
            direction: Vector::with_size(n),
            extrapolated: Vector::with_size(n),
            displacement: Vector::with_size(n),
        })
    }
    fn finish(self, status: Status) -> VectorResult {
        VectorResult {
            status,
            iterations: self.iterations,
            solution: Some(self.point),
            value: Some(self.fx),
            ..Default::default()
        }
    }
    fn line(&mut self, f: &mut impl MultipleVarFunction, tolerance: f64) -> Result<bool, Status> {
        let result: LineSearchResult =
            exact_line_search(f, &self.point, &self.direction, 10.0, tolerance, 100);
        if !result.is_valid && is_line_search_evaluation_error(&result) {
            return Err(result.status);
        }
        if !result.is_valid
            || !result.alpha.is_finite()
            || !result.f_new.is_finite()
            || result.f_new > self.fx
        {
            return Ok(false);
        }
        for index in 0..self.point.length() {
            self.point[index] += result.alpha * self.direction[index];
        }
        self.fx = result.f_new;
        Ok(true)
    }
    fn sweep(
        &mut self,
        f: &mut impl MultipleVarFunction,
        tolerance: f64,
    ) -> Result<(f64, i32), Status> {
        let (mut decrease, mut biggest): (f64, i32) = (0.0, 0);
        for row in 0..self.point.length() {
            for column in 0..self.point.length() {
                self.direction[column] = self.directions[(row, column)];
            }
            let previous: f64 = self.fx;
            if self.line(f, tolerance)? && previous - self.fx > decrease {
                decrease = previous - self.fx;
                biggest = row + 1;
            }
        }
        Ok((decrease, biggest))
    }
    fn extrapolate(&mut self, f: &mut impl MultipleVarFunction) -> Result<f64, Status> {
        for index in 0..self.point.length() {
            self.extrapolated[index] = 2.0 * self.point[index] - self.old[index];
            self.displacement[index] = self.point[index] - self.old[index];
        }
        let mut value: f64 = 0.0;
        let status: Status = value_status(f, &self.extrapolated, &mut value);
        if status == Status::Ok {
            Ok(value)
        } else {
            Err(status)
        }
    }
    fn replace_direction(&mut self, index: i32) {
        if index == 0 {
            return;
        }
        let last: i32 = self.point.length() - 1;
        for column in 0..self.point.length() {
            self.directions[(index - 1, column)] = self.directions[(last, column)];
            self.directions[(last, column)] = self.displacement[column];
        }
    }
    fn extrapolated_line(
        &mut self,
        f: &mut impl MultipleVarFunction,
        previous: f64,
        decrease: f64,
        biggest: i32,
        tolerance: f64,
    ) -> Result<(), Status> {
        let extrapolated: f64 = self.extrapolate(f)?;
        if extrapolated >= previous {
            return Ok(());
        }
        let diff: f64 = previous - self.fx - decrease;
        let outside: f64 = previous - extrapolated;
        let criterion: f64 = 2.0 * (previous - 2.0 * self.fx + extrapolated) * (diff * diff)
            - decrease * (outside * outside);
        if criterion < 0.0 {
            self.direction.initialized(&self.displacement).unwrap();
            if self.line(f, tolerance)? {
                self.replace_direction(biggest);
            }
        }
        Ok(())
    }
    fn iteration(
        &mut self,
        f: &mut impl MultipleVarFunction,
        config: &Config,
    ) -> Result<Option<Status>, Status> {
        let previous: f64 = self.fx;
        self.old.initialized(&self.point).unwrap();
        let (decrease, biggest): (f64, i32) = self.sweep(f, config.x_tolerance)?;
        if 2.0 * (previous - self.fx).abs()
            <= config.f_tolerance * (previous.abs() + self.fx.abs() + ZERO_TOL)
        {
            return Ok(Some(stationary_status(f, &self.point, self.fx, config)));
        }
        self.extrapolated_line(f, previous, decrease, biggest, config.x_tolerance)?;
        if max_difference(&self.point, &self.old) < config.x_tolerance {
            return Ok(Some(stationary_status(f, &self.point, self.fx, config)));
        }
        Ok(None)
    }
}
fn run(
    f: &mut impl MultipleVarFunction,
    start: &Vector<'_>,
    initial: Option<&Matrix<'_>>,
    config: &Config,
) -> VectorResult {
    if !is_valid_config(config)
        || !is_finite_vector(start)
        || initial.is_some_and(|m| {
            m.row_number() != start.length()
                || m.col_number() != start.length()
                || !is_finite_matrix(m)
        })
    {
        return VectorResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    let mut state: PowellState = match PowellState::initialize(f, start, initial) {
        Ok(state) => state,
        Err(status) => {
            return VectorResult {
                status,
                ..Default::default()
            };
        }
    };
    for iteration in 0..config.max_iterations {
        state.iterations = iteration + 1;
        match state.iteration(f, config) {
            Ok(Some(status)) | Err(status) => return state.finish(status),
            Ok(None) => {}
        }
    }
    state.finish(Status::MaxIterations)
}
/// Minimizes along native coordinate directions; `powell(&mut f,&start,&Config::default())`.
pub fn powell(
    f: &mut impl MultipleVarFunction,
    start: &Vector<'_>,
    config: &Config,
) -> VectorResult {
    run(f, start, None, config)
}
/// Starts with caller-supplied direction rows; `powell_with_directions(&mut f,&start,&directions,&config)`.
pub fn powell_with_directions(
    f: &mut impl MultipleVarFunction,
    start: &Vector<'_>,
    directions: &Matrix<'_>,
    config: &Config,
) -> VectorResult {
    run(f, start, Some(directions), config)
}
