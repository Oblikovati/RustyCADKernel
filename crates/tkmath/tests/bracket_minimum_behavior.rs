//! Callback failures retain NotDone and supplied endpoint values avoid evaluations.
use rustycad_tkmath::math::BracketMinimum;
#[test]
fn supplied_values_skip_both_initial_callbacks() {
    let mut calls: Vec<f64> = Vec::new();
    let mut function = |x: f64, value: &mut f64| {
        calls.push(x);
        *value = (x - 2.0) * (x - 2.0) + 1.0;
        true
    };
    let solver: BracketMinimum = BracketMinimum::with_values(&mut function, 0.0, 1.0, 5.0, 2.0);
    assert!(solver.is_done());
    assert_ne!(calls[0], 0.0);
    assert_ne!(calls[0], 1.0);
}
#[test]
fn callback_failure_guards_both_triplets() {
    let mut function = |_x: f64, value: &mut f64| {
        *value = 91.0;
        false
    };
    let solver: BracketMinimum = BracketMinimum::compute(&mut function, 0.0, 1.0);
    assert!(!solver.is_done());
    assert_eq!(
        solver.values().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(
        solver.function_values().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
}
