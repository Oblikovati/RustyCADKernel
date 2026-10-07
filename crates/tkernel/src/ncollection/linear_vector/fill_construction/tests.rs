//! Fill failure observes the common constructor owner; normal kernel calls retain native leaks.
#![allow(unsafe_code)]
use super::super::LinearVector;
use std::{
    cell::Cell,
    panic::{AssertUnwindSafe, catch_unwind},
};
#[path = "../../../../tests/support/linear_fill_constructor_fixture.rs"]
mod fixture;
use super::super::constructor_test_support::{item, recovery};
mod safety;
use fixture::{CopyItem, CopyTrace, FILL_CASES, FillSpec, row};

fn assert_native(row: &str) {
    let expected =
        include_str!("../../../../../../tools/oracle/tkernel_linear_fill_constructor.expected");
    assert!(
        expected.lines().any(|native| native == row),
        "missing native fill row {row:?}"
    );
}

fn run(spec: &FillSpec) {
    let trace = CopyTrace::default();
    let donor = (!spec.owned).then(|| CopyItem::new(99, &trace));
    trace.failing.set(spec.fail);
    trace.active.set(true);
    let abandoned = Cell::new(None);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let temporary_donor = spec.owned.then(|| CopyItem::new(99, &trace));
        let value = temporary_donor.as_ref().or(donor.as_ref()).unwrap();
        LinearVector::construct_fill(spec.count, value, |vector| {
            assert_eq!(vector.len(), 0);
            abandoned.set(Some((vector.data_mut_ptr(), vector.capacity())));
        })
    }));
    assert_eq!(result.is_err(), spec.fail != 0);
    assert_native(&row(
        "native",
        spec,
        result.as_ref().ok(),
        &trace,
        result.is_ok(),
    ));
    if let Some(allocation) = abandoned.get() {
        assert_eq!(
            result.as_ref().err().unwrap().downcast_ref::<usize>(),
            Some(&spec.fail)
        );
        recovery::repair(
            spec.name,
            spec.count.max(2),
            &trace,
            allocation,
            assert_native,
        );
        assert_native(&row("repaired", spec, None, &trace, false));
    }
    trace.active.set(false);
    drop((result, donor));
    assert_eq!(trace.resources.get(), 0);
}

macro_rules! native_fill {
    ($name:ident,$index:literal) => {
        #[test]
        fn $name() {
            run(&FILL_CASES[$index]);
        }
    };
}
native_fill!(borrowed_zero, 0);
native_fill!(borrowed_one, 1);
native_fill!(borrowed_two, 2);
native_fill!(borrowed_three, 3);
native_fill!(borrowed_five, 4);
native_fill!(borrowed_one_first, 5);
native_fill!(borrowed_three_first, 6);
native_fill!(borrowed_middle, 7);
native_fill!(borrowed_final, 8);
native_fill!(borrowed_long_final, 9);
native_fill!(owned_zero, 10);
native_fill!(owned_one, 11);
native_fill!(owned_two, 12);
native_fill!(owned_three, 13);
native_fill!(owned_five, 14);
native_fill!(owned_one_first, 15);
native_fill!(owned_three_first, 16);
native_fill!(owned_middle, 17);
native_fill!(owned_final, 18);
native_fill!(owned_long_final, 19);

#[test]
fn previous_rust_rollback_regression_now_asserts_native_retained_fill_prefix() {
    // Replaces the former integration-only automatic-tail-drop expectation. This same
    // public failure is separately compared by the non-recovering constructor probe.
    run(&FILL_CASES[7]);
}
