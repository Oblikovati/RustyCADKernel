//! Key destructor order of IndexedMap against the native rows. The probe also printed, inside
//! each destructor, the map metadata it could reach; that is protected native state (plan D1
//! class B6), so each native event keeps only the key and the final state is compared.
use rustycad_tkernel::ncollection::{BaseAllocator, HashPolicy, IndexedMap};
use std::{cell::RefCell, fmt::Write, rc::Rc};
#[derive(Clone)]
struct Key {
    value: i32,
    drops: Rc<RefCell<String>>,
}
impl std::fmt::Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl Drop for Key {
    fn drop(&mut self) {
        write!(self.drops.borrow_mut(), "{};", self.value).unwrap();
    }
}
struct KeyPolicy;
impl HashPolicy<Key> for KeyPolicy {
    fn hash(&self, key: &Key) -> usize {
        key.value as usize % 2
    }
    fn equal(&self, a: &Key, b: &Key) -> bool {
        a.value == b.value
    }
}
fn native_order(label: &str) -> String {
    let row = super::native_row_text(label);
    let fields: Vec<&str> = row.split(' ').collect();
    let drops: String = fields[1]
        .split_terminator(';')
        .map(|event| event.split(',').next().unwrap().to_owned() + ";")
        .collect();
    format!("{label} {drops} {} {}", fields[2], fields[3])
}
fn observed(mode: usize) {
    let drops = Rc::new(RefCell::new(String::new()));
    let mut map = IndexedMap::with_hasher(KeyPolicy, 1);
    let allocator = map.allocator().clone();
    for value in 1..=3 {
        map.add(Key {
            value,
            drops: drops.clone(),
        });
    }
    match mode {
        0 => map.remove_last().unwrap(),
        1 => map.clear(false),
        _ => map.clear_with_allocator(Some(BaseAllocator::fresh_common())),
    }
    let old = u8::from(map.allocator() == &allocator);
    let label = format!("observer{mode}");
    let row = format!("{label} {} {} {old}", drops.borrow(), map.len());
    assert_eq!(row, native_order(&label));
}
#[test]
fn remove_last_drops_the_last_key() {
    observed(0);
}
#[test]
fn clear_drops_keys_in_bucket_order() {
    observed(1);
}
#[test]
fn replacing_allocator_adopts_it_after_the_last_drop() {
    observed(2);
}
#[test]
fn exchange_moves_the_table_with_its_keys() {
    let mut first = IndexedMap::<i32>::default();
    first.add(7);
    let mut second = IndexedMap::<i32>::default();
    second.add(8);
    second.add(9);
    let table = second.buckets().unwrap().identity();
    first.exchange(&mut second);
    assert_eq!((first.len(), second.len()), (2, 1));
    assert_eq!(first.buckets().unwrap().identity(), table);
}
