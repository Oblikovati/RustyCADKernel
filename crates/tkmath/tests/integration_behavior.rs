//! Native integration mode differences, callback failures and table validation.
use rustycad_tkmath::math::{
    Function, GaussPointsAndWeights, GaussSingleIntegration as Gauss,
    KronrodSingleIntegration as Kronrod, Vector, kronrod_points_and_weights,
    ordered_gauss_points_and_weights,
};
struct CountedConstant {
    calls: usize,
    fail: usize,
}
impl Function for CountedConstant {
    fn value(&mut self, _x: f64, value: &mut f64) -> bool {
        self.calls += 1;
        *value = 2.0;
        self.calls != self.fail
    }
}
#[test]
fn default_integrators_guard_results() {
    let gauss: Gauss = Gauss::default();
    let kronrod: Kronrod = Kronrod::default();
    assert!(!gauss.is_done());
    assert!(!kronrod.is_done());
    assert_eq!(
        gauss.value().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    assert!(kronrod.value().is_err());
    assert!(kronrod.error_reached().is_err());
    assert!(kronrod.absolute_error().is_err());
    assert!(kronrod.order_reached().is_err());
    assert!(kronrod.iterations_reached().is_err());
}
#[test]
fn gauss_adaptive_recovers_from_failure_in_initial_rule() {
    let mut f: CountedConstant = CountedConstant { calls: 0, fail: 1 };
    let s: Gauss = Gauss::with_tolerance(&mut f, 0.0, 1.0, 3, 1e-8);
    assert!(s.is_done());
    assert!((s.value().unwrap() - 2.0).abs() < 1e-14);
    assert_eq!(f.calls, 7);
}
#[test]
fn gauss_caps_order_and_stops_on_failed_callback() {
    let mut f: CountedConstant = CountedConstant { calls: 0, fail: 0 };
    let s: Gauss = Gauss::new(&mut f, 0.0, 1.0, 100);
    assert!(s.is_done());
    assert_eq!(f.calls, 61);
    let mut f: CountedConstant = CountedConstant { calls: 0, fail: 2 };
    let s: Gauss = Gauss::new(&mut f, 0.0, 1.0, 4);
    assert!(!s.is_done());
    assert_eq!(f.calls, 2);
}
#[test]
fn kronrod_adaptive_accepts_reversed_and_zero_intervals() {
    let mut f: CountedConstant = CountedConstant { calls: 0, fail: 0 };
    let mut s: Kronrod = Kronrod::new(&mut f, 1.0, 0.0, 4);
    assert!(!s.is_done());
    assert_eq!(f.calls, 0);
    s.perform_with_tolerance(&mut f, 1.0, 0.0, 4, 1e-8, 10);
    assert!(s.is_done());
    assert!((s.value().unwrap() + 2.0).abs() < 1e-14);
    assert_eq!(s.order_reached().unwrap(), 5);
    s.perform_with_tolerance(&mut f, 1.0, 1.0, 4, 1e-8, 10);
    assert!(s.is_done());
    assert_eq!(s.value().unwrap(), 0.0);
    assert_eq!(s.iterations_reached().unwrap(), 1);
}
#[test]
fn kronrod_invalid_requests_do_not_evaluate_function() {
    let mut f: CountedConstant = CountedConstant { calls: 0, fail: 0 };
    let mut s: Kronrod = Kronrod::new(&mut f, 0.0, 1.0, 2);
    assert!(!s.is_done());
    s.perform_with_tolerance(&mut f, 0.0, 1.0, 3, 0.0, 10);
    assert!(!s.is_done());
    assert_eq!(f.calls, 0);
}
#[test]
fn quadrature_validates_lengths_and_computed_order() {
    let mut points: Vector<'static> = Vector::with_bounds(1, 3).unwrap();
    let mut weights: Vector<'static> = Vector::with_bounds(1, 3).unwrap();
    assert!(!ordered_gauss_points_and_weights(
        0,
        &mut points,
        &mut weights
    ));
    assert!(!ordered_gauss_points_and_weights(
        2,
        &mut points,
        &mut weights
    ));
    assert!(!kronrod_points_and_weights(2, &mut points, &mut weights));
    assert!(!kronrod_points_and_weights(5, &mut points, &mut weights));
    assert!(ordered_gauss_points_and_weights(
        3,
        &mut points,
        &mut weights
    ));
    assert!(kronrod_points_and_weights(3, &mut points, &mut weights));
    // Order 0 is not done natively (the empty eigenproblem raises inside a try block);
    // a negative order corrupts the native heap and is refused.
    assert!(!GaussPointsAndWeights::new(0).unwrap().is_done());
    assert!(GaussPointsAndWeights::new(-1).is_err());
}
