//! Native comparisons retain observable callback order and separate ordering from end equality.
use rustycad_tkernel::ncollection::{BaseMapIndexedIterator, IndexedCursorSource};
use std::{cell::RefCell, rc::Rc};
#[path = "base_map_indexed_comparisons/support.rs"]
mod support;
use support::{ProbeMap, take_trace};
type Read = BaseMapIndexedIterator<ProbeMap>;
type Write = BaseMapIndexedIterator<ProbeMap, true>;

fn equality_row(
    label: &str,
    left: &Write,
    first: &ProbeMap,
    right: &Read,
    second: &ProbeMap,
    calls: &Rc<RefCell<String>>,
) -> String {
    let equal = left.is_equal(first, right, second).unwrap();
    let equal_trace = take_trace(calls);
    let unequal = left.is_not_equal(first, right, second).unwrap();
    format!(
        "{label} {} {equal_trace} {} {}\n",
        u8::from(equal),
        u8::from(unequal),
        take_trace(calls)
    )
}

const EQUALITY_CASES: [(&str, Option<usize>, Option<usize>, bool); 8] = [
    ("live_same", Some(1), Some(1), true),
    ("live_foreign", Some(1), Some(1), false),
    ("end_live", Some(4), Some(1), false),
    ("both_end", Some(4), Some(5), false),
    ("null_live", None, Some(1), false),
    ("live_null", Some(1), None, false),
    ("both_null", None, None, false),
    ("same_end", Some(4), Some(4), true),
];

fn comparison_cursor<const M: bool>(
    map: &ProbeMap,
    index: Option<usize>,
) -> BaseMapIndexedIterator<ProbeMap, M> {
    match index {
        Some(index) => BaseMapIndexedIterator::new(map, index),
        None => BaseMapIndexedIterator::default(),
    }
}

fn equality_rows() -> String {
    let calls = Rc::new(RefCell::new(String::new()));
    let first = ProbeMap::new('a', 3, &calls);
    let second = ProbeMap::new('b', 3, &calls);
    let mut rows = String::new();
    for (label, left_index, right_index, same_owner) in EQUALITY_CASES {
        let owner = if same_owner { &first } else { &second };
        let left = comparison_cursor(&first, left_index);
        let right = comparison_cursor(owner, right_index);
        rows.push_str(&equality_row(label, &left, &first, &right, owner, &calls));
    }
    rows
}

fn ordering_row(label: &str, left_index: usize, right_index: usize) -> String {
    let calls = Rc::new(RefCell::new(String::new()));
    let first = ProbeMap::new('a', 0, &calls);
    let second = ProbeMap::new('b', 0, &calls);
    let left = Write::new(&first, left_index);
    let right = Read::new(&second, right_index);
    format!(
        "{label} {} {} {} {} {}\n",
        u8::from(left.is_less(&right)),
        u8::from(left.is_greater(&right)),
        u8::from(left.is_less_or_equal(&right)),
        u8::from(left.is_greater_or_equal(&right)),
        take_trace(&calls)
    )
}

#[test]
fn native_comparison_rows_match_callback_order_and_results() {
    let mut rows = equality_rows();
    for (label, left, right) in [
        ("less", 1, 2),
        ("greater", 2, 1),
        ("same_position", 2, 2),
        ("signed_max_right", 0, isize::MAX as usize),
        ("signed_max_left", isize::MAX as usize, 0),
    ] {
        rows.push_str(&ordering_row(label, left, right));
    }
    assert_eq!(
        rows,
        include_str!("../../../tools/oracle/tkernel_base_map_indexed_comparisons.expected")
    );
}

#[test]
fn skipped_right_owner_validation_follows_native_more_short_circuit() {
    let calls = Rc::new(RefCell::new(String::new()));
    let first = ProbeMap::new('a', 3, &calls);
    let other = ProbeMap::new('b', 3, &calls);
    let left = Write::new(&first, 1);
    let right = Read::new(&other, 1);
    assert!(!left.is_equal(&first, &right, &first).unwrap());
    assert_eq!(take_trace(&calls), "a");
    let exhausted = Write::new(&first, 4);
    assert!(exhausted.is_equal(&first, &right, &first).is_err());
    assert_eq!(take_trace(&calls), "a");
    assert!(exhausted.is_not_equal(&first, &right, &first).is_err());
    assert_eq!(take_trace(&calls), "a");
}

#[test]
fn probe_accessors_are_independent_of_size_trace() {
    let calls = Rc::new(RefCell::new(String::new()));
    let source = ProbeMap::new('x', 3, &calls);
    assert_eq!(source.indexed_key(1).unwrap(), &42);
    assert_eq!(source.indexed_value(1).unwrap(), &42);
    assert_eq!(take_trace(&calls), "-");
    assert_eq!(source.indexed_size(), 3);
    assert_eq!(take_trace(&calls), "x");
}
