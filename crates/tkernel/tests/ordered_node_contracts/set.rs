use super::fixture::{NodeItem, NodeTrace, native_node_row, node_caught};
use rustycad_tkernel::ncollection::OrderedMapNode;
#[test]
fn set_borrowed_constructor_copies_one_key() {
    let trace = NodeTrace::new();
    let key = trace.item('K');
    drop(OrderedMapNode::from_ref(&key, std::ptr::null_mut()));
    native_node_row("set_borrowed", format!("set_borrowed {}", trace.events()));
}
#[test]
fn set_owned_constructor_and_emplacement_move_without_copy() {
    let trace = NodeTrace::new();
    drop(OrderedMapNode::new(trace.item('K'), std::ptr::null_mut()));
    native_node_row("set_owned", format!("set_owned {}", trace.events()));
    trace.clear();
    drop(OrderedMapNode::construct(std::ptr::null_mut(), || {
        trace.item('K')
    }));
    native_node_row("set_emplaced", format!("set_emplaced {}", trace.events()));
}
#[test]
fn set_allocation_precedes_copy_and_native_deleter_frees_only_node() {
    let trace = NodeTrace::new();
    let key = trace.item('K');
    let owner = OrderedMapNode::allocate_ref(&trace.arena, &key, std::ptr::null_mut());
    native_node_row(
        "set_allocation",
        format!(
            "set_allocation {} {}",
            trace.events(),
            u8::from(trace.key_live.get())
        ),
    );
    trace.clear();
    let (node, mut arena) = OrderedMapNode::into_raw(owner);
    unsafe { OrderedMapNode::<NodeItem>::del_node(node, &mut arena) };
    native_node_row(
        "set_deletion",
        format!("set_deletion {} {}", trace.events(), u8::from(trace.live())),
    );
}
#[test]
fn set_rejected_copy_drops_no_uninitialized_key_and_releases_node() {
    let trace = NodeTrace::new();
    let key = trace.item('K');
    trace.reject_clone.set('K');
    assert!(node_caught(|| {
        OrderedMapNode::allocate_ref(&trace.arena, &key, std::ptr::null_mut());
    }));
    native_node_row(
        "set_failure",
        format!("set_failure {} {}", trace.events(), u8::from(trace.live())),
    );
}
#[test]
fn set_destructor_panic_still_releases_allocation() {
    let trace = NodeTrace::new();
    let owner =
        OrderedMapNode::allocate_with(&trace.arena, std::ptr::null_mut(), || trace.item('K'));
    trace.reject_drop.set('K');
    assert!(node_caught(|| drop(owner)));
    assert_eq!(trace.events(), "DK,");
    assert!(!trace.live());
}
#[test]
fn set_factory_failure_releases_uninitialized_allocation() {
    let trace = NodeTrace::new();
    assert!(node_caught(|| {
        OrderedMapNode::<NodeItem>::allocate_with(&trace.arena, std::ptr::null_mut(), || {
            assert!(trace.live());
            panic!("rejected K; expected successful construction");
        });
    }));
    assert!(!trace.live());
    assert_eq!(trace.events(), "");
}

#[test]
fn set_value_projection_and_owner_mutation_retain_native_key_identity() {
    let trace = NodeTrace::new();
    let mut node = OrderedMapNode::allocate_with(&trace.arena, std::ptr::null_mut(), || 7);
    assert!(std::ptr::eq(node.key(), node.value()));
    assert!(node.order_previous().is_null());
    assert!(node.order_next().is_null());
    assert!(node.order_next_const().is_null());
    *node.change_value() = 8;
    assert_eq!(node.key(), &8);
    drop(node);
    assert!(!trace.live());
}

#[test]
fn linked_node_deletion_never_owns_hash_or_order_successors() {
    let trace = NodeTrace::new();
    let first =
        OrderedMapNode::allocate_with(&trace.arena, std::ptr::null_mut(), || trace.item('A'));
    let (first, mut arena) = OrderedMapNode::into_raw(first);
    let second = OrderedMapNode::allocate_with(&trace.arena, first, || trace.item('B'));
    let (second, _) = OrderedMapNode::into_raw(second);
    unsafe {
        *(*first.cast::<OrderedMapNode<NodeItem>>()).order_next_mut() = second.cast();
        *(*second.cast::<OrderedMapNode<NodeItem>>()).order_previous_mut() = first.cast();
        OrderedMapNode::<NodeItem>::del_node(first, &mut arena);
    }
    let prefix = format!("nonowning {} {}", trace.events(), u8::from(trace.live()));
    unsafe { OrderedMapNode::<NodeItem>::del_node(second, &mut arena) };
    native_node_row(
        "nonowning",
        format!("{prefix} {} {}", trace.events(), u8::from(trace.live())),
    );
}
