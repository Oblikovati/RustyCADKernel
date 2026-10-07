use rustycad_tkernel::ncollection::{
    BaseMap, HashBucketTable, HashPolicy, OrderedDataMap, OrderedMap,
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
/// The two public null checks of the native row: `Buckets()` and `FirstNode()`.
pub trait OrderedProbe: BaseMap {
    fn table_absent(&self) -> bool;
    fn first_absent(&self) -> bool;
}
impl OrderedProbe for Set {
    fn table_absent(&self) -> bool {
        self.buckets().is_none()
    }
    fn first_absent(&self) -> bool {
        self.first_node().is_none()
    }
}
impl OrderedProbe for Map {
    fn table_absent(&self) -> bool {
        self.buckets().is_none()
    }
    fn first_absent(&self) -> bool {
        self.first_node().is_none()
    }
}
pub fn queries<T: OrderedProbe>(label: &str, map: &T) {
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
            u8::from(map.table_absent()),
            u8::from(map.first_absent())
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
/// A native row of `key:value,` pairs, from any traversal.
pub fn record<'a, K: std::fmt::Display + 'a, V: std::fmt::Display + 'a>(
    label: &str,
    pairs: impl Iterator<Item = (&'a K, &'a V)>,
) {
    let observed: String = pairs
        .map(|(key, value)| format!("{key}:{value},"))
        .collect();
    native_row(label, format!("{label} {observed}"));
}
/// Every chain of a bucket table in native order (bucket `0..=NbBuckets`, then each chain).
pub fn bucket_pairs<'a, K, V>(table: &HashBucketTable<'a, K, V>) -> Vec<(&'a K, &'a V)> {
    (0..table.len())
        .flat_map(|bucket| table.chain(bucket).unwrap())
        .collect()
}
