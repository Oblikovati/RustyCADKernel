//! Direct math_FRPR.cxx base algorithm at OCCT 3d097a0, including OCC30492.
use super::optimization::{Optimization, optimization_results, standard_dump};
use super::{MultipleVarFunctionWithGradient, Status, Vector};
use rustycad_tkernel::standard::Failure;
mod iterate;
mod line;
/// Fletcher–Reeves–Polak–Ribiere minimization; `Frpr::new(&function,1e-8)?`.
pub struct Frpr {
    result: Optimization,
    state: i32,
}
impl Frpr {
    /// Uses the function dimension and native iteration defaults; `Frpr::new(&function,1e-8)?`.
    pub fn new(
        function: &impl MultipleVarFunctionWithGradient,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        Self::with_options(function, tolerance, 200, 1e-12)
    }
    /// Configures the native stopping options; `Frpr::with_options(&function,1e-8,200,1e-12)?`.
    pub fn with_options(
        function: &impl MultipleVarFunctionWithGradient,
        tolerance: f64,
        limit: i32,
        epsilon: f64,
    ) -> Result<Self, Failure> {
        Ok(Self {
            result: Optimization::new(
                function.variables(),
                [tolerance, epsilon],
                limit,
                Status::NotBracketed,
            )?,
            state: 0,
        })
    }
    /// Performs Polak–Ribiere updates with native line search; `solver.perform(&mut function,&start)?`.
    pub fn perform<F: MultipleVarFunctionWithGradient>(
        &mut self,
        function: &mut F,
        start: &Vector<'_>,
    ) -> Result<(), Failure> {
        self.perform_with_criterion(function, start, &mut |solver: &Self, _: &mut F| {
            solver.is_solution_reached()
        })
    }
    /// Runs the search with a replacement for the virtual `IsSolutionReached(F)`;
    /// `solver.perform_with_criterion(&mut f,&start,&mut |s, _| s.is_solution_reached())?`.
    pub fn perform_with_criterion<F: MultipleVarFunctionWithGradient>(
        &mut self,
        function: &mut F,
        start: &Vector<'_>,
        criterion: &mut impl FnMut(&Self, &mut F) -> bool,
    ) -> Result<(), Failure> {
        if !self.result.initialize(function, start)? {
            return Ok(());
        }
        let mut negative: Vector<'static> = self.result.gradient.clone();
        for index in 1..=negative.upper() {
            negative[index] = -negative[index];
        }
        let mut previous_direction: Vector<'static> = negative.clone();
        self.result.gradient.initialized(&negative)?;
        self.iterate(
            function,
            [&mut negative, &mut previous_direction],
            criterion,
        );
        Ok(())
    }
}
optimization_results!(Frpr);
standard_dump!(Frpr, "math_FRPR ");
