use super::fixture::{NodeItem, NodeTrace, borrowed_node, native_node_row, node_caught};
use rustycad_tkernel::ncollection::{DoubleMapNode, MapLinkNode};
#[test]
fn native_node_allocation_precedes_both_copies() {
    let trace = NodeTrace::new();
    let key = trace.item('K');
    let value = trace.item('V');
    let _node = DoubleMapNode::allocate_ref(
        &trace.arena,
        &key,
        &value,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    native_node_row("allocation", allocation_row(&trace));
}
fn allocation_row(trace: &NodeTrace) -> String {
    format!(
        "allocation {} {} {} {}",
        trace.events(),
        u8::from(trace.value_live.get()),
        u8::from(trace.key_live.get()),
        u8::from(trace.live())
    )
}
#[test]
fn native_node_deletion_preserves_successor_and_releases_allocator() {
    let trace = NodeTrace::new();
    let mut tail = MapLinkNode::new(std::ptr::null_mut());
    let next = std::ptr::from_mut(&mut tail);
    let owner = DoubleMapNode::allocate_with(
        &trace.arena,
        next,
        next,
        || trace.item('K'),
        || trace.item('V'),
    );
    let (node, mut arena) = DoubleMapNode::into_raw(owner);
    assert_eq!(arena, trace.arena);
    unsafe { DoubleMapNode::<NodeItem, NodeItem>::del_node(node, &mut arena) };
    native_node_row("deletion", deletion_row(&trace, &tail));
}
fn deletion_row(trace: &NodeTrace, tail: &MapLinkNode) -> String {
    format!(
        "deletion {} {} {}",
        trace.events(),
        u8::from(trace.live()),
        u8::from(tail.next().is_null())
    )
}
fn failed_node_copy(kind: char, prefix: &str) {
    let trace = NodeTrace::new();
    let key = trace.item('K');
    let value = trace.item('V');
    trace.reject_clone.set(kind);
    assert!(node_caught(|| {
        DoubleMapNode::allocate_ref(
            &trace.arena,
            &key,
            &value,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
    }));
    native_node_row(
        prefix,
        format!("{prefix} {} {}", trace.events(), u8::from(trace.live())),
    );
}
#[test]
fn native_key_copy_failure_drops_value_and_frees_node() {
    failed_node_copy('K', "key_failure");
}
#[test]
fn native_value_copy_failure_frees_uninitialized_node() {
    failed_node_copy('V', "value_failure");
}
#[test]
fn rust_key_drop_panic_still_drops_value_and_frees_allocation() {
    let trace = NodeTrace::new();
    let node = borrowed_node(&trace);
    trace.clear();
    trace.reject_drop.set('K');
    assert!(node_caught(|| drop(node)));
    assert_eq!(trace.events(), "DK,DV,");
    assert!(!trace.live());
}
#[test]
fn rust_value_drop_panic_still_frees_allocation_after_key() {
    let trace = NodeTrace::new();
    let node = borrowed_node(&trace);
    trace.clear();
    trace.reject_drop.set('V');
    assert!(node_caught(|| drop(node)));
    assert_eq!(trace.events(), "DK,DV,");
    assert!(!trace.live());
}
#[test]
fn rust_value_factory_panic_frees_preallocated_node() {
    let trace = NodeTrace::new();
    assert!(node_caught(|| {
        DoubleMapNode::<NodeItem, NodeItem>::allocate_with(
            &trace.arena,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            || trace.item('K'),
            || {
                assert!(trace.live());
                panic!("value factory rejected")
            },
        );
    }));
    assert_eq!(trace.events(), "");
    assert!(!trace.live());
}
#[test]
fn rust_node_owner_mutation_and_raw_callback_preserve_address() {
    let arena = rustycad_tkernel::ncollection::BaseAllocator::common();
    let mut owner = DoubleMapNode::allocate_with(
        &arena,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        || 1,
        || 7,
    );
    let pointer = std::ptr::from_ref(&*owner);
    *owner.change_value() = 8;
    assert_eq!((owner.key1(), owner.value()), (&1, &8));
    let (raw, mut allocator) = DoubleMapNode::into_raw(owner);
    assert_eq!(raw.cast_const().cast(), pointer);
    unsafe { DoubleMapNode::<i32, i32>::del_node(raw, &mut allocator) };
}
#[test]
fn rust_native_deletion_callback_frees_after_key_drop_panic() {
    let trace = NodeTrace::new();
    let owner = borrowed_node(&trace);
    let (node, mut allocator) = DoubleMapNode::into_raw(owner);
    trace.clear();
    trace.reject_drop.set('K');
    assert!(node_caught(|| unsafe {
        DoubleMapNode::<NodeItem, NodeItem>::del_node(node, &mut allocator)
    }));
    assert_eq!(trace.events(), "DK,DV,");
    assert!(!trace.live());
}
