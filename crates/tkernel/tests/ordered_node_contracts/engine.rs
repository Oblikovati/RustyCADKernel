use super::fixture::native_node_row;
use rustycad_tkernel::ncollection::{
    BaseAllocator, BaseMapBasicIterator, BasicMapNode, BasicMapNodeMut, OrderedDataMapNode,
    OrderedDataMapNodeOwner, OrderedMapNode, OrderedMapNodeOwner,
};
macro_rules! assert_not {
    ($ty:ty,$bound:path) => {{
        trait Ambiguous<A> {}
        impl<T: ?Sized> Ambiguous<()> for T {}
        struct Forbidden;
        impl<T: ?Sized + $bound> Ambiguous<Forbidden> for T {}
        fn check<T: ?Sized + Ambiguous<A>, A>() {}
        check::<$ty, _>();
    }};
}
type SetNode = OrderedMapNode<i32>;
type DataNode = OrderedDataMapNode<i32, i32>;
fn set_nodes() -> [SetNode; 3] {
    [1, 2, 9].map(|key| SetNode::new(key, std::ptr::null_mut()))
}
fn data_nodes() -> [DataNode; 3] {
    [1, 2, 9].map(|key| DataNode::new(key, key * 10, std::ptr::null_mut()))
}
unsafe fn wire_set(head: *mut SetNode, tail: *mut SetNode, bucket: *mut SetNode) {
    unsafe {
        *(*head).link_mut().next_mut() = bucket.cast();
        *(*head).order_next_mut() = tail;
        *(*tail).order_previous_mut() = head;
    }
}
unsafe fn wire_data(head: *mut DataNode, tail: *mut DataNode, bucket: *mut DataNode) {
    unsafe {
        *(*head).link_mut().next_mut() = bucket.cast();
        *(*head).order_next_mut() = tail;
        *(*tail).order_previous_mut() = head;
    }
}
#[test]
fn set_generic_iterator_uses_order_successor_instead_of_bucket_chain() {
    let [mut head, mut tail, mut bucket] = set_nodes();
    let (head, tail, bucket) = (&raw mut head, &raw mut tail, &raw mut bucket);
    unsafe { wire_set(head, tail, bucket) };
    let mut cursor = BaseMapBasicIterator::<_, false, true, true>::from_node(head);
    let first = unsafe { *cursor.value() };
    unsafe { cursor.advance() };
    unsafe { assert_set_links(head, tail, bucket) };
    native_node_row(
        "set_links",
        format!("set_links 1 1 1 {first} {}", unsafe { cursor.value() }),
    );
}
unsafe fn assert_set_links(head: *mut SetNode, tail: *mut SetNode, bucket: *mut SetNode) {
    unsafe {
        assert_eq!((*head).order_next(), tail);
        assert_eq!((*head).order_next_const(), tail);
        assert_eq!((*tail).order_previous(), head);
        assert_eq!((*head).bucket_next(), bucket);
        assert_eq!((*head).map_key(), &1);
        assert_eq!((*head).map_value(), &1);
    }
}
#[test]
fn data_generic_iterator_mutates_value_and_uses_order_successor() {
    let [mut head, mut tail, mut bucket] = data_nodes();
    let (head, tail, bucket) = (&raw mut head, &raw mut tail, &raw mut bucket);
    unsafe { wire_data(head, tail, bucket) };
    let mut cursor = BaseMapBasicIterator::<_, true, false, true>::from_node(head);
    unsafe { *cursor.change_value() = 11 };
    let first = unsafe { *cursor.value() };
    unsafe { cursor.advance() };
    unsafe { assert_data_links(head, tail, bucket) };
    native_node_row(
        "data_links",
        format!("data_links 1 1 1 {first} {}", unsafe { cursor.value() }),
    );
}
unsafe fn assert_data_links(head: *mut DataNode, tail: *mut DataNode, bucket: *mut DataNode) {
    unsafe {
        assert_eq!((*head).order_next(), tail);
        assert_eq!((*head).order_next_const(), tail);
        assert_eq!((*tail).order_previous(), head);
        assert_eq!((*head).bucket_next(), bucket);
        assert_eq!((*head).map_key(), &1);
        assert_eq!((*head).map_value(), &11);
    }
}
fn shorten_set<'a>(node: OrderedMapNode<&'static str>, _: &'a ()) -> OrderedMapNode<&'a str> {
    node
}
fn shorten_data<'a>(
    node: OrderedDataMapNode<&'static str, &'static str>,
    _: &'a (),
) -> OrderedDataMapNode<&'a str, &'a str> {
    node
}
fn shorten_owner<'a>(
    node: OrderedDataMapNodeOwner<&'static str, &'static str>,
    _: &'a (),
) -> OrderedDataMapNodeOwner<&'a str, &'a str> {
    node
}
fn reference_owner() -> OrderedDataMapNodeOwner<&'static str, &'static str> {
    OrderedDataMapNode::allocate_with(
        &BaseAllocator::common(),
        std::ptr::null_mut(),
        || "key",
        || "value",
    )
}
#[test]
fn payload_lifetimes_are_covariant_and_heap_ownership_stays_unpin() {
    let marker = ();
    let set = shorten_set(OrderedMapNode::new("key", std::ptr::null_mut()), &marker);
    let map = shorten_data(
        OrderedDataMapNode::new("key", "value", std::ptr::null_mut()),
        &marker,
    );
    let owner = shorten_owner(reference_owner(), &marker);
    assert_eq!(
        (*set.key(), *map.value(), *owner.key()),
        ("key", "value", "key")
    );
    fn unpin<T: Unpin>() {}
    unpin::<OrderedMapNodeOwner<std::marker::PhantomPinned>>();
    unpin::<OrderedDataMapNodeOwner<std::marker::PhantomPinned, std::marker::PhantomPinned>>();
}
fn shorten_set_owner<'a>(
    node: OrderedMapNodeOwner<&'static str>,
    _: &'a (),
) -> OrderedMapNodeOwner<&'a str> {
    node
}
#[test]
fn set_owned_payload_lifetime_is_covariant() {
    let marker = ();
    let owner =
        OrderedMapNode::allocate_with(&BaseAllocator::common(), std::ptr::null_mut(), || "key");
    assert_eq!(*shorten_set_owner(owner, &marker).key(), "key");
}
#[test]
fn nodes_keep_native_raw_link_thread_restrictions() {
    assert_not!(OrderedMapNode<i32>, Send);
    assert_not!(OrderedMapNode<i32>, Sync);
    assert_not!(OrderedDataMapNode<i32,i32>,Send);
    assert_not!(OrderedDataMapNode<i32,i32>,Sync);
    assert_not!(OrderedMapNodeOwner<i32>, Send);
    assert_not!(OrderedDataMapNodeOwner<i32,i32>,Sync);
    assert_not!(OrderedMapNode<i32>, BasicMapNodeMut);
    assert_not!(OrderedMapNode<std::marker::PhantomPinned>, Unpin);
}
