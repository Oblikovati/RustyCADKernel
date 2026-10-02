// Port of BOPAlgo_Algo.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Abstract Boolean algorithm callbacks and progress normalization.
use super::Options;
use rustycad_tkernel::message::{Gravity, ProgressRange};
use rustycad_tkernel::ncollection::{Array1, ArrayError};
use rustycad_tkernel::standard::Failure;

/// Native operation weights, with zero-based array bounds.
/// Example: `let steps = PISteps::new(3)?; assert_eq!(steps.get_step(-1), 0.0);`.
#[derive(Debug)]
pub struct PISteps {
    steps: Array1<'static, f64>,
}

impl PISteps {
    /// Allocates zero weights; `let steps = PISteps::new(2)?;`.
    pub fn new(operation_count: i32) -> Result<Self, ArrayError> {
        let upper = operation_count.saturating_sub(1);
        Ok(Self {
            steps: Array1::with_bounds(0, upper)?,
        })
    }

    /// Returns the native bounded array; `steps.steps().lower() == 0`.
    pub fn steps(&self) -> &Array1<'static, f64> {
        &self.steps
    }

    /// Allows changing weights and array bounds; `steps.change_steps().init(1.0)`.
    pub fn change_steps(&mut self) -> &mut Array1<'static, f64> {
        &mut self.steps
    }

    /// Ignores an out-of-range operation, as natively; `steps.set_step(-1, 5.0)`.
    pub fn set_step(&mut self, operation: i32, weight: f64) {
        if let Ok(target) = self.steps.change_value(operation) {
            *target = weight;
        }
    }

    /// Returns zero outside the stored bounds; `steps.get_step(-1) == 0.0`.
    pub fn get_step(&self, operation: i32) -> f64 {
        self.steps.value(operation).copied().unwrap_or(0.0)
    }
}

/// `BOPAlgo_Algo` virtual interface; implementations own their [`Options`].
/// Example: `algorithm.check_data(); algorithm.perform(&ProgressRange::default())?;`.
pub trait Algo {
    /// Accesses shared base state; `algorithm.options().has_errors()`.
    fn options(&self) -> &Options;

    /// Supplies the concrete Boolean operation; `algorithm.perform(&range)?`.
    fn perform(&mut self, range: &ProgressRange<'_>) -> Result<(), Failure>;

    /// Base input validation clears failures only; `algorithm.check_data()`.
    fn check_data(&self) {
        self.options().report().clear_gravity(Gravity::Fail);
    }

    /// Base result validation clears failures only; `algorithm.check_result()`.
    fn check_result(&self) {
        self.options().report().clear_gravity(Gravity::Fail);
    }

    /// Reserves constant operation weights; the base does nothing.
    /// Example: `algorithm.fill_pi_constants(100.0, &mut steps)`.
    fn fill_pi_constants(&self, _whole: f64, _steps: &mut PISteps) {}

    /// Supplies input-dependent weights; the base does nothing.
    /// Example: `algorithm.fill_pi_steps(&mut steps)`.
    fn fill_pi_steps(&self, _steps: &mut PISteps) {}

    /// Normalizes variable weights after preserving positive constant slots.
    /// Example: `algorithm.analyze_progress(100.0, &mut steps)`.
    fn analyze_progress(&self, whole: f64, steps: &mut PISteps) {
        self.fill_pi_constants(whole, steps);
        let (remaining, constants) = constant_progress_slots(whole, steps);
        self.fill_pi_steps(steps);
        normalize_variable_progress(remaining, &constants, steps);
    }
}

fn constant_progress_slots(whole: f64, steps: &PISteps) -> (f64, Vec<i32>) {
    let mut remaining = whole;
    let mut constants = Vec::new();
    for index in steps.steps().lower()..=steps.steps().upper() {
        let weight = steps.get_step(index);
        if weight > 0.0 {
            constants.push(index);
            remaining -= weight;
        }
    }
    (remaining, constants)
}

fn normalize_variable_progress(whole: f64, constants: &[i32], steps: &mut PISteps) {
    let indices: Vec<i32> = (steps.steps().lower()..=steps.steps().upper())
        .filter(|index| !constants.contains(index))
        .collect();
    let sum: f64 = indices.iter().map(|index| steps.get_step(*index)).sum();
    if sum <= 0.0 || sum.is_nan() {
        return;
    }
    for index in indices {
        steps.set_step(index, whole * steps.get_step(index) / sum);
    }
}
