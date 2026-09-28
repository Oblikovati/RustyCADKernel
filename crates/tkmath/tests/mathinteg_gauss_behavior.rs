//! Supplementary tests for native Gauss failure fields and node generation.
use rustycad_tkmath::{
    math::Vector,
    math_integ::*,
    math_utils::{IntegConfig, Status, get_gauss_points_and_weights},
};
#[test]
fn zero_width_skips_callbacks_and_adaptive_has_absolute_error() {
    let mut f = |_: f64, _: &mut f64| -> bool { panic!("unexpected zero-width callback") };
    let result = gauss_adaptive(&mut f, 2.0, 2.0, &IntegConfig::default());
    assert_eq!(result.value, Some(0.0));
    assert_eq!(result.absolute_error, Some(0.0));
    assert_eq!(result.relative_error, None);
    assert_eq!(result.number_of_points, 0);
    assert_eq!(gauss(&mut f, 2.0, 2.0, 0).status, Status::InvalidInput);
    assert_eq!(gauss_composite(&mut f, 2.0, 2.0, 3, 7).value, Some(0.0));
}
#[test]
fn adaptive_budget_retains_estimates() {
    let mut f = |x: f64, value: &mut f64| {
        *value = x.exp();
        true
    };
    let config = IntegConfig {
        initial_order: 1,
        max_order: 2,
        max_iterations: 1,
        tolerance: 1e-18,
    };
    let result = gauss_adaptive(&mut f, 0.0, 1.0, &config);
    assert_eq!(result.status, Status::MaxIterations);
    assert!(result.value.is_some());
    assert!(result.absolute_error.unwrap() > 0.0);
    assert!(result.relative_error.unwrap() > 0.0);
    assert_eq!(result.number_of_points, 3);
    assert_eq!(result.iterations, 1);
}
#[test]
fn adaptive_child_failure_preserves_parent_estimate() {
    let mut calls: usize = 0;
    let mut f = |x: f64, value: &mut f64| {
        calls += 1;
        *value = x.exp();
        calls != 4
    };
    let config = IntegConfig {
        initial_order: 1,
        max_order: 2,
        max_iterations: 10,
        tolerance: 1e-18,
    };
    let result = gauss_adaptive(&mut f, 0.0, 1.0, &config);
    assert_eq!(result.status, Status::CallbackError);
    assert!(result.value.is_some());
    assert_eq!(result.number_of_points, 4);
    assert_eq!(result.iterations, 1);
}
#[test]
fn composite_failure_keeps_completed_subintervals() {
    let mut calls: usize = 0;
    let mut f = |_: f64, value: &mut f64| {
        calls += 1;
        *value = 1.0;
        calls != 5
    };
    let result = gauss_composite(&mut f, 0.0, 2.0, 2, 3);
    assert_eq!(result.status, Status::CallbackError);
    assert_eq!(result.value, Some(1.0));
    assert_eq!(result.number_of_points, 5);
    assert_eq!(result.iterations, 2);
}
#[test]
fn eigen_generated_nodes_respect_arbitrary_vector_bounds() {
    let mut points = Vector::with_bounds(-5, 59).unwrap();
    let mut weights = Vector::with_bounds(7, 71).unwrap();
    assert!(get_gauss_points_and_weights(65, &mut points, &mut weights));
    let sum: f64 = (0..65).map(|i| *weights.at(i).unwrap()).sum();
    assert!((sum - 2.0).abs() < 1e-13);
    for i in 1..65 {
        assert!(points.at(i).unwrap() > points.at(i - 1).unwrap());
    }
    assert!(!get_gauss_points_and_weights(64, &mut points, &mut weights));
}
