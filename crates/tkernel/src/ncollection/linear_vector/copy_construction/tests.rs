//! Instrumented construction shares the production body; only tests recover native leaks.
#![allow(unsafe_code)]
use super::super::LinearVector;
use std::{
    cell::Cell,
    panic::{AssertUnwindSafe, catch_unwind},
};
#[path = "../../../../tests/support/linear_copy_constructor_fixture.rs"]
mod fixture;
#[path = "../../../../tests/support/linear_constructor_recovery.rs"]
mod recovery;
mod safety;
use fixture::{COPY_CASES, CopySpec, CopyTrace, filled, row};

fn assert_native(row: &str) {
    let expected =
        include_str!("../../../../../../tools/oracle/tkernel_linear_copy_constructor.expected");
    assert!(
        expected.lines().any(|native| native == row),
        "missing native constructor row {row:?}"
    );
}

fn run(spec: &CopySpec) {
    let trace = CopyTrace::default();
    let source = filled(spec.capacity, spec.count, 10, &trace);
    let mut receiver = filled(spec.receiver_count, spec.receiver_count, 90, &trace);
    let before = if spec.assigning {
        receiver.data_ptr()
    } else {
        source.data_ptr()
    };
    let abandoned = Cell::new(None);
    let mut copied = None;
    trace.failing.set(spec.fail);
    trace.active.set(true);
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        let temporary = source.construct_copy(|vector| {
            assert_eq!(vector.len(), 0);
            abandoned.set(Some((vector.data_mut_ptr(), vector.capacity())));
        });
        if spec.assigning {
            receiver = temporary;
        } else {
            copied = Some(temporary);
        }
    }));
    assert_eq!(outcome.is_err(), spec.fail != 0);
    let result = if spec.assigning {
        Some(&receiver)
    } else {
        copied.as_ref()
    };
    assert_native(&row(
        "native",
        spec,
        &source,
        result,
        before,
        &trace,
        outcome.is_ok(),
    ));
    if let Some(allocation) = abandoned.get() {
        assert_eq!(
            outcome.as_ref().unwrap_err().downcast_ref::<usize>(),
            Some(&spec.fail)
        );
        recovery::repair(spec.name, spec.count, &trace, allocation);
        assert_native(&row(
            "repaired", spec, &source, result, before, &trace, false,
        ));
    }
    trace.active.set(false);
    drop((copied, receiver, source));
    assert_eq!(trace.resources.get(), 0);
}

macro_rules! native_copy {
    ($name:ident, $index:literal) => {
        #[test]
        fn $name() {
            run(&COPY_CASES[$index]);
        }
    };
}
native_copy!(clone_empty, 0);
native_copy!(clone_retained_empty, 1);
native_copy!(clone_single, 2);
native_copy!(clone_retained, 3);
native_copy!(clone_first_failure, 4);
native_copy!(clone_middle_failure, 5);
native_copy!(clone_final_failure, 6);
native_copy!(clone_long_final_failure, 7);
native_copy!(assignment_empty_receiver, 8);
native_copy!(assignment_occupied_receiver, 9);
native_copy!(assignment_first_failure, 10);
native_copy!(assignment_middle_failure, 11);
native_copy!(assignment_final_failure, 12);
native_copy!(assignment_empty_first_failure, 13);
