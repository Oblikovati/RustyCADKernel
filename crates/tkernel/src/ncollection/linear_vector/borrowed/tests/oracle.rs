use super::*;
const EXPECTED: &str =
    include_str!("../../../../../../../tools/oracle/tkernel_linear_borrowed.expected");
fn append_case(name: &str, capacity: usize, fail: bool) -> String {
    let trace = Rc::new(BorrowTrace::default());
    let donor = BorrowedItem::new(9, &trace);
    let mut target = LinearVector::with_capacity(capacity);
    trace.fail_copy.set(fail);
    let result = catch_unwind(AssertUnwindSafe(|| {
        target.append_ref(&donor);
    }));
    assert_eq!(result.is_err(), fail);
    row(name, &target, &donor)
}
fn set_case(name: &str, index: usize, fail: bool) -> String {
    let trace = Rc::new(BorrowTrace::default());
    let donor = BorrowedItem::new(9, &trace);
    let mut target = LinearVector::with_capacity(8);
    target.append(BorrowedItem::new(3, &trace));
    trace.fail_assign.set(fail);
    let result = catch_unwind(AssertUnwindSafe(|| {
        target.set_value_ref_with(index, &donor, || BorrowedItem::default_in(&trace));
    }));
    assert_eq!(result.is_err(), fail);
    row(name, &target, &donor)
}
#[test]
fn borrowed_copy_and_assignment_phases_join_native_rows() {
    let actual = append_case("append_grow_success", 0, false)
        + &append_case("append_grow_failed", 0, true)
        + &append_case("append_retained_success", 4, false)
        + &append_case("append_retained_failed", 4, true)
        + &set_case("set_existing_success", 0, false)
        + &set_case("set_existing_failed", 0, true)
        + &set_case("set_gap_success", 3, false)
        + &set_case("set_gap_failed", 3, true);
    assert_eq!(actual, EXPECTED);
}
