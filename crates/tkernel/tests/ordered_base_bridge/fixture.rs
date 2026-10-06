use rustycad_tkernel::ncollection::{
    BaseMap, BaseMapBasicIterator, BasicMapIteratorSource, HashPolicy, OrderedDataMap,
    OrderedDataMapNode, OrderedMap, OrderedMapNode,
};
#[derive(Default)]
pub struct BridgeHash;
impl HashPolicy<i32> for BridgeHash {
    fn hash(&self, key: &i32) -> usize {
        (*key as usize) % 2
    }
    fn equal(&self, left: &i32, right: &i32) -> bool {
        left == right
    }
}
pub type Set = OrderedMap<i32, BridgeHash>;
pub type Map = OrderedDataMap<i32, i32, BridgeHash>;
pub type SetOrder<'a> = BaseMapBasicIterator<'a, OrderedMapNode<i32>, false, true, true>;
pub type SetBucket<'a> = BaseMapBasicIterator<'a, OrderedMapNode<i32>, false, true, false>;
pub type DataOrder<'a> = BaseMapBasicIterator<'a, OrderedDataMapNode<i32, i32>, false, false, true>;
pub type DataBucket<'a> =
    BaseMapBasicIterator<'a, OrderedDataMapNode<i32, i32>, false, false, false>;
pub type DataMutable<'a> =
    BaseMapBasicIterator<'a, OrderedDataMapNode<i32, i32>, true, false, true>;
pub fn native_row(label: &str, observed: String) {
    let expected = include_str!("../../../../tools/oracle/tkernel_ordered_base_bridge.expected");
    assert_eq!(
        observed,
        expected
            .lines()
            .find(|line| line.starts_with(&format!("{label} ")))
            .unwrap()
    );
}
pub fn queries<T: BaseMap + BasicMapIteratorSource>(label: &str, map: &T) {
    let base: &dyn BaseMap = map;
    native_row(
        label,
        format!(
            "{label} {} {} {} {} {} {} {}",
            base.nb_buckets(),
            base.extent(),
            base.length(),
            base.size(),
            u8::from(base.is_empty()),
            u8::from(map.basic_bucket_slots().is_none()),
            u8::from(map.basic_first_node().is_null())
        ),
    );
}
pub fn fill_set(map: &mut Set) {
    for key in [4, 1, 2, 3, 6, 5] {
        map.add(key);
    }
}
pub fn fill_map(map: &mut Map) {
    for key in [4, 1, 2, 3, 6, 5] {
        map.bind(key, key * 10);
    }
}
macro_rules! record_keys {
    ($name:ident,$cursor:ident) => {
        pub fn $name(label: &str, mut cursor: $cursor<'_>) {
            let mut observed = format!("{label} ");
            while cursor.more() {
                // SAFETY: the fixture's map borrow retains every current node; reads never escape.
                unsafe {
                    observed.push_str(&format!("{}:{},", cursor.key(), cursor.value()));
                    cursor.advance();
                }
            }
            native_row(label, observed);
        }
    };
}
record_keys!(set_order, SetOrder);
record_keys!(set_buckets, SetBucket);
record_keys!(data_order, DataOrder);
record_keys!(data_buckets, DataBucket);
