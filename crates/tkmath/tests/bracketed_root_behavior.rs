//! Native callback status handling and result guards.
use rustycad_tkmath::math::{BracketedRoot, Function};
#[test]
fn false_callback_status_preserves_written_value() {
    let mut function = |x: f64, result: &mut f64| {
        *result = 2.0 * x - 4.0;
        false
    };
    assert_eq!(function.state_number(), 0);
    let solver: BracketedRoot = BracketedRoot::new(&mut function, 1.0, 3.0, 1e-10);
    assert!(solver.is_done());
    assert_eq!(solver.root().unwrap(), 2.0);
    assert_eq!(solver.value().unwrap(), 0.0);
}
#[test]
fn unfinished_search_guards_all_results() {
    let mut function = |x: f64, result: &mut f64| {
        *result = x * x + 1.0;
        true
    };
    let solver: BracketedRoot = BracketedRoot::new(&mut function, -1.0, 1.0, 1e-10);
    assert!(!solver.is_done());
    assert_eq!(
        solver.root().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(
        solver.value().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(
        solver.iterations().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
}
