//! Native flags and root access remain independent across protected Solve calls.
use rustycad_tkmath::math::DirectPolynomialRoots;
#[test]
fn identity_has_infinite_solutions_and_guard_precedence() {
    let solver: DirectPolynomialRoots = DirectPolynomialRoots::linear(0.0, 0.0);
    assert!(solver.is_done());
    assert!(solver.infinite_roots());
    assert_eq!(
        solver.solutions().unwrap_err().exception_type(),
        "StdFail_InfiniteSolutions"
    );
    assert_eq!(
        solver.value(0).unwrap_err().exception_type(),
        "StdFail_InfiniteSolutions"
    );
}
#[test]
fn reused_infinite_flag_is_not_cleared_by_solve() {
    let mut solver: DirectPolynomialRoots = DirectPolynomialRoots::linear(0.0, 0.0);
    solver.solve_linear(1.0, -2.0);
    solver.solve_quadratic(1.0, -5.0, 6.0);
    solver.solve_cubic(1.0, -6.0, 11.0, -6.0);
    solver.solve_quartic(1.0, 0.0, -5.0, 0.0, 4.0);
    assert!(solver.is_done());
    assert!(solver.infinite_roots());
    assert!(solver.solutions().is_err());
}
#[test]
fn overflow_leaves_previously_defined_roots_accessible_without_done_guard() {
    let mut solver: DirectPolynomialRoots = DirectPolynomialRoots::linear(1.0, -2.0);
    solver.solve_cubic(1.0, 1e60, 1.0, 1.0);
    assert!(!solver.is_done());
    assert_eq!(solver.solutions().unwrap(), 1);
    assert_eq!(solver.value(1).unwrap(), 2.0);
    solver.solve_linear(1.0, -3.0);
    assert!(!solver.is_done());
    assert_eq!(solver.value(1).unwrap(), 3.0);
}
#[test]
fn finite_solver_checks_one_based_root_range() {
    let solver: DirectPolynomialRoots = DirectPolynomialRoots::quadratic(1.0, -2.0, 1.0);
    assert_eq!(solver.solutions().unwrap(), 2);
    assert_eq!(
        solver.value(0).unwrap_err().exception_type(),
        "Standard_RangeError"
    );
    assert_eq!(
        solver.value(3).unwrap_err().exception_type(),
        "Standard_RangeError"
    );
    assert_eq!(solver.value(1).unwrap(), solver.value(2).unwrap());
}
