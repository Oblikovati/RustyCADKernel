//! Native copy-assignment publication and spare lifetime transitions with borrowed payloads.
use rustycad_tkernel::ncollection::LinearVector;
use std::{
    cell::{Cell, RefCell},
    panic::{AssertUnwindSafe, catch_unwind},
};
#[path = "linear_vector_assignment_tail/item.rs"]
mod item;
#[path = "linear_vector_assignment_tail/transitions.rs"]
mod transitions;
use item::{AssignmentItem, AssignmentTrace};

fn assert_row(row: &str) {
    let expected = include_str!("../../../tools/oracle/tkernel_linear_assignment_tail.expected");
    assert!(
        expected.lines().any(|native| native == row),
        "missing native assignment row {row:?}"
    );
}

#[test]
fn empty_receiver_tail_failure_retry_and_release_matrix() {
    transitions::matrix(0);
}
#[test]
fn assigned_prefix_tail_failure_retry_and_release_matrix() {
    transitions::matrix(2);
}
#[test]
fn common_assignment_failure_preserves_mutations_and_skips_tail() {
    for fail in 1..=2 {
        for action in ["drop", "retry-success"] {
            transitions::run(
                &format!("common-{fail}-{action}"),
                8,
                2,
                5,
                (0, fail),
                action,
            );
        }
    }
}
#[test]
fn normal_equal_shrink_empty_and_growth_branches() {
    for (name, capacity, count, source) in [
        ("equal", 8, 3, 3),
        ("shrink", 8, 5, 3),
        ("empty-source", 8, 3, 0),
        ("empty-both", 0, 0, 0),
        ("grow-success", 1, 1, 5),
    ] {
        transitions::run(name, capacity, count, source, (0, 0), "drop");
    }
}
