//! Reusing a minimizer preserves OCCT's unusual early-failure state.
use rustycad_tkmath::math::BrentMinimum;
#[test]
fn callback_failure_after_success_retains_done_and_replaces_cached_value() {
    let mut function = |x: f64, value: &mut f64| {
        *value = (x - 2.0) * (x - 2.0) + 1.0;
        true
    };
    let mut solver: BrentMinimum = BrentMinimum::new(1e-10);
    solver.perform(&mut function, 0.0, 1.5, 4.0);
    assert!(solver.is_done());
    let previous: i32 = solver.iterations().unwrap();
    let mut failing = |_x: f64, value: &mut f64| {
        *value = 91.0;
        false
    };
    solver.perform(&mut failing, 0.0, 3.0, 4.0);
    assert!(solver.is_done());
    assert_eq!(solver.location().unwrap(), 3.0);
    assert_eq!(solver.minimum().unwrap(), 91.0);
    assert_eq!(solver.iterations().unwrap(), previous);
}
#[test]
fn fresh_minimizer_guards_results() {
    let solver: BrentMinimum = BrentMinimum::new(1e-10);
    assert_eq!(
        solver.location().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(
        solver.minimum().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(
        solver.iterations().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
}
