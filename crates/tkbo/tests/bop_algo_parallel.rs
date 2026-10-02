//! Native parallel overload dispatch and progress-range ownership.
use rustycad_tkbo::bop_algo::{Algo, PISteps, ParallelAlgo, ParallelAlgoBase};
use rustycad_tkernel::message::{
    ProgressIndicator, ProgressIndicatorHooks, ProgressRange, ProgressScope,
};
use rustycad_tkernel::standard::Failure;

#[derive(Default)]
struct ParallelTask<'a> {
    base: ParallelAlgoBase<'a>,
    runs: usize,
}
impl<'a> ParallelAlgo<'a> for ParallelTask<'a> {
    fn parallel_options(&self) -> &rustycad_tkbo::bop_algo::Options {
        self.base.options()
    }
    fn parallel_base(&self) -> &ParallelAlgoBase<'a> {
        &self.base
    }
    fn parallel_base_mut(&mut self) -> &mut ParallelAlgoBase<'a> {
        &mut self.base
    }
    fn perform_parallel(&mut self) -> Result<(), Failure> {
        self.runs += 1;
        let _scope = ProgressScope::unnamed(self.base.progress_range(), 1.0, false);
        Ok(())
    }
}

struct QuietProgress;
impl ProgressIndicatorHooks for QuietProgress {
    fn show(&self, _: &ProgressScope<'_>, _: f64, _: bool) {}
}

#[test]
fn base_overload_does_not_execute_parallel_task() {
    let mut task = ParallelTask::default();
    assert!(!task.base.progress_range().is_active());
    task.base.options_mut().set_run_parallel(true);
    assert!(Algo::options(&task).run_parallel());
    Algo::perform(&mut task, &ProgressRange::default()).unwrap();
    assert_eq!(task.runs, 0);
    task.perform_parallel().unwrap();
    assert_eq!(task.runs, 1);
    let mut steps = PISteps::new(2).unwrap();
    task.analyze_progress(100.0, &mut steps);
    assert_eq!(steps.steps().as_slice(), &[0.0, 0.0]);
}

#[test]
fn assigning_a_range_disarms_source_and_task_completes_it_once() {
    let indicator = ProgressIndicator::new(Box::new(QuietProgress));
    let range = indicator.start();
    let mut task = ParallelTask::default();
    task.set_progress_range(&range);
    assert!(!range.is_active());
    assert!(task.base.progress_range().is_active());
    Algo::perform(&mut task, &ProgressRange::default()).unwrap();
    assert_eq!(indicator.get_position(), 0.0);
    task.perform_parallel().unwrap();
    assert_eq!(indicator.get_position(), 1.0);
    assert!(!task.base.progress_range().is_active());
    drop(task);
    assert_eq!(indicator.get_position(), 1.0);
}

#[test]
fn range_assignment_does_not_close_the_old_range() {
    let indicator = ProgressIndicator::new(Box::new(QuietProgress));
    let range = indicator.start();
    let scope = ProgressScope::unnamed(&range, 4.0, false);
    let first = scope.next(1.0);
    let second = scope.next(1.0);
    let mut task = ParallelTask::default();
    task.set_progress_range(&first);
    task.set_progress_range(&second);
    assert!(!first.is_active() && !second.is_active());
    assert_eq!(indicator.get_position(), 0.0);
    task.perform_parallel().unwrap();
    assert_eq!(indicator.get_position(), 0.25);
}
