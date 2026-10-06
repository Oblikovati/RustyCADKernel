use super::fixture::{Node, PayloadTrace, caught};
use rustycad_tkernel::ncollection::BoundingNode;
fn default_row(label: &str, observed: String) {
    let expected =
        include_str!("../../../../tools/oracle/tkernel_ubtree_node_default_contracts.expected");
    let expected = expected
        .lines()
        .find(|line| line.starts_with(&format!("{label} ")))
        .unwrap();
    assert_eq!(observed, expected);
}
#[test]
fn node_default_constructs_bounds_before_object_with_detached_links() {
    let trace = PayloadTrace::new();
    let node = trace.with_defaults(|| Box::new(Node::default()));
    let state = (
        node.is_leaf(),
        node.is_root(),
        node.object().value,
        node.bounds().value,
    );
    trace.watch_node(&node);
    drop(node);
    successful_row(&trace, state);
}
fn successful_row(trace: &PayloadTrace, state: (bool, bool, i32, i32)) {
    default_row(
        "node_default",
        format!(
            "node_default {} {} {} {} {} {}",
            trace.events(),
            trace.stable(),
            u8::from(state.0),
            u8::from(state.1),
            state.2,
            state.3
        ),
    );
}
#[test]
fn node_default_failure_destroys_only_completed_members() {
    for kind in ['B', 'O'] {
        let trace = PayloadTrace::new();
        trace.reject('N', kind);
        let failed = trace.with_defaults(|| caught(|| drop(Node::default())));
        let label = format!("node_default_fail {kind}");
        default_row(
            &label,
            format!("{label} {} {}", trace.events(), u8::from(failed)),
        );
    }
}
#[test]
fn node_default_destructor_failure_preserves_member_order_and_addresses() {
    for kind in ['B', 'O'] {
        let trace = PayloadTrace::new();
        let node = trace.with_defaults(|| Box::new(Node::default()));
        trace.clear();
        trace.watch_node(&node);
        trace.reject('D', kind);
        let failed = caught(|| drop(node));
        let label = format!("node_default_drop {kind}");
        default_row(
            &label,
            format!(
                "{label} {} {} {}",
                trace.events(),
                u8::from(failed),
                trace.stable()
            ),
        );
    }
}
#[derive(Default)]
struct OnlyDefault(i32);
#[test]
fn default_requires_neither_clone_nor_bounding_volume() {
    let node = BoundingNode::<OnlyDefault, OnlyDefault>::default();
    assert_eq!(node.object().0, 0);
    assert_eq!(node.bounds().0, 0);
    assert!(node.is_leaf());
    assert!(node.is_root());
}
#[test]
fn default_allows_borrowed_and_zero_sized_payloads() {
    let node = BoundingNode::<&str, ()>::default();
    assert_eq!(*node.object(), "");
    assert_eq!(*node.bounds(), ());
    assert_eq!(node.child_id(0), None);
    assert_eq!(node.parent_id(), None);
}
