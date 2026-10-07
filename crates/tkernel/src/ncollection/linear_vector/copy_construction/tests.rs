//! Native copy construction and copy assignment on the success path.
//! Cases with a throwing copy (`fail != 0`) are D1 class B2 and are not reproduced.
use super::super::LinearVector;
#[path = "../../../../tests/support/linear_copy_constructor_fixture.rs"]
mod fixture;
use super::super::constructor_test_support::item;
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
    assert_eq!(
        spec.fail, 0,
        "case {}: expected a nonthrowing copy",
        spec.name
    );
    let trace = CopyTrace::default();
    let source = filled(spec.capacity, spec.count, 10, &trace);
    let mut receiver = filled(spec.receiver_count, spec.receiver_count, 90, &trace);
    let before = if spec.assigning {
        receiver.data_ptr()
    } else {
        source.data_ptr()
    };
    let mut copied = None;
    trace.active.set(true);
    let temporary = source.clone();
    if spec.assigning {
        receiver = temporary;
    } else {
        copied = Some(temporary);
    }
    let result = if spec.assigning {
        Some(&receiver)
    } else {
        copied.as_ref()
    };
    assert_native(&row("native", spec, &source, result, before, &trace, true));
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
native_copy!(assignment_empty_receiver, 8);
native_copy!(assignment_occupied_receiver, 9);
