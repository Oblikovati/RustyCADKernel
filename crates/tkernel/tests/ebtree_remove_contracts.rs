//! Exact native Remove callback prefixes plus Rust panic recovery of the leaf index.
#[path = "ebtree_remove_contracts/fixture.rs"]
mod fixture;
#[path = "ebtree_remove_contracts/recovery.rs"]
mod recovery;
use fixture::*;
fn assert_remove_case(phase: char, count: usize) {
    let trace = RemoveTrace::default();
    let mut tree = populated(&trace, 4);
    let expected = ancestor_addresses(&tree);
    trace.reset(phase, count);
    let failed = caught(|| {
        assert!(tree.remove(&1));
    });
    let row = remove_observation(&tree, &trace, &expected, &format!("{phase}{count}"), failed);
    assert_eq!(row, native_remove_observation(phase, count));
    trace.reset('\0', 0);
}
fn remove_observation(
    tree: &Editable,
    trace: &RemoveTrace,
    expected: &[usize],
    label: &str,
    failed: bool,
) -> String {
    let rebound = std::ptr::from_ref(tree.find_node(&4).unwrap().bounds()).addr() == expected[0];
    let (assignments, additions) = trace.addresses();
    format!(
        "remove {label} {} {} {} {} {} {} {}",
        trace.events(),
        u8::from(failed),
        u8::from(rebound),
        u8::from(expected.starts_with(&assignments)),
        u8::from(expected[1..].starts_with(&additions)),
        u8::from(!tree.contains(&1)),
        tree.as_ubtree().root().unwrap().bounds().value
    )
}
fn native_remove_observation(phase: char, count: usize) -> String {
    let mut columns: Vec<&str> = expected_row(&format!("remove {phase}{count} "))
        .split_whitespace()
        .collect();
    // Column 5 is C++ reentrant callback observation; Rust checks rebinding after caught panic.
    columns.remove(5);
    columns.join(" ")
}
#[test]
fn remove_rebinds_before_in_place_ancestor_updates() {
    assert_remove_case('N', 0);
}
#[test]
fn kill_assignment_panic_preserves_live_original_survivor() {
    assert_remove_case('A', 1);
}
#[test]
fn first_ancestor_assignment_panic_observes_rebound_survivor() {
    assert_remove_case('A', 2);
}
#[test]
fn second_ancestor_assignment_panic_observes_rebound_survivor() {
    assert_remove_case('A', 3);
}
#[test]
fn first_ancestor_add_panic_preserves_native_partial_bounds() {
    assert_remove_case('G', 1);
}
#[test]
fn second_ancestor_add_panic_preserves_native_partial_bounds() {
    assert_remove_case('G', 2);
}
fn assert_edge(count: i32, removed: i32) {
    let trace = RemoveTrace::default();
    let mut tree = populated(&trace, count);
    let removed_result = tree.remove(&removed);
    let events = trace.events();
    let row = format!(
        "edge {count} {removed} {} {} {}",
        u8::from(removed_result),
        u8::from(tree.is_empty()),
        if events.is_empty() { "empty" } else { &events }
    );
    assert_eq!(row, expected_row(&format!("edge {count} {removed} ")));
}
#[test]
fn remove_from_empty_tree_matches_native() {
    assert_edge(0, 1);
}
#[test]
fn remove_only_root_uses_clear() {
    assert_edge(1, 1);
}
#[test]
fn missing_key_leaves_bounds_untouched() {
    assert_edge(4, 99);
}
#[test]
fn root_absorption_needs_no_ancestor_update() {
    assert_edge(2, 2);
}

#[test]
fn absorbed_branch_updates_grandchild_parents_without_leaf_rebind() {
    assert_edge(4, 2);
    let trace = RemoveTrace::default();
    let mut tree = populated(&trace, 4);
    tree.remove(&2);
    let base = tree.as_ubtree();
    let root = base.root().unwrap();
    for child in [0, 1] {
        assert_eq!(base.child(root, child).unwrap().parent_id(), base.root_id());
    }
}
