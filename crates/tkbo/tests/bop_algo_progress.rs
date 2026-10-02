//! Native BOPAlgo_Algo callback order and progress weight normalization.
use rustycad_tkbo::bop_algo::{AlertUserBreak, Algo, Options, PISteps};
use rustycad_tkernel::message::{AlertHandle, BaseAlert, ProgressRange};
use rustycad_tkernel::standard::Failure;
use std::sync::Arc;

struct WeightedAlgorithm {
    options: Options,
    constants: Vec<(i32, f64)>,
    variable: Vec<(i32, f64)>,
}

impl Algo for WeightedAlgorithm {
    fn options(&self) -> &Options {
        &self.options
    }
    fn perform(&mut self, _: &ProgressRange<'_>) -> Result<(), Failure> {
        Ok(())
    }
    fn fill_pi_constants(&self, _: f64, steps: &mut PISteps) {
        for (index, weight) in &self.constants {
            steps.set_step(*index, *weight);
        }
    }
    fn fill_pi_steps(&self, steps: &mut PISteps) {
        for (index, weight) in &self.variable {
            steps.set_step(*index, *weight);
        }
    }
}

fn normalized(whole: f64, constants: &[(i32, f64)], variable: &[(i32, f64)]) -> Vec<f64> {
    let algorithm = WeightedAlgorithm {
        options: Options::new(),
        constants: constants.to_vec(),
        variable: variable.to_vec(),
    };
    let mut steps = PISteps::new(3).unwrap();
    algorithm.analyze_progress(whole, &mut steps);
    steps.steps().as_slice().to_vec()
}

#[test]
fn progress_bounds_and_out_of_range_access() {
    let mut steps = PISteps::new(3).unwrap();
    assert_eq!((steps.steps().lower(), steps.steps().upper()), (0, 2));
    assert_eq!(steps.steps().as_slice(), &[0.0, 0.0, 0.0]);
    steps.set_step(-1, 9.0);
    steps.set_step(3, 9.0);
    assert_eq!(steps.get_step(-1), 0.0);
    assert_eq!(steps.get_step(3), 0.0);
    steps.change_steps().init(2.0);
    assert_eq!(steps.get_step(1), 2.0);
    assert!(PISteps::new(-1).is_err());
    assert!(PISteps::new(0).unwrap().steps().is_empty());
}

#[test]
fn positive_constant_slots_are_reserved_before_variable_callback() {
    assert_eq!(
        normalized(100.0, &[(0, 20.0)], &[(1, 2.0), (2, 6.0)]),
        [20.0, 20.0, 60.0]
    );
    // The callback can overwrite a reserved slot; its original weight was subtracted.
    assert_eq!(
        normalized(100.0, &[(0, 20.0)], &[(0, 50.0), (1, 1.0)]),
        [50.0, 80.0, 0.0]
    );
    assert_eq!(
        normalized(10.0, &[(0, 20.0)], &[(1, 1.0), (2, 1.0)]),
        [20.0, -5.0, -5.0]
    );
}

#[test]
fn nonpositive_and_nan_sums_leave_variable_steps_unchanged() {
    assert_eq!(
        normalized(100.0, &[], &[(0, -2.0), (1, 2.0)]),
        [-2.0, 2.0, 0.0]
    );
    assert_eq!(normalized(100.0, &[], &[(0, -2.0)]), [-2.0, 0.0, 0.0]);
    let result = normalized(100.0, &[], &[(0, f64::NAN), (1, 2.0)]);
    assert!(result[0].is_nan());
    assert_eq!(result[1], 2.0);
}

#[test]
fn negative_constants_participate_in_variable_normalization() {
    assert_eq!(
        normalized(100.0, &[(0, -2.0)], &[(1, 6.0)]),
        [-50.0, 150.0, 0.0]
    );
}

struct DefaultAlgorithm(Options);
impl Algo for DefaultAlgorithm {
    fn options(&self) -> &Options {
        &self.0
    }
    fn perform(&mut self, _: &ProgressRange<'_>) -> Result<(), Failure> {
        Ok(())
    }
}

#[test]
fn default_validation_clears_errors_and_preserves_warnings() {
    let mut algorithm = DefaultAlgorithm(Options::new());
    let failure: AlertHandle = Arc::new(AlertUserBreak);
    algorithm
        .0
        .add_warning(&(Arc::new(BaseAlert) as AlertHandle));
    algorithm.0.add_error(&failure);
    algorithm.check_data();
    assert!(!algorithm.0.has_errors() && algorithm.0.has_warnings());
    algorithm.0.add_error(&failure);
    algorithm.check_result();
    assert!(!algorithm.0.has_errors() && algorithm.0.has_warnings());
    let mut steps = PISteps::new(3).unwrap();
    algorithm.analyze_progress(100.0, &mut steps);
    assert_eq!(steps.steps().as_slice(), &[0.0, 0.0, 0.0]);
    algorithm.perform(&ProgressRange::default()).unwrap();
}
