use super::fixture::native_node_row;
use rustycad_tkernel::ncollection::{
    BaseDoubleMapStorage, BaseMap, BaseMapBasicIterator, BasicMapIteratorSource, BasicMapNode,
    BasicMapNodeMut, DoubleMapNode, MapLinkNode,
};
macro_rules! assert_not {
    ($ty:ty, $bound:path) => {{
        trait Ambiguous<A> {}
        impl<T: ?Sized> Ambiguous<()> for T {}
        struct Forbidden;
        impl<T: ?Sized + $bound> Ambiguous<Forbidden> for T {}
        fn check<T: ?Sized + Ambiguous<A>, A>() {}
        check::<$ty, _>();
    }};
}
struct DoubleNodeBuckets(BaseDoubleMapStorage);
impl DoubleNodeBuckets {
    fn new() -> Self {
        let mut base = BaseDoubleMapStorage::new(101, None);
        base.end_resize(base.begin_resize(0).unwrap());
        let tail = Self::allocate(&base, 2, 20, std::ptr::null_mut());
        let head = Self::allocate(&base, 1, 10, tail);
        // The second hash chain has the reverse order; neither link owns its target.
        unsafe { *(*tail.cast::<DoubleMapNode<i32, i32>>()).next2() = head.cast() };
        base.bucket_slots_mut()[1] = head;
        base.second_slots_mut()[1] = tail;
        base.increment();
        base.increment();
        Self(base)
    }
    fn allocate(
        base: &BaseDoubleMapStorage,
        key: i32,
        value: i32,
        next: *mut MapLinkNode,
    ) -> *mut MapLinkNode {
        let owner = DoubleMapNode::allocate_with(
            base.base().allocator(),
            next,
            std::ptr::null_mut(),
            || key,
            || value,
        );
        let (node, arena) = DoubleMapNode::into_raw(owner);
        assert_eq!(&arena, base.base().allocator());
        node
    }
}
impl BasicMapIteratorSource for DoubleNodeBuckets {
    type Node = DoubleMapNode<i32, i32>;
    fn basic_bucket_slots(&self) -> Option<&[*mut MapLinkNode]> {
        Some(self.0.bucket_slots())
    }
    fn basic_first_node(&self) -> *mut Self::Node {
        std::ptr::null_mut()
    }
}
impl Drop for DoubleNodeBuckets {
    fn drop(&mut self) {
        unsafe {
            self.0.destroy(
                |node, arena| DoubleMapNode::<i32, i32>::del_node(node, arena),
                true,
            )
        };
    }
}
#[test]
fn real_double_bucket_graph_drives_key_only_generic_iteration() {
    let source = DoubleNodeBuckets::new();
    let mut cursor = BaseMapBasicIterator::<DoubleMapNode<i32, i32>, false, true>::new(&source);
    unsafe {
        let row = format!(
            "generic_key {} {} {}",
            cursor.key(),
            cursor.value(),
            u8::from(std::ptr::eq(cursor.key(), cursor.value_ptr()))
        );
        native_node_row("generic_key", row);
        assert_eq!((*cursor.node()).map_key(), &1);
        assert_eq!((*cursor.node()).map_value(), &10);
        assert!((*cursor.node()).order_next().is_null());
        cursor.advance();
        assert_eq!(cursor.key(), &20);
        cursor.advance();
        assert!(!cursor.more());
    }
}
#[test]
fn second_chain_uses_its_own_typed_pointer_and_does_not_drive_iteration() {
    let source = DoubleNodeBuckets::new();
    let second = source.0.second_slots()[1].cast::<DoubleMapNode<i32, i32>>();
    unsafe {
        assert_eq!((*second).key1(), &2);
        let head = (*second).next2_value();
        assert_eq!((*head).key1(), &1);
        assert!((*head).next2_value().is_null());
    }
}
#[test]
fn native_mutable_node_fields_keep_independent_link_identity() {
    let mut first = DoubleMapNode::new(2, 20, std::ptr::null_mut(), std::ptr::null_mut());
    let mut second = DoubleMapNode::new(3, 30, std::ptr::null_mut(), std::ptr::null_mut());
    let (first, second) = (
        std::ptr::from_mut(&mut first),
        std::ptr::from_mut(&mut second),
    );
    let mut node = DoubleMapNode::new(1, 10, first.cast(), second.cast());
    assert_eq!(node.link().next(), first.cast());
    assert_eq!(node.next2_value(), second);
    assert!(std::ptr::eq(node.key2(), node.value()));
    *node.change_key1() = 11;
    *node.change_value() = 12;
    *node.next2() = first;
    assert_link_row(&node, first);
}
fn assert_link_row(node: &DoubleMapNode<i32, i32>, first: *mut DoubleMapNode<i32, i32>) {
    native_node_row(
        "links",
        format!(
            "links 1 1 1 {} {} {}",
            node.key1(),
            node.key2(),
            u8::from(node.next2_value() == first)
        ),
    );
}
#[test]
fn double_node_keeps_raw_link_thread_restrictions_and_readonly_generic_value() {
    assert_not!(DoubleMapNode<i32,i32>,Send);
    assert_not!(DoubleMapNode<i32,i32>,Sync);
    assert_not!(DoubleMapNode<i32,i32>,BasicMapNodeMut);
    let node = DoubleMapNode::new((), (), std::ptr::null_mut(), std::ptr::null_mut());
    assert_eq!((node.key1(), node.key2()), (&(), &()));
}
