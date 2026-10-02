// Port of BOPAlgo_ParallelAlgo in BOPAlgo_Algo.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Parallel task state and native overload dispatch.
use super::{Algo, Options, PISteps};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;

/// Base state for parallel Boolean tasks. A task owns one progress range.
/// Example: `let base = ParallelAlgoBase::default(); assert!(!base.progress_range().is_active());`.
#[derive(Default)]
pub struct ParallelAlgoBase<'a> {
    options: Options,
    progress_range: ProgressRange<'a>,
}

impl<'a> ParallelAlgoBase<'a> {
    /// Reads inherited configuration; `base.options().has_errors()`.
    pub fn options(&self) -> &Options {
        &self.options
    }

    /// Changes inherited configuration; `base.options_mut().set_run_parallel(true)`.
    pub fn options_mut(&mut self) -> &mut Options {
        &mut self.options
    }

    /// Borrows the stored range so the task can open its scope once.
    /// Example: `ProgressScope::unnamed(base.progress_range(), 1.0, false)`.
    pub fn progress_range(&self) -> &ProgressRange<'a> {
        &self.progress_range
    }

    /// Transfers progress responsibility by native range-copy semantics.
    /// Example: `base.set_progress_range(&range)` disarms `range`.
    pub fn set_progress_range(&mut self, range: &ProgressRange<'a>) {
        self.progress_range.clone_from(range);
    }
}

/// `BOPAlgo_ParallelAlgo`: the range-free task entry point.
/// Calls through [`Algo::perform`] deliberately do nothing, matching the private native override.
/// Example: `task.set_progress_range(&range); task.perform_parallel()?;`.
pub trait ParallelAlgo<'a> {
    /// Reads inherited options independently of the stored range lifetime.
    /// Example: `task.parallel_options().run_parallel()`.
    fn parallel_options(&self) -> &Options;

    /// Accesses task state; `task.parallel_base().progress_range()`.
    fn parallel_base(&self) -> &ParallelAlgoBase<'a>;

    /// Accesses mutable task state; `task.parallel_base_mut().options_mut()`.
    fn parallel_base_mut(&mut self) -> &mut ParallelAlgoBase<'a>;

    /// Executes the concrete task using its stored range; `task.perform_parallel()?`.
    fn perform_parallel(&mut self) -> Result<(), Failure>;

    /// Stores the range for a single execution; `task.set_progress_range(&range)`.
    fn set_progress_range(&mut self, range: &ProgressRange<'a>) {
        self.parallel_base_mut().set_progress_range(range);
    }

    /// Supplies constant weights for inherited progress analysis; the base does nothing.
    /// Example: `task.fill_parallel_constants(100.0, &mut steps)`.
    fn fill_parallel_constants(&self, _whole: f64, _steps: &mut PISteps) {}

    /// Supplies variable weights for inherited progress analysis; the base does nothing.
    /// Example: `task.fill_parallel_steps(&mut steps)`.
    fn fill_parallel_steps(&self, _steps: &mut PISteps) {}
}

impl<'a, T: ParallelAlgo<'a>> Algo for T {
    fn options(&self) -> &Options {
        self.parallel_options()
    }

    fn perform(&mut self, _range: &ProgressRange<'_>) -> Result<(), Failure> {
        Ok(())
    }

    fn fill_pi_constants(&self, whole: f64, steps: &mut PISteps) {
        self.fill_parallel_constants(whole, steps);
    }

    fn fill_pi_steps(&self, steps: &mut PISteps) {
        self.fill_parallel_steps(steps);
    }
}
