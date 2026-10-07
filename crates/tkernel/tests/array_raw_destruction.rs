#![allow(unsafe_code)]
//! Dead raw slots never become typed values or participate in automatic cleanup.
#[path = "array_raw_destruction/support.rs"]
mod support;
use rustycad_tkernel::ncollection::Array1;
use std::cell::RefCell;
use std::mem::MaybeUninit;
use std::rc::Rc;
use support::{
    DestructionCell, DestructionTrace, cleanup_survivors, invoke_destruction, live_slots,
    native_row, survivor_values,
};

type DestructionCase = (&'static str, usize, usize, Option<usize>);

fn cases() -> [DestructionCase; 6] {
    [
        ("all", 0, 4, None),
        ("middle", 1, 3, None),
        ("fail_first", 1, 4, Some(1)),
        ("fail_late", 0, 4, Some(2)),
        ("empty", 2, 2, None),
        ("reverse", 3, 1, None),
    ]
}

fn make_trace(failure: Option<usize>) -> Rc<RefCell<DestructionTrace>> {
    Rc::new(RefCell::new(DestructionTrace {
        failure,
        live: [true; 4],
        ..Default::default()
    }))
}

fn live_bitmap(trace: &DestructionTrace) -> String {
    trace
        .live
        .iter()
        .map(|alive| if *alive { '1' } else { '0' })
        .collect()
}

fn case_state(
    slots: &[MaybeUninit<DestructionCell>],
    trace: &DestructionTrace,
    failed: bool,
) -> String {
    let bitmap = live_bitmap(trace);
    let values = unsafe { survivor_values(slots, &trace.live) };
    format!("{};{};{bitmap};{values};", trace.events, u8::from(failed))
}

fn destruction_case(
    slots: &mut [MaybeUninit<DestructionCell>],
    trace: &RefCell<DestructionTrace>,
    case: DestructionCase,
) {
    let (label, from, to, _) = case;
    let pointer = slots.as_mut_ptr();
    let failed = unsafe { invoke_destruction(pointer.cast(), from, to) };
    let mut state = case_state(slots, &trace.borrow(), failed);
    unsafe { cleanup_survivors(slots, trace) };
    state += &trace.borrow().events;
    native_row(label, &state);
    assert_eq!(trace.borrow().live, [false; 4]);
    assert_eq!(slots.as_mut_ptr(), pointer);
}

#[test]
fn native_raw_destruction_matches_stack_lifetimes_failures_and_member_cleanup() {
    for case in cases() {
        let trace = make_trace(case.3);
        let mut slots = live_slots(&trace);
        destruction_case(&mut slots, &trace, case);
        assert_eq!(Rc::strong_count(&trace), 1);
    }
}

#[test]
fn native_raw_destruction_matches_heap_lifetimes_failures_and_member_cleanup() {
    for case in cases() {
        let trace = make_trace(case.3);
        let mut slots = Vec::from(live_slots(&trace));
        destruction_case(&mut slots, &trace, case);
        assert_eq!(Rc::strong_count(&trace), 1);
    }
}

#[test]
fn null_empty_and_reversed_ranges_perform_no_pointer_arithmetic_or_drop() {
    unsafe {
        Array1::<DestructionCell>::destroy_nontrivial_raw(std::ptr::null_mut(), 0, 0);
        Array1::<DestructionCell>::destroy_nontrivial_raw(std::ptr::null_mut(), usize::MAX, 0);
    }
    native_row("null_empty", "");
}

#[test]
#[should_panic(expected = "element size 0; expected nonzero size for destruction range 0..1")]
fn nonempty_zero_sized_elements_are_rejected_before_pointer_arithmetic() {
    unsafe {
        Array1::<()>::destroy_nontrivial_raw(std::ptr::null_mut(), 0, 1);
    }
}
