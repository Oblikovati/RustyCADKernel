use super::fixture::{Collision, native_row};
use rustycad_tkernel::ncollection::{
    BaseMap, BaseMapBasicIterator, BaseMapStorage, BasicMapIteratorSource, BasicMapNode, DataMap,
    MapLinkNode, MapNode,
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
// The native generic BasicIterator rows, read through the public DataMap::Iterator (plan D1 B6).
fn mutate_through_public_cursor(map: &mut DataMap<i32, i32, Collision>) -> (String, *const i32) {
    let mut cursor = map.cursor_mut();
    *cursor.change_value().unwrap() = 21;
    let old = cursor.advance_post();
    let (first_key, first_value) = (*old.key().unwrap(), *old.value().unwrap());
    let old_value = std::ptr::from_ref(old.value().unwrap());
    *cursor.change_value().unwrap() = 11;
    let (last_key, last_value) = (*cursor.key().unwrap(), *cursor.value().unwrap());
    let row = format!("data_iterator {first_key} {first_value} {last_key} {last_value}");
    (row, old_value)
}
#[test]
fn public_data_map_cursor_mutates_values_in_native_chain_order() {
    let mut map = populated_data_map();
    let (row, old_value) = mutate_through_public_cursor(&mut map);
    let arrow = old_value == std::ptr::from_mut(map.change_find(&2).unwrap());
    native_row("data_iterator", format!("{row} {}", u8::from(arrow)));
    assert_const_iterator(&map, old_value);
    assert_eq!(map.find(&1), Ok(&11));
    assert_eq!(map.find(&2), Ok(&21));
}
fn assert_const_iterator(map: &DataMap<i32, i32, Collision>, old_value: *const i32) {
    let cursor = map.cursor();
    let same = std::ptr::eq(cursor.value().unwrap(), old_value);
    native_row(
        "const_iterator",
        format!(
            "const_iterator {} {} {}",
            cursor.key().unwrap(),
            cursor.value().unwrap(),
            u8::from(same)
        ),
    );
}
#[test]
fn bucket_table_is_unallocated_retained_and_released_like_native() {
    let mut map = DataMap::<i32, i32>::default();
    assert!(map.buckets().is_none());
    map.bind(1, 10);
    let table = map.buckets().unwrap().identity();
    map.clear(false);
    assert_eq!(map.buckets().map(|kept| kept.identity()), Some(table));
    assert!(!map.cursor().more());
    map.clear(true);
    assert!(map.buckets().is_none());
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
