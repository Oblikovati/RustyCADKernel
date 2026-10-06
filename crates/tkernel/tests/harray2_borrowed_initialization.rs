//! HArray2 constructor oracles use the shared assignment-only item from HArray1.
#[path = "harray2_borrowed_initialization/default_failure.rs"]
mod default_failure;
#[path = "harray1_borrowed_initialization/fixture.rs"]
mod fixture;
#[path = "harray2_borrowed_initialization/native_rows.rs"]
mod native_rows;
use fixture::{ClonePolicy, FillProbe, FillScope};
use rustycad_tkernel::ncollection::{ArrayError, HArray2};
use std::panic::{AssertUnwindSafe, catch_unwind};

#[test]
fn native_matrix_rows_match_assignment_failures_empty_rectangles_and_handles() {
    assert_eq!(
        native_rows::matrix_rows(),
        include_str!("../../../tools/oracle/tkernel_harray2_borrowed_initialization.expected")
    );
}
#[test]
fn borrowed_matrix_clone_policy_assigns_without_copy_construction() {
    let scope = FillScope::new(0);
    let donor = ClonePolicy(scope.donor(9));
    let matrix = HArray2::with_value_ref(-1, 0, 3, 4, &donor).unwrap();
    assert_eq!(matrix.array2().value(0, 3).unwrap().0.number, 9);
    assert_eq!(donor.0.reads(), 4);
    assert_eq!(scope.take_events(), "d1;d2;d3;d4;a1=9;a2=9;a3=9;a4=9;");
    assert!(catch_unwind(AssertUnwindSafe(|| donor.clone())).is_err());
}
#[test]
fn invalid_matrix_bounds_validate_rows_before_columns_without_assignment() {
    let scope = FillScope::new(0);
    let donor = scope.donor(9);
    let both = HArray2::with_value_ref_with_assignment(3, 0, 9, 1, &donor, FillProbe::assign);
    let col = HArray2::with_value_ref_with_assignment(1, 2, 9, 1, &donor, FillProbe::assign);
    assert!(matches!(
        both,
        Err(ArrayError::Bounds { lower: 3, upper: 0 })
    ));
    assert!(matches!(
        col,
        Err(ArrayError::Bounds { lower: 9, upper: 1 })
    ));
    assert_eq!((donor.number, donor.reads()), (9, 0));
    assert_eq!(scope.take_events(), "-");
}
#[test]
fn safe_default_failure_drops_completed_prefix_and_skips_assignment() {
    for failure in 1..=4 {
        default_failure::check_default_failure(failure);
    }
}
#[test]
fn owned_matrix_convenience_uses_the_same_assignment_phase() {
    let scope = FillScope::new(0);
    let donor = ClonePolicy(scope.donor(9));
    let matrix = HArray2::with_value(-1, 0, 3, 4, donor).unwrap();
    assert_eq!(matrix.array2().len(), 4);
    assert_eq!(scope.take_events(), "d1;d2;d3;d4;a1=9;a2=9;a3=9;a4=9;x0=9;");
}
