use rustycad_tkernel::ncollection::{
    BaseMap, BaseMapBasicIterator, BaseMapStorage, BasicMapIteratorSource, BasicMapNode,
    DataMapNode, MapLinkNode,
};
struct RealNodeBuckets(BaseMapStorage);
impl RealNodeBuckets {
    fn new() -> Self {
        let mut base = BaseMapStorage::new(101, None);
        let plan = base.begin_resize(0).unwrap();
        base.end_resize(plan);
        let tail = Self::allocate(&base, 2, 20, std::ptr::null_mut());
        let head = Self::allocate(&base, 1, 10, tail);
        base.bucket_slots_mut()[1] = head;
        base.increment();
        base.increment();
        Self(base)
    }
    fn allocate(
        base: &BaseMapStorage,
        key: i32,
        value: i32,
        next: *mut MapLinkNode,
    ) -> *mut MapLinkNode {
        let owner = DataMapNode::allocate_with(base.allocator(), next, || key, || value);
        let (node, arena) = DataMapNode::into_raw(owner);
        assert_eq!(&arena, base.allocator());
        node
    }
}
impl BasicMapIteratorSource for RealNodeBuckets {
    type Node = DataMapNode<i32, i32>;
    fn basic_bucket_slots(&self) -> Option<&[*mut MapLinkNode]> {
        Some(self.0.bucket_slots())
    }
    fn basic_first_node(&self) -> *mut Self::Node {
        std::ptr::null_mut()
    }
}
impl Drop for RealNodeBuckets {
    fn drop(&mut self) {
        unsafe {
            self.0.destroy(
                |node, arena| DataMapNode::<i32, i32>::del_node(node, arena),
                true,
            )
        };
    }
}
#[test]
fn typed_nodes_drive_the_actual_base_map_bucket_engine() {
    let source = RealNodeBuckets::new();
    let mut cursor = BaseMapBasicIterator::<DataMapNode<i32, i32>>::new(&source);
    assert_eq!(source.0.size(), 2);
    assert_eq!(unsafe { (*cursor.node()).map_key() }, &1);
    assert_eq!(unsafe { (*cursor.node()).map_value() }, &10);
    assert!(unsafe { (*cursor.node()).order_next() }.is_null());
    unsafe { cursor.advance() };
    assert_eq!(unsafe { (*cursor.node()).map_key() }, &2);
    unsafe { cursor.advance() };
    assert!(!cursor.more());
}
#[test]
fn typed_nodes_support_existing_mutable_generic_iterator() {
    let mut source = RealNodeBuckets::new();
    let mut cursor = BaseMapBasicIterator::<DataMapNode<i32, i32>, true>::new(&mut source);
    unsafe {
        *cursor.change_value() = 11;
        cursor.advance();
        *cursor.change_value() = 22;
    }
    let cursor = BaseMapBasicIterator::<DataMapNode<i32, i32>>::new(&source);
    assert_eq!(unsafe { cursor.value() }, &11);
}
#[derive(Clone)]
#[repr(align(128))]
struct AlignedNodeValue(i32);
#[test]
fn typed_node_prefix_survives_overaligned_value_and_zero_sized_key() {
    let arena = rustycad_tkernel::ncollection::BaseAllocator::common();
    let owner = DataMapNode::allocate_ref(&arena, &(), &AlignedNodeValue(7), std::ptr::null_mut());
    assert_eq!(std::ptr::from_ref(owner.value()).addr() % 128, 0);
    assert_eq!(owner.value().0, 7);
    let (node, mut arena) = DataMapNode::into_raw(owner);
    assert!(unsafe { (*node).next() }.is_null());
    unsafe { DataMapNode::<(), AlignedNodeValue>::del_node(node, &mut arena) };
}
