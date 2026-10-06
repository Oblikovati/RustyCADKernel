use super::fixture::{Collision, native_row};
use rustycad_tkernel::ncollection::{
    BaseMap, BaseMapBasicIterator, BaseMapStorage, BasicMapIteratorSource, BasicMapNode, DataMap,
    DataMapNode, MapLinkNode, MapNode,
};
struct NativeKeyBuckets(BaseMapStorage);
impl NativeKeyBuckets {
    fn new() -> Self {
        let mut base = BaseMapStorage::new(101, None);
        base.end_resize(base.begin_resize(0).unwrap());
        let mut head = std::ptr::null_mut();
        for key in 1..=2 {
            let owner = MapNode::allocate_with(base.allocator(), head, || key);
            (head, _) = MapNode::into_raw(owner);
            base.increment();
        }
        base.bucket_slots_mut()[1] = head;
        Self(base)
    }
}
impl BasicMapIteratorSource for NativeKeyBuckets {
    type Node = MapNode<i32>;
    fn basic_bucket_slots(&self) -> Option<&[*mut MapLinkNode]> {
        Some(self.0.bucket_slots())
    }
    fn basic_first_node(&self) -> *mut Self::Node {
        std::ptr::null_mut()
    }
}
impl Drop for NativeKeyBuckets {
    fn drop(&mut self) {
        unsafe {
            self.0
                .destroy(|node, arena| MapNode::<i32>::del_node(node, arena), true)
        };
    }
}
#[test]
fn key_only_generic_iterator_reads_actual_map_node_values() {
    let mut source = NativeKeyBuckets::new();
    let mut cursor = BaseMapBasicIterator::<MapNode<i32>, true, true>::new(&mut source);
    let old = unsafe { cursor.advance_post() };
    let (first, key, next) = unsafe { (*old.value(), *old.key(), *cursor.value()) };
    let arrow = unsafe { old.value_ptr() == std::ptr::from_ref(old.value()) };
    unsafe { cursor.advance() };
    native_row(
        "key_iterator",
        format!(
            "key_iterator {first} {key} {next} {} {}",
            u8::from(arrow),
            u8::from(!cursor.more())
        ),
    );
}
#[test]
fn unordered_key_node_projection_uses_one_key_object() {
    let source = NativeKeyBuckets::new();
    let cursor = BaseMapBasicIterator::<MapNode<i32>, false, true>::new(&source);
    let node = unsafe { &*cursor.node() };
    assert_eq!(node.map_key(), &2);
    assert_eq!(node.map_value(), &2);
    assert!(!node.bucket_next().is_null());
    assert!(node.order_next().is_null());
    assert_eq!(source.0.size(), 2);
}
fn populated_data_map() -> DataMap<i32, i32, Collision> {
    let mut map = DataMap::with_hasher(Collision, 1);
    map.bind(1, 10);
    map.bind(2, 20);
    map
}
type NativeDataCursorResult = (String, *const DataMapNode<i32, i32>, *mut i32);
fn mutate_canonical_map(map: &mut DataMap<i32, i32, Collision>) -> NativeDataCursorResult {
    let mut cursor = BaseMapBasicIterator::<DataMapNode<i32, i32>, true>::new(map);
    unsafe { *cursor.value() = 21 };
    let mut old = unsafe { cursor.advance_post() };
    unsafe { *cursor.change_value() = 11 };
    let (first_key, first_value) = unsafe { (*old.key(), *old.value()) };
    let (last_key, last_value) = unsafe { (*cursor.key(), *cursor.value()) };
    let row = format!("data_iterator {first_key} {first_value} {last_key} {last_value}");
    (row, old.node(), unsafe { old.value_ptr() })
}
#[test]
fn generic_data_map_iterator_mutates_sole_canonical_nodes() {
    let mut map = populated_data_map();
    let (row, old_node, old_value) = mutate_canonical_map(&mut map);
    let arrow = old_value == std::ptr::from_mut(map.change_find(&2).unwrap());
    native_row("data_iterator", format!("{row} {}", u8::from(arrow)));
    assert_const_iterator(&map, old_node);
    assert_eq!(map.find(&1), Ok(&11));
    assert_eq!(map.find(&2), Ok(&21));
}
fn assert_const_iterator(
    map: &DataMap<i32, i32, Collision>,
    old_node: *const DataMapNode<i32, i32>,
) {
    let cursor = BaseMapBasicIterator::<DataMapNode<i32, i32>>::new(map);
    native_row("const_iterator", unsafe {
        format!(
            "const_iterator {} {} {}",
            cursor.key(),
            cursor.value(),
            u8::from(cursor.node() == old_node)
        )
    });
}
#[test]
fn source_bridge_reports_actual_table_in_unallocated_retained_and_released_states() {
    let mut map = DataMap::<i32, i32>::default();
    assert!(map.basic_bucket_slots().is_none());
    assert!(map.basic_first_node().is_null());
    map.bind(1, 10);
    assert_eq!(map.basic_bucket_slots().unwrap().as_ptr(), map.buckets());
    map.clear(false);
    assert!(map.basic_bucket_slots().is_some());
    assert!(!BaseMapBasicIterator::<DataMapNode<i32, i32>>::new(&map).more());
    map.clear(true);
    assert!(map.basic_bucket_slots().is_none());
}
#[test]
fn mutable_map_specialization_keeps_keys_readonly() {
    trait Ambiguous<A> {}
    impl<T: ?Sized> Ambiguous<()> for T {}
    struct Forbidden;
    impl<T: rustycad_tkernel::ncollection::BasicMapNodeMut> Ambiguous<Forbidden> for T {}
    fn unique<T: ?Sized + Ambiguous<A>, A>() {}
    unique::<MapNode<i32>, _>();
    let mut source = NativeKeyBuckets::new();
    let cursor = BaseMapBasicIterator::<MapNode<i32>, true, true>::new(&mut source);
    let _: &i32 = unsafe { cursor.value() };
}
