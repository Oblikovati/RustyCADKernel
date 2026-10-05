//! Native EmplaceValue constructs gap defaults before the requested constructor.
use rustycad_tkernel::ncollection::DynamicArray;
use std::{
    cell::Cell,
    panic::{AssertUnwindSafe, catch_unwind},
};

struct FailingDefault;
impl Default for FailingDefault {
    fn default() -> Self {
        panic!("requested default failure: expected gap construction before supplied constructor")
    }
}

#[test]
fn a_failing_gap_default_prevents_the_requested_constructor() {
    let supplied_constructor_called = Cell::new(false);
    let mut array = DynamicArray::<FailingDefault>::with_increment(4);
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        let _ = array.emplace_value(1, || {
            supplied_constructor_called.set(true);
            FailingDefault
        });
    }));
    assert!(outcome.is_err());
    assert!(!supplied_constructor_called.get());
    assert!(array.is_empty());
}
