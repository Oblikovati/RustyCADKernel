#![allow(unsafe_code)]
//! Only completed objects become readable or receive caller-owned cleanup.
#[path = "array_default_construction/support.rs"]
mod support;
use rustycad_tkernel::ncollection::Array1;
use std::cell::RefCell;
use std::mem::MaybeUninit;
use std::rc::Rc;
use support::{
    DefaultCell, DefaultTrace, cleanup_prefix, construct_cell, native_row, prefix_values,
};

type DefaultCase = (&'static str, usize, usize, Option<usize>);
fn cases() -> [DefaultCase; 6] {
    [
        ("all", 0, 4, None),
        ("middle", 1, 3, None),
        ("fail_first", 1, 4, Some(0)),
        ("fail_late", 1, 4, Some(2)),
        ("empty", 2, 2, None),
        ("reverse", 3, 1, None),
    ]
}

fn construct_range(
    slots: &mut [MaybeUninit<DefaultCell>],
    from: usize,
    to: usize,
    trace: &Rc<RefCell<DefaultTrace>>,
) -> bool {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        Array1::construct_nontrivial_raw(slots.as_mut_ptr(), from, to, || construct_cell(trace));
    }));
    if let Err(payload) = result {
        assert_eq!(
            payload.downcast_ref::<&str>(),
            Some(&"default construction")
        );
        return true;
    }
    false
}

fn construction_state(
    slots: &[MaybeUninit<DefaultCell>],
    from: usize,
    trace: &DefaultTrace,
    failed: bool,
) -> String {
    let values = unsafe { prefix_values(slots, from, trace.completed) };
    format!(
        "{};{};{};{values};",
        trace.events,
        u8::from(failed),
        trace.completed
    )
}

fn default_case(slots: &mut [MaybeUninit<DefaultCell>], case: DefaultCase) {
    let (label, from, to, failure) = case;
    let trace = Rc::new(RefCell::new(DefaultTrace {
        failure,
        ..Default::default()
    }));
    let failed = construct_range(slots, from, to, &trace);
    let mut state = construction_state(slots, from, &trace.borrow(), failed);
    let completed = trace.borrow().completed;
    trace.borrow_mut().events.clear();
    unsafe { cleanup_prefix(slots, from, completed) };
    state += &trace.borrow().events;
    native_row(label, &state);
    assert_eq!(Rc::strong_count(&trace), 1);
}

#[test]
fn native_default_range_constructs_stack_prefix_and_unwinds_failed_members() {
    for case in cases() {
        let mut slots: [_; 4] = std::array::from_fn(|_| MaybeUninit::uninit());
        default_case(&mut slots, case);
    }
}

#[test]
fn native_default_range_constructs_heap_prefix_and_unwinds_failed_members() {
    for case in cases() {
        let mut slots: Vec<_> = (0..4).map(|_| MaybeUninit::uninit()).collect();
        default_case(&mut slots, case);
    }
}

#[test]
fn null_empty_and_reversed_ranges_do_not_invoke_default_constructor() {
    unsafe {
        Array1::<DefaultCell>::construct_nontrivial_raw(
            std::ptr::null_mut(),
            0,
            0,
            || unreachable!(),
        );
        Array1::<DefaultCell>::construct_nontrivial_raw(
            std::ptr::null_mut(),
            usize::MAX,
            0,
            || unreachable!(),
        );
    }
    native_row("null_empty", "");
}

#[test]
#[should_panic(expected = "element size 0; expected nonzero size for construction range 0..1")]
fn nonempty_zero_sized_default_construction_is_rejected_before_pointer_arithmetic() {
    unsafe {
        Array1::<()>::construct_nontrivial_raw(std::ptr::null_mut(), 0, 1, || unreachable!());
    }
}
