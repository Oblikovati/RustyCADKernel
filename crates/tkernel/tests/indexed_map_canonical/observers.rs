use rustycad_tkernel::ncollection::{BaseAllocator, HashPolicy, IndexedMap, IndexedMapObserver};
use std::{cell::RefCell, fmt::Write, rc::Rc};
#[derive(Default)]
struct Trace {
    observer: Option<IndexedMapObserver>,
    allocator: usize,
    rows: String,
}
#[derive(Clone)]
struct Key {
    value: i32,
    trace: Rc<RefCell<Trace>>,
}
impl std::fmt::Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl Drop for Key {
    fn drop(&mut self) {
        let mut trace = self.trace.borrow_mut();
        let Some(observer) = &trace.observer else {
            return;
        };
        let state = observer.snapshot();
        let indices = state.indices.iter().filter(|p| **p != 0).count();
        let heads = state.buckets.iter().filter(|p| **p != 0).count();
        let old = u8::from(state.allocator == trace.allocator);
        write!(
            trace.rows,
            "{},{},{indices},{heads},{old};",
            self.value, state.size
        )
        .unwrap();
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
fn observed(mode: usize) {
    let trace = Rc::new(RefCell::new(Trace::default()));
    let mut map = IndexedMap::with_hasher(KeyPolicy, 1);
    trace.borrow_mut().allocator = map.allocator().identity().addr();
    trace.borrow_mut().observer = Some(map.observer());
    fill_observed(&mut map, &trace);
    match mode {
        0 => map.remove_last().unwrap(),
        1 => map.clear(false),
        _ => map.clear_with_allocator(Some(BaseAllocator::fresh_common())),
    }
    let old = u8::from(map.allocator().identity().addr() == trace.borrow().allocator);
    let row = format!("observer{mode} {} {} {old}", trace.borrow().rows, map.len());
    super::native_row(&format!("observer{mode}"), &row);
    trace.borrow_mut().observer = None;
}
#[test]
fn remove_last_drop_sees_old_size_and_cleared_index() {
    observed(0);
}
#[test]
fn clear_drop_sees_native_bucket_phase() {
    observed(1);
}
#[test]
fn replacing_allocator_adopts_after_last_drop() {
    observed(2);
}
#[test]
fn observer_survives_owner_and_exchange_keeps_container_identity() {
    let mut first = IndexedMap::<i32>::default();
    first.add(7);
    let mut second = IndexedMap::<i32>::default();
    second.add(8);
    second.add(9);
    let observer = first.observer();
    first.exchange(&mut second);
    assert_eq!(observer.snapshot().size, 2);
    drop(first);
    assert_eq!(observer.snapshot().size, 0);
    assert_eq!(observer.snapshot().index_table, 0);
}

fn fill_observed(map: &mut IndexedMap<Key, KeyPolicy>, trace: &Rc<RefCell<Trace>>) {
    for value in 1..=3 {
        map.add(Key {
            value,
            trace: trace.clone(),
        });
    }
}
