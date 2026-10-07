#![allow(unsafe_code)]
//! Native nontrivial copying uses sequential, momentary, disjoint element borrows.
use rustycad_tkernel::ncollection::Array1;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

struct AssignmentCell {
    value: i32,
    deaths: Rc<Cell<usize>>,
}

impl Drop for AssignmentCell {
    fn drop(&mut self) {
        let previous = self.deaths.get();
        self.deaths.set(previous + 1);
    }
}

#[derive(Default)]
struct AssignmentTrace {
    operations: String,
    calls: usize,
    failure: Option<usize>,
}

impl AssignmentTrace {
    fn assign(&mut self, slot: &mut AssignmentCell, value: i32, same: bool) {
        let operation = if same { 'S' } else { 'A' };
        self.operations += &format!("{operation}{}:{value},", slot.value);
        slot.value = value + if same { 10 } else { 0 };
        let failed = self.failure == Some(self.calls);
        self.calls += 1;
        assert!(!failed, "assignment {} deliberately failed", self.calls - 1);
    }
}

fn assert_native_row(label: &str, value: impl std::fmt::Display) {
    let expected = include_str!("../../../tools/oracle/tkernel_array_nontrivial_raw_copy.expected");
    let row = format!("{label}={value}");
    assert!(
        expected.lines().any(|line| line == row),
        "missing native row {row}"
    );
}

fn cells(deaths: &Rc<Cell<usize>>) -> [AssignmentCell; 5] {
    std::array::from_fn(|offset| AssignmentCell {
        value: offset as i32 + 1,
        deaths: Rc::clone(deaths),
    })
}

fn copy_case(
    array: &mut Array1<'_, AssignmentCell>,
    target: usize,
    source: usize,
    trace: &RefCell<AssignmentTrace>,
) -> bool {
    let pointer = array.data_mut_ptr();
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        Array1::copy_assign_nontrivial_raw(
            pointer.add(target),
            pointer.add(source),
            4,
            |slot, donor| trace.borrow_mut().assign(slot, donor.value, false),
            |slot| trace.borrow_mut().assign(slot, slot.value, true),
        );
    }))
    .is_err()
}

fn check_case(
    array: &mut Array1<'_, AssignmentCell>,
    case: (&str, usize, usize, Option<usize>),
    deaths: &Cell<usize>,
) {
    let (label, target, source, failure) = case;
    let trace = RefCell::new(AssignmentTrace {
        failure,
        ..Default::default()
    });
    let pointer = array.data_ptr();
    let failed = copy_case(array, target, source, &trace);
    let state = native_state(array.as_slice(), &trace.borrow(), failed, deaths);
    assert_native_row(label, state);
    assert_eq!(array.data_ptr(), pointer);
}

fn native_state(
    values: &[AssignmentCell],
    trace: &AssignmentTrace,
    failed: bool,
    deaths: &Cell<usize>,
) -> String {
    let values = values
        .iter()
        .map(|cell| cell.value.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{values};{};{};{}",
        trace.operations,
        u8::from(failed),
        deaths.get()
    )
}

fn cases() -> [(&'static str, usize, usize, Option<usize>); 7] {
    [
        ("forward", 1, 0, None),
        ("reverse", 0, 1, None),
        ("same", 0, 0, None),
        ("forward_fail_first", 1, 0, Some(0)),
        ("forward_fail_late", 1, 0, Some(2)),
        ("reverse_fail_late", 0, 1, Some(2)),
        ("same_fail_late", 0, 0, Some(2)),
    ]
}

#[test]
fn native_nontrivial_overlap_preserves_borrowed_storage_and_failure_prefix() {
    for (label, target, source, failure) in cases() {
        let deaths = Rc::new(Cell::new(0));
        let mut buffer = cells(&deaths);
        let mut array = Array1::from_slice(&mut buffer);
        check_case(&mut array, (label, target, source, failure), &deaths);
        assert!(!array.is_deletable());
        drop(array);
        assert_eq!(deaths.get(), 0);
        drop(buffer);
        assert_eq!(deaths.get(), 5);
    }
}

#[test]
fn native_nontrivial_overlap_preserves_heap_storage_and_failure_prefix() {
    for (label, target, source, failure) in cases() {
        let deaths = Rc::new(Cell::new(0));
        let mut buffer = Vec::from(cells(&deaths));
        let mut array = Array1::from_slice_bounds(&mut buffer, -2, 2).unwrap();
        check_case(&mut array, (label, target, source, failure), &deaths);
        assert!(!array.is_deletable());
        assert_eq!(array.lower(), -2);
        drop(array);
        assert_eq!(deaths.get(), 0);
        drop(buffer);
        assert_eq!(deaths.get(), 5);
    }
}

#[test]
fn zero_count_null_pointers_do_not_read_or_invoke_callbacks() {
    unsafe {
        Array1::<AssignmentCell>::copy_assign_nontrivial_raw(
            std::ptr::null_mut(),
            std::ptr::null(),
            0,
            |_, _| panic!("unexpected assignment"),
            |_| panic!("unexpected self assignment"),
        );
    }
    assert_native_row("zero", 0);
}

#[test]
#[should_panic(expected = "element size 0; expected nonzero size for 1 slots")]
fn zero_sized_elements_are_rejected_before_pointer_arithmetic() {
    unsafe {
        Array1::<()>::copy_assign_nontrivial_raw(
            std::ptr::null_mut(),
            std::ptr::null(),
            1,
            |_, _| unreachable!(),
            |_| unreachable!(),
        );
    }
}
