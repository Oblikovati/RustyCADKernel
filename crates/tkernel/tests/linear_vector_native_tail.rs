//! Native spare lifetime transitions use borrowed elements, so omitted Drop never leaks allocation.
use rustycad_tkernel::ncollection::LinearVector;
use std::{
    cell::{Cell, RefCell},
    panic::{AssertUnwindSafe, catch_unwind},
};
#[path = "linear_vector_native_tail/item.rs"]
mod item;
#[path = "linear_vector_native_tail/transitions.rs"]
mod transitions;
use item::{TailItem, TailTrace};

fn assert_row(row: &str) {
    let expected = include_str!("../../../tools/oracle/tkernel_linear_native_tail.expected");
    assert!(
        expected.lines().any(|native| native == row),
        "missing native row {row:?}"
    );
}

fn emit(name: &str, phase: &str, vector: &LinearVector<TailItem<'_>>, trace: &TailTrace) {
    let values: String = vector
        .iter()
        .map(|item| format!("{},", item.value))
        .collect();
    assert_row(&format!(
        "{name}:{phase}:{}:{}:{}:{values}",
        vector.len(),
        vector.capacity(),
        trace.events.borrow()
    ));
}

fn matrix(capacity: usize, fill: bool) {
    for fail in 1..=3 {
        for action in [
            "clear-keep",
            "clear-free",
            "reserve",
            "drop",
            "retry-success",
            "retry-first",
            "retry-second",
        ] {
            transitions::run(capacity, fill, fail, action);
        }
    }
}

#[test]
fn retained_default_failure_lifetime_matrix() {
    matrix(8, false);
}
#[test]
fn retained_fill_failure_lifetime_matrix() {
    matrix(8, true);
}
#[test]
fn grown_default_failure_lifetime_matrix() {
    matrix(2, false);
}
#[test]
fn grown_fill_failure_lifetime_matrix() {
    matrix(2, true);
}
