//! Exact Gemmate parent phases; native paired child allocation is explicitly outside this slice.
#[path = "ubtree_gemmate_parent_contracts/fixture.rs"]
mod fixture;
use fixture::*;
fn assert_parent_case(branch: bool, rejected: char) {
    let trace = ParentTrace::default();
    let mut tree = populated(&trace, branch);
    let root = tree.root_id().unwrap();
    trace.start(&tree, rejected);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tree.gemmate(root, trace.item(99), trace.item(3), trace.item(30));
    }));
    let (events, stable) = trace.finish();
    let prefix = format!("parent {} {rejected}", u8::from(branch));
    let state = parent_state(&tree, branch);
    let row = format!("{prefix} {events} {} {state}", u8::from(result.is_err()));
    assert_eq!(row, native_row(&prefix));
    assert!(stable);
    tree.clear();
    tree.allocator().reset(true).unwrap();
}
fn native_row(prefix: &str) -> &'static str {
    include_str!("../../../tools/oracle/tkernel_ubtree_gemmate_parent_contracts.expected")
        .lines()
        .find(|line| line.starts_with(prefix))
        .unwrap()
}
fn parent_state(tree: &Tree, branch: bool) -> String {
    let root = tree.root().unwrap();
    let first = tree.child(root, 0).unwrap();
    let second = tree.child(root, 1).unwrap();
    let parents = first.parent_id() == tree.root_id() && second.parent_id() == tree.root_id();
    let grandchildren = !branch
        || [0, 1]
            .iter()
            .all(|&child| tree.child(first, child).unwrap().parent_id() == root.child_id(0));
    format!(
        "{} {} {} {} {} {}",
        root.bounds().value,
        root.object().value,
        first.object().value,
        second.object().value,
        u8::from(parents),
        u8::from(grandchildren)
    )
}
#[test]
fn leaf_parent_assignments_retain_payload_addresses() {
    assert_parent_case(false, 'X');
}
#[test]
fn branch_parent_assignments_retain_payload_addresses() {
    assert_parent_case(true, 'X');
}
#[test]
fn leaf_bound_assignment_panic_preserves_published_children() {
    assert_parent_case(false, 'B');
}
#[test]
fn branch_bound_assignment_panic_preserves_reparented_grandchildren() {
    assert_parent_case(true, 'B');
}
#[test]
fn leaf_default_object_panic_follows_bound_assignment() {
    assert_parent_case(false, 'N');
}
#[test]
fn branch_default_object_panic_follows_bound_assignment() {
    assert_parent_case(true, 'N');
}
#[test]
fn leaf_object_assignment_panic_destroys_default_temporary() {
    assert_parent_case(false, 'O');
}
#[test]
fn branch_object_assignment_panic_destroys_default_temporary() {
    assert_parent_case(true, 'O');
}
