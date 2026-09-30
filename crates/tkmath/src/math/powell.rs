//! Direct math_Powell.cxx at OCCT 3d097a0, including direction replacement and ignored Value status.
use super::optimization::{Optimization, minimum_results, standard_dump};
use super::{Matrix, MultipleVarFunction, Status, Vector};
use rustycad_tkernel::standard::Failure;
mod extrapolate;
mod iterate;
/// Derivative-free Powell minimization; `Powell::new(&function,1e-8)?`.
pub struct Powell {
    result: Optimization,
    directions: Matrix<'static>,
    state: i32,
    scratch_value: f64,
}
impl Powell {
    /// Uses native iteration and epsilon defaults; `Powell::new(&function,1e-8)?`.
    pub fn new(function: &impl MultipleVarFunction, tolerance: f64) -> Result<Self, Failure> {
        Self::with_options(function, tolerance, 200, 1e-12)
    }
    /// Configures all native stopping options; `Powell::with_options(&function,1e-8,200,1e-12)?`.
    pub fn with_options(
        function: &impl MultipleVarFunction,
        tolerance: f64,
        limit: i32,
        epsilon: f64,
    ) -> Result<Self, Failure> {
        let size: i32 = function.variables();
        let mut result: Optimization =
            Optimization::new(size, [tolerance, epsilon], limit, Status::NotBracketed)?;
        result.minimum = f64::MAX;
        result.previous = f64::MAX;
        Ok(Self {
            result,
            directions: Matrix::with_bounds(1, size, 1, size).unwrap(),
            state: 0,
            scratch_value: 0.0,
        })
    }
    /// Searches along the matrix columns; `solver.perform(&mut function,&start,&directions)?`.
    pub fn perform<F: MultipleVarFunction>(
        &mut self,
        function: &mut F,
        start: &Vector<'_>,
        directions: &Matrix<'_>,
    ) -> Result<(), Failure> {
        self.perform_with_criterion(
            function,
            start,
            directions,
            &mut |solver: &Self, _: &mut F| solver.is_solution_reached(),
        )
    }
    /// Searches with a replacement for the virtual `IsSolutionReached(F)`;
    /// `solver.perform_with_criterion(&mut f,&start,&dirs,&mut |s, _| s.is_solution_reached())?`.
    pub fn perform_with_criterion<F: MultipleVarFunction>(
        &mut self,
        function: &mut F,
        start: &Vector<'_>,
        directions: &Matrix<'_>,
        criterion: &mut impl FnMut(&Self, &mut F) -> bool,
    ) -> Result<(), Failure> {
        self.result.done = false;
        self.scratch_value = 0.0;
        self.result.location.initialized(start)?;
        self.directions.initialized(directions)?;
        let mut previous: Vector<'static> = self.result.location.clone();
        let mut direction: Vector<'static> = Vector::with_bounds(1, previous.upper())?;
        self.result.iterations = 1;
        while self.result.iterations <= self.result.limit {
            if self.iteration(function, [&mut previous, &mut direction], criterion) {
                return Ok(());
            }
            self.result.iterations += 1;
        }
        Ok(())
    }
    /// Applies the native objective convergence criterion; `solver.is_solution_reached()`.
    pub fn is_solution_reached(&self) -> bool {
        2.0 * (self.result.previous - self.result.minimum).abs()
            <= self.result.tolerance
                * (self.result.previous.abs() + self.result.minimum.abs() + self.result.epsilon)
    }
    fn minimize_direction(
        &mut self,
        function: &mut impl MultipleVarFunction,
        direction: &mut Vector<'static>,
    ) -> bool {
        super::optimization::line::minimize(
            function,
            &mut self.result.location,
            direction,
            &mut self.result.minimum,
        )
    }
}
minimum_results!(Powell);
standard_dump!(Powell, "math_Powell resolution:");
