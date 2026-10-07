//! White-box checks of the order links, chain successors and rehash order against the native
//! rows. The probe's node layout, reservation and arena rows describe allocator-owned nodes
//! (plan D1 classes B5 and B6) and are not compared.
use super::*;
use std::{cell::RefCell, rc::Rc};
#[derive(Clone, Default)]
struct CanonicalHash(Rc<RefCell<String>>);
impl HashPolicy<i32> for CanonicalHash {
    fn hash(&self, key: &i32) -> usize {
        self.0.borrow_mut().push_str(&format!("{key},"));
        (*key as usize) % 2
    }
    fn equal(&self, left: &i32, right: &i32) -> bool {
        left == right
    }
}
fn native_row(label: &str, observed: String) {
    let expected = include_str!("../../../../../tools/oracle/tkernel_ordered_canonical.expected");
    assert_eq!(
        observed,
        expected
            .lines()
            .find(|line| line.starts_with(&format!("{label} ")))
            .unwrap()
    );
}
// Each entry in insertion order with the key of its bucket-chain successor (0 at a chain end);
// the trailing 1 is the native check that every order back link names the previous entry.
fn graph<V>(label: &str, map: &OrderedTable<i32, V, CanonicalHash>) {
    let mut observed = format!("{label} ");
    let mut previous = None;
    let mut link = map.first();
    while let Some(slot) = link {
        assert_eq!(map.order[slot].previous, previous);
        let next = map
            .storage
            .chains()
            .next(slot)
            .map_or(0, |next| *map.key(next));
        observed.push_str(&format!("{}:{next},", map.key(slot)));
        previous = Some(slot);
        link = map.next_slot(slot);
    }
    observed.push_str(" 1");
    native_row(label, observed);
}
fn exercise<V: Default>(prefix: &str, map: &mut OrderedTable<i32, V, CanonicalHash>) {
    for key in [4, 1, 2, 3, 6, 5] {
        map.place(key, V::default, false, false);
    }
    assert!(map.remove(&1));
    map.place(9, V::default, false, false);
    graph(&format!("{prefix}_before"), map);
    let first = map.serial(map.first().unwrap());
    map.hasher.0.borrow_mut().clear();
    map.resize(200);
    let same = map.first().map(|slot| map.serial(slot)) == Some(first);
    let rehash = format!(
        "{prefix}_rehash {} {}",
        map.hasher.0.borrow(),
        u8::from(same)
    );
    native_row(&format!("{prefix}_rehash"), rehash);
    graph(&format!("{prefix}_after"), map);
    map.clear(false);
    // The native row counts live arena allocations after Clear(false): none, as no entry remains.
    let clear = format!("{prefix}_clear {} {}", map.len(), map.bucket_count());
    native_row(&format!("{prefix}_clear"), clear);
    assert!(map.buckets().is_some());
}
#[test]
fn native_set_links_chains_and_rehash() {
    let arena = BaseAllocator::accumulating(1024).unwrap();
    let mut map = OrderedTable::<i32, (), _>::new(CanonicalHash::default(), 101, arena);
    exercise("set", &mut map);
}
#[test]
fn native_data_links_chains_and_rehash() {
    let arena = BaseAllocator::accumulating(1024).unwrap();
    let mut map = OrderedTable::<i32, i32, _>::new(CanonicalHash::default(), 101, arena);
    exercise("data", &mut map);
}
#[test]
fn removed_identities_are_never_reused_and_slots_move_with_their_entries() {
    let mut map = OrderedTable::<i32, i32, _>::new(
        CanonicalHash::default(),
        101,
        BaseAllocator::incremental(512),
    );
    let (_, first) = map.place(1, || 10, false, false);
    let removed = map.serial(first);
    map.place(2, || 20, false, false);
    map.place(3, || 30, false, false);
    assert!(map.remove(&1));
    let (_, last) = map.place(4, || 40, false, false);
    assert_eq!(map.slot_of(removed), None);
    assert_eq!(
        map.items().map(|(key, _)| *key).collect::<Vec<_>>(),
        [2, 3, 4]
    );
    assert_eq!(map.slot_of(map.serial(last)), Some(last));
    map.clear(true);
    assert!(map.buckets().is_none());
}
