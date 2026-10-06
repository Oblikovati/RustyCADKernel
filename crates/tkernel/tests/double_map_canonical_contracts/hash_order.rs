use rustycad_tkernel::ncollection::{DoubleMap, HashPolicy};
use std::{cell::RefCell, rc::Rc};
#[derive(Clone)]
pub struct OrderedHash<const SIDE: usize>(pub Rc<RefCell<String>>);
impl<const SIDE: usize> HashPolicy<i32> for OrderedHash<SIDE> {
    fn hash(&self, key: &i32) -> usize {
        self.0.borrow_mut().push_str(&format!("H{SIDE}:{key},"));
        if SIDE == 1 { *key as usize } else { 0 }
    }
    fn equal(&self, first: &i32, second: &i32) -> bool {
        self.0
            .borrow_mut()
            .push_str(&format!("E{SIDE}:{first}:{second},"));
        first == second
    }
}
pub type OrderedMap = DoubleMap<i32, i32, OrderedHash<1>, OrderedHash<2>>;
pub fn ordered_map() -> (OrderedMap, Rc<RefCell<String>>) {
    let events = Rc::new(RefCell::new(String::new()));
    let map = DoubleMap::with_allocator(
        OrderedHash(events.clone()),
        OrderedHash(events.clone()),
        1,
        Default::default(),
    );
    (map, events)
}
fn assert_trace(label: &str, events: &RefCell<String>) {
    crate::native_row(label, format!("{label} {}", events.borrow()));
    events.borrow_mut().clear();
}
#[test]
fn bind_hashes_once_per_index_and_stops_before_copying_duplicate_keys() {
    let (mut map, events) = ordered_map();
    map.bind(1, 5).unwrap();
    assert_trace("insert_empty", &events);
    map.bind(2, 6).unwrap();
    assert_trace("insert_nonempty", &events);
    assert!(!map.try_bind(1, 99));
    assert_trace("duplicate_first", &events);
    assert!(!map.try_bind(3, 5));
    assert_trace("duplicate_second", &events);
}
#[test]
fn are_bound_preserves_native_wrong_chain_and_hashes_both_before_search() {
    let (mut map, events) = ordered_map();
    map.bind(1, 5).unwrap();
    map.bind(2, 6).unwrap();
    events.borrow_mut().clear();
    assert!(!map.are_bound(&99, &5));
    crate::native_row(
        "are_missing",
        format!("are_missing 0 trace {}", events.borrow()),
    );
    events.borrow_mut().clear();
    assert!(!map.are_bound(&1, &5));
    crate::native_row(
        "are_primary_next_quirk",
        format!("are_primary_next_quirk 0 trace {}", events.borrow()),
    );
}
#[test]
fn resize_and_both_unbind_paths_keep_native_hash_order() {
    let (mut map, events) = ordered_map();
    map.bind(1, 5).unwrap();
    map.bind(2, 6).unwrap();
    events.borrow_mut().clear();
    map.resize(200);
    crate::native_row(
        "resize",
        format!("resize {} trace {}", map.bucket_count(), events.borrow()),
    );
    events.borrow_mut().clear();
    assert!(map.unbind1(&2));
    assert_trace("unbind_first", &events);
    assert!(map.unbind2(&5));
    assert_trace("unbind_second", &events);
}
