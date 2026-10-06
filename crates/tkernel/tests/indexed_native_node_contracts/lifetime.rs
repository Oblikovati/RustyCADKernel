use super::{
    fixture::{NodeItem, NodeTrace, node_caught},
    native_row,
};
use rustycad_tkernel::ncollection::{
    IndexedDataMapNode, IndexedDataMapNodeOwner, IndexedMapNode, MapLinkNode,
};
#[test]
fn native_allocation_precedes_indexed_key_copy() {
    let trace = NodeTrace::new();
    let key = trace.item('K');
    let _node = IndexedMapNode::allocate_ref(&trace.arena, &key, -7, std::ptr::null_mut());
    native_row(
        "map_allocate",
        format!(
            "map_allocate {} {} {}",
            trace.events(),
            u8::from(trace.key_live.get()),
            u8::from(trace.live())
        ),
    );
}
#[test]
fn native_allocation_precedes_indexed_value_then_key_copy() {
    let trace = NodeTrace::new();
    let (key, value) = (trace.item('K'), trace.item('V'));
    let _node =
        IndexedDataMapNode::allocate_ref(&trace.arena, &key, -7, &value, std::ptr::null_mut());
    native_row(
        "allocation",
        format!(
            "allocation {} {} {} {}",
            trace.events(),
            u8::from(trace.value_live.get()),
            u8::from(trace.key_live.get()),
            u8::from(trace.live())
        ),
    );
}
#[test]
fn native_indexed_key_deleter_frees_only_the_owned_node() {
    let trace = NodeTrace::new();
    let mut tail = MapLinkNode::new(std::ptr::null_mut());
    let owner =
        IndexedMapNode::allocate_with(&trace.arena, -7, std::ptr::from_mut(&mut tail), || {
            trace.item('K')
        });
    let (node, mut arena) = IndexedMapNode::into_raw(owner);
    assert_eq!(arena, trace.arena);
    unsafe { IndexedMapNode::<NodeItem>::del_node(node, &mut arena) };
    assert_deletion_trace("map_delete", &trace, &tail);
}
#[test]
fn native_indexed_data_deleter_drops_key_before_value_and_preserves_successor() {
    let trace = NodeTrace::new();
    let mut tail = MapLinkNode::new(std::ptr::null_mut());
    let owner = allocated_data_node(&trace, std::ptr::from_mut(&mut tail));
    let (node, mut arena) = IndexedDataMapNode::into_raw(owner);
    assert_eq!(arena, trace.arena);
    unsafe { IndexedDataMapNode::<NodeItem, NodeItem>::del_node(node, &mut arena) };
    assert_deletion_trace("deletion", &trace, &tail);
}
#[test]
fn native_indexed_key_copy_failure_frees_the_uninitialized_node() {
    let trace = NodeTrace::new();
    let key = trace.item('K');
    trace.reject_clone.set('K');
    assert!(node_caught(|| {
        IndexedMapNode::allocate_ref(&trace.arena, &key, -7, std::ptr::null_mut());
    }));
    native_row(
        "map_copy_failure",
        format!(
            "map_copy_failure {} {}",
            trace.events(),
            u8::from(trace.live())
        ),
    );
}
fn data_copy_failure(kind: char, prefix: &str) {
    let trace = NodeTrace::new();
    let (key, value) = (trace.item('K'), trace.item('V'));
    trace.reject_clone.set(kind);
    assert!(node_caught(|| {
        IndexedDataMapNode::allocate_ref(&trace.arena, &key, -7, &value, std::ptr::null_mut());
    }));
    native_row(
        prefix,
        format!("{prefix} {} {}", trace.events(), u8::from(trace.live())),
    );
}
#[test]
fn native_indexed_data_copy_failures_preserve_partial_construction_cleanup() {
    data_copy_failure('K', "key_failure");
    data_copy_failure('V', "value_failure");
}
#[test]
fn indexed_data_owner_and_callback_release_allocation_when_key_or_value_drop_panics() {
    for kind in ['K', 'V'] {
        for callback in [false, true] {
            drop_panicking_node(kind, callback);
        }
    }
}
fn drop_panicking_node(kind: char, callback: bool) {
    let trace = NodeTrace::new();
    let owner = allocated_data_node(&trace, std::ptr::null_mut());
    trace.reject_drop.set(kind);
    assert!(node_caught(|| {
        if callback {
            let (node, mut arena) = IndexedDataMapNode::into_raw(owner);
            unsafe { IndexedDataMapNode::<NodeItem, NodeItem>::del_node(node, &mut arena) };
        } else {
            drop(owner);
        }
    }));
    assert_eq!(trace.events(), "DK,DV,");
    assert!(!trace.live());
}
#[test]
fn indexed_key_owner_and_callback_release_allocation_after_drop_panics() {
    for callback in [false, true] {
        drop_panicking_key(callback);
    }
}
fn drop_panicking_key(callback: bool) {
    let trace = NodeTrace::new();
    let owner =
        IndexedMapNode::allocate_with(&trace.arena, -7, std::ptr::null_mut(), || trace.item('K'));
    trace.reject_drop.set('K');
    assert!(node_caught(|| {
        if callback {
            let (node, mut arena) = IndexedMapNode::into_raw(owner);
            unsafe { IndexedMapNode::<NodeItem>::del_node(node, &mut arena) };
        } else {
            drop(owner);
        }
    }));
    assert_eq!(trace.events(), "DK,");
    assert!(!trace.live());
}
fn allocated_data_node(
    trace: &NodeTrace,
    next: *mut MapLinkNode,
) -> IndexedDataMapNodeOwner<NodeItem, NodeItem> {
    IndexedDataMapNode::allocate_with(
        &trace.arena,
        -7,
        next,
        || trace.item('K'),
        || trace.item('V'),
    )
}
fn assert_deletion_trace(prefix: &str, trace: &NodeTrace, tail: &MapLinkNode) {
    native_row(
        prefix,
        format!(
            "{prefix} {} {} {}",
            trace.events(),
            u8::from(trace.live()),
            u8::from(tail.next().is_null())
        ),
    );
}
