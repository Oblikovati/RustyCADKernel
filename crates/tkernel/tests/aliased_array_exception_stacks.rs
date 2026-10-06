//! Positive native stack capture is isolated from other process-global stack settings.
#![cfg(not(miri))]
use rustycad_tkernel::{ncollection::AliasedArray, standard::Failure};
#[path = "aliased_array_exceptions/support.rs"]
mod support;
use support::record;
// Real frame capture uses libc and is checked outside Miri's FFI domain.
#[test]
fn native_message_only_errors_capture_configured_stack() {
    let previous = Failure::default_stack_trace_length();
    Failure::set_default_stack_trace_length(3);
    let mut array = AliasedArray::new(4, 2).unwrap();
    let mut rows = Vec::new();
    record(&mut rows, "type", array.value::<f64>(0));
    record(&mut rows, "range", AliasedArray::empty(0));
    record(
        &mut rows,
        "dimension",
        array.assign(&AliasedArray::new(8, 2).unwrap()),
    );
    record(&mut rows, "index", array.value_signed::<i32>(-1));
    Failure::set_default_stack_trace_length(previous);
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!("../../../tools/oracle/tkernel_aliased_array_exception_stacks.expected")
    );
}
