#![allow(unsafe_code)]
//! Construction owns each completed raw slot until the caller explicitly drops it.
use rustycad_tkernel::ncollection::Array1;
use std::cell::RefCell;
use std::mem::MaybeUninit;
use std::rc::Rc;

#[derive(Default)]
struct ConstructionTrace {
    events: String,
    calls: usize,
    completed: usize,
    deaths: usize,
    failure: Option<usize>,
}

struct ConstructionCell {
    value: i32,
    trace: Rc<RefCell<ConstructionTrace>>,
}

impl Drop for ConstructionCell {
    fn drop(&mut self) {
        let mut trace = self.trace.borrow_mut();
        trace.events += &format!("D{},", self.value);
        trace.deaths += 1;
    }
}

fn source_cells(trace: &Rc<RefCell<ConstructionTrace>>) -> [ConstructionCell; 3] {
    std::array::from_fn(|offset| ConstructionCell {
        value: offset as i32 + 1,
        trace: Rc::clone(trace),
    })
}

fn construct_cell(donor: &ConstructionCell) -> ConstructionCell {
    let mut trace = donor.trace.borrow_mut();
    trace.events += &format!("C{},", donor.value);
    let failed = trace.failure == Some(trace.calls);
    trace.calls += 1;
    assert!(!failed, "construction");
    trace.completed += 1;
    ConstructionCell {
        value: donor.value + 10,
        trace: Rc::clone(&donor.trace),
    }
}

fn invoke_construction(
    target: &mut [MaybeUninit<ConstructionCell>],
    source: &[ConstructionCell],
) -> bool {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        Array1::copy_construct_nontrivial_raw(
            target.as_mut_ptr(),
            source.as_ptr(),
            source.len(),
            construct_cell,
        );
    }));
    if let Err(payload) = result {
        assert_eq!(payload.downcast_ref::<&str>(), Some(&"construction"));
        return true;
    }
    false
}

fn prefix_values(target: &[MaybeUninit<ConstructionCell>], count: usize) -> String {
    target[..count]
        .iter()
        .map(|slot| unsafe { slot.assume_init_ref() }.value.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn drop_constructed_prefix(target: &mut [MaybeUninit<ConstructionCell>], count: usize) {
    for slot in &mut target[..count] {
        unsafe { slot.assume_init_drop() };
    }
}

fn assert_native_row(label: &str, value: &str) {
    let expected = include_str!("../../../tools/oracle/tkernel_array_raw_construction.expected");
    let row = format!("{label}={value}");
    assert!(
        expected.lines().any(|line| line == row),
        "missing native row {row}"
    );
}

fn construction_state(
    target: &[MaybeUninit<ConstructionCell>],
    trace: &ConstructionTrace,
    failed: bool,
) -> String {
    let values = prefix_values(target, trace.completed);
    let (events, completed, deaths) = (&trace.events, trace.completed, trace.deaths);
    format!(
        "{values};{events};{};{completed};{deaths};",
        u8::from(failed)
    )
}

fn construction_case(
    target: &mut [MaybeUninit<ConstructionCell>],
    case: (&str, Option<usize>, usize),
) {
    let (label, failure, count) = case;
    let trace = Rc::new(RefCell::new(ConstructionTrace {
        failure,
        ..Default::default()
    }));
    let source = source_cells(&trace);
    let failed = invoke_construction(target, &source[..count]);
    let mut state = construction_state(target, &trace.borrow(), failed);
    let completed = trace.borrow().completed;
    trace.borrow_mut().events.clear();
    drop_constructed_prefix(target, completed);
    state += &trace.borrow().events;
    assert_native_row(label, &state);
    assert_eq!(trace.borrow().deaths, completed);
    assert_eq!(source.each_ref().map(|cell| cell.value), [1, 2, 3]);
}

fn construction_cases() -> [(&'static str, Option<usize>, usize); 4] {
    [
        ("success", None, 3),
        ("fail_first", Some(0), 3),
        ("fail_late", Some(2), 3),
        ("empty", None, 0),
    ]
}

#[test]
fn native_raw_construction_retains_stack_prefix_until_explicit_cleanup() {
    for (label, failure, count) in construction_cases() {
        let mut target: [_; 3] = std::array::from_fn(|_| MaybeUninit::uninit());
        construction_case(&mut target, (label, failure, count));
    }
}

#[test]
fn native_raw_construction_retains_heap_prefix_until_explicit_cleanup() {
    for (label, failure, count) in construction_cases() {
        let mut target: Vec<_> = (0..3).map(|_| MaybeUninit::uninit()).collect();
        construction_case(&mut target, (label, failure, count));
    }
}

#[test]
fn raw_construction_zero_count_accepts_null_without_callbacks() {
    unsafe {
        Array1::<ConstructionCell>::copy_construct_nontrivial_raw(
            std::ptr::null_mut(),
            std::ptr::null(),
            0,
            |_| panic!("unexpected construction"),
        );
    }
    assert_native_row("zero_null", "0");
}

#[test]
#[should_panic(expected = "element size 0; expected nonzero size for 1 construction slots")]
fn raw_construction_zero_sized_elements_are_rejected_before_pointer_arithmetic() {
    unsafe {
        Array1::<()>::copy_construct_nontrivial_raw(
            std::ptr::null_mut(),
            std::ptr::null(),
            1,
            |_| unreachable!(),
        );
    }
}

#[test]
fn raw_construction_preserves_borrowed_payload_lifetimes_and_destination_tail() {
    let text = String::from("lifetime");
    let source = [&text];
    let mut target = [MaybeUninit::uninit(), MaybeUninit::new(&text)];
    unsafe {
        Array1::copy_construct_nontrivial_raw(target.as_mut_ptr(), source.as_ptr(), 1, |value| {
            *value
        });
        assert!(std::ptr::eq(target[0].assume_init(), &text));
        assert!(std::ptr::eq(target[1].assume_init(), &text));
    }
}
