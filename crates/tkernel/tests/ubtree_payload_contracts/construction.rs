use super::fixture::{Node, Pair, PayloadTrace};
use rustycad_tkernel::ncollection::{UBTree, UBTreeFiller};
fn result(label: &str, trace: &PayloadTrace) {
    crate::native_row(
        label,
        format!("{label} {} {}", trace.events(), trace.stable()),
    );
    trace.clear();
}
#[test]
fn node_constructor_and_copy_preserve_native_order_and_drop_addresses() {
    let trace = PayloadTrace::new();
    let (object, bounds) = (trace.item(7), trace.item(8));
    let node = Box::new(Node::new_ref(&object, &bounds));
    trace.watch_node(&node);
    drop(node);
    result("node_ctor", &trace);
    let source = Node::new_ref(&object, &bounds);
    trace.clear();
    let node = Box::new(source.clone());
    trace.watch_node(&node);
    drop(node);
    result("node_copy", &trace);
}
#[test]
fn pair_constructor_copy_and_default_preserve_native_order_and_drop_addresses() {
    let trace = PayloadTrace::new();
    let (object, bounds) = (trace.item(7), trace.item(8));
    let pair = Box::new(Pair::new_ref(&object, &bounds));
    trace.watch_pair(&pair);
    drop(pair);
    result("pair_ctor", &trace);
    let source = Pair::new_ref(&object, &bounds);
    trace.clear();
    let pair = Box::new(source.clone());
    trace.watch_pair(&pair);
    drop(pair);
    result("pair_copy", &trace);
    let pair = trace.with_defaults(|| Box::new(Pair::default()));
    trace.watch_pair(&pair);
    drop(pair);
    result("pair_default", &trace);
}
#[test]
fn node_clone_from_assigns_members_in_place_in_native_order() {
    let trace = PayloadTrace::new();
    let source = Node::new(trace.item(7), trace.item(8));
    let mut target = Node::new(trace.item(1), trace.item(2));
    trace.watch_node(&target);
    target.clone_from(&source);
    assert_eq!((target.object().value, target.bounds().value), (7, 8));
    result("node_assign", &trace);
}
#[test]
fn pair_clone_from_preserves_public_field_values_and_native_order() {
    let trace = PayloadTrace::new();
    let source = Pair::new(trace.item(7), trace.item(8));
    let mut target = Pair::new(trace.item(1), trace.item(2));
    trace.watch_pair(&target);
    target.clone_from(&source);
    assert_eq!((target.object.value, target.bounds.value), (7, 8));
    result("pair_assign", &trace);
}
#[test]
fn filler_reset_drops_each_pending_bounds_before_its_object() {
    let trace = PayloadTrace::new();
    let mut tree = UBTree::default();
    let mut filler = UBTreeFiller::new(&mut tree, None, true);
    for value in 0..3 {
        filler.add(trace.item::<'O'>(value), trace.item::<'B'>(value));
    }
    trace.clear();
    filler.reset();
    result("filler_reset", &trace);
}
