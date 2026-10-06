use super::{PackedInteger, PackedMap, algo};
use std::cell::RefCell;
thread_local! {
    static CALLS: RefCell<Option<Vec<u64>>> = const { RefCell::new(None) };
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
struct ObservedKey(u32);
impl PackedInteger for ObservedKey {
    type Word = u32;
    const IS_64_BIT: bool = false;
    const MIN: Self = Self(0);
    const MAX: Self = Self(u32::MAX);
    fn bits(self) -> u64 {
        observe(u64::from(self.0));
        u64::from(self.0)
    }
    fn from_bits(bits: u64) -> Self {
        observe(bits);
        Self(bits as u32)
    }
}
fn observe(value: u64) {
    CALLS.with_borrow_mut(|calls| {
        if let Some(calls) = calls {
            calls.push(value);
        }
    });
}
struct RestoreCalls;
impl Drop for RestoreCalls {
    fn drop(&mut self) {
        CALLS.with_borrow_mut(|calls| *calls = None);
    }
}
fn recorded(operation: impl FnOnce()) -> Vec<u64> {
    CALLS.with_borrow_mut(|calls| *calls = Some(Vec::new()));
    let _restore: RestoreCalls = RestoreCalls;
    operation();
    CALLS.with_borrow_mut(|calls| calls.take().unwrap())
}
fn map(key: u32) -> PackedMap<ObservedKey> {
    let mut result: PackedMap<ObservedKey> = PackedMap::default();
    result.add(ObservedKey(key));
    result
}
fn self_predicates(map: &PackedMap<ObservedKey>) {
    assert!(algo::contains(map, map));
    assert!(algo::is_subset(map, map));
    assert!(algo::is_equal(map, map));
    assert_eq!(algo::has_intersection(map, map), !map.is_empty());
    assert!(map.contains_map(map));
    assert!(map.is_subset(map));
    assert!(map.is_equal(map));
    assert_eq!(map.has_intersection(map), !map.is_empty());
}
#[test]
fn self_short_circuits_do_not_call_custom_key_conversions() {
    for map in [PackedMap::default(), map(33)] {
        assert_eq!(recorded(|| self_predicates(&map)), []);
    }
}
fn empty_predicates(empty: &PackedMap<ObservedKey>, full: &PackedMap<ObservedKey>) {
    assert!(algo::contains(full, empty));
    assert!(algo::is_subset(empty, full));
    assert!(!algo::has_intersection(full, empty));
    assert!(!algo::has_intersection(empty, full));
    assert!(full.contains_map(empty));
    assert!(empty.is_subset(full));
    assert!(!full.has_intersection(empty));
    assert!(!empty.has_intersection(full));
    assert!(!algo::is_equal(empty, full));
    assert!(!algo::contains(empty, full));
    assert!(!empty.is_equal(full));
    assert!(!empty.contains_map(full));
}
#[test]
fn empty_operand_short_circuits_do_not_call_custom_key_conversions() {
    let empty: PackedMap<ObservedKey> = PackedMap::default();
    let full: PackedMap<ObservedKey> = map(33);
    assert_eq!(recorded(|| empty_predicates(&empty, &full)), []);
}
#[test]
fn synthetic_signed_extent_selects_native_first_operand() {
    let mut left: PackedMap<ObservedKey> = map(33);
    let right: PackedMap<ObservedKey> = map(65);
    left.count = 2_147_483_648;
    let calls: Vec<u64> = recorded(|| assert!(!algo::has_intersection(&left, &right)));
    left.count = 1;
    assert_eq!(&calls[..2], &[0, 32]);
}

fn assert_no_second_block(operation: impl FnOnce()) {
    let calls: Vec<u64> = recorded(operation);
    assert!(calls.contains(&33));
    assert!(!calls.contains(&64), "advanced into next block: {calls:?}");
    assert!(!calls.contains(&65), "converted next key: {calls:?}");
}
#[test]
fn early_predicate_result_precedes_next_cursor_conversion() {
    let mut left: PackedMap<ObservedKey> = map(33);
    left.add(ObservedKey(65));
    let mut superset: PackedMap<ObservedKey> = map(33);
    superset.add(ObservedKey(97));
    superset.add(ObservedKey(129));
    assert_no_second_block(|| assert!(algo::has_intersection(&left, &superset)));
    assert_no_second_block(|| assert!(left.has_intersection(&superset)));
    superset.remove(ObservedKey(33));
    assert_no_second_block(|| assert!(!algo::contains(&superset, &left)));
    assert_no_second_block(|| assert!(!superset.contains_map(&left)));
    assert_no_second_block(|| assert!(!algo::is_subset(&left, &superset)));
    assert_no_second_block(|| assert!(!left.is_subset(&superset)));
    assert_no_second_block(|| assert!(!algo::is_equal(&superset, &left)));
    assert_no_second_block(|| assert!(!superset.is_equal(&left)));
}
