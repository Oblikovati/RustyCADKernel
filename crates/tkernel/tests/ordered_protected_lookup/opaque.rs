//! Protected bridges retain native key/value/policy bounds without formatting or copying.
use rustycad_tkernel::ncollection::{HashPolicy, OrderedDataMap, OrderedMap};
struct Opaque(i32);
struct OpaquePolicy;
impl HashPolicy<Opaque> for OpaquePolicy {
    fn hash(&self, key: &Opaque) -> usize {
        key.0 as usize
    }
    fn equal(&self, left: &Opaque, right: &Opaque) -> bool {
        left.0 == right.0
    }
}
fn set() -> OrderedMap<Opaque, OpaquePolicy> {
    let mut map = OrderedMap::with_hasher(OpaquePolicy, 101);
    map.add(Opaque(4));
    map
}
fn data_map() -> OrderedDataMap<Opaque, Opaque, OpaquePolicy> {
    let mut map = OrderedDataMap::with_hasher(OpaquePolicy, 101);
    map.bind(Opaque(4), Opaque(40));
    map
}
macro_rules! opaque_contract {
    ($test:ident,$build:expr) => {
        #[test]
        fn $test() {
            let map = ($build)();
            let mut node = std::ptr::null_mut();
            let mut hash = 0;
            assert!(map.lookup_node(&Opaque(4), &mut node));
            let first = node;
            assert!(map.lookup_node_with_hash(&Opaque(4), &mut node, &mut hash));
            assert_eq!(node, first);
            assert_eq!(hash, 5);
            assert!(map.keys_equal(&Opaque(4), &Opaque(4)));
            assert_eq!(map.bounded_hash(&Opaque(4), 3), 2);
        }
    };
}
opaque_contract!(set_lookup_needs_no_debug_clone_or_default, set);
opaque_contract!(map_lookup_needs_no_debug_clone_or_default, data_map);
