//! Shared metadata synchronization never grants an observer access to payload memory.
use rustycad_tkernel::ncollection::{DoubleMap, HashPolicy, IndexedMap};
use std::rc::Rc;
mod map_topology_sync_contracts {
    pub mod changed_routes;
}
#[derive(Default)]
struct LocalKeyPolicy;
impl HashPolicy<Rc<i32>> for LocalKeyPolicy {
    fn hash(&self, key: &Rc<i32>) -> usize {
        **key as usize
    }
    fn equal(&self, first: &Rc<i32>, second: &Rc<i32>) -> bool {
        first == second
    }
}
type LocalDoubleMap = DoubleMap<Rc<i32>, Rc<i32>, LocalKeyPolicy, LocalKeyPolicy>;
type LocalIndexedMap = IndexedMap<Rc<i32>, LocalKeyPolicy>;
#[test]
fn double_observer_clones_follow_original_owner_after_exchange_and_drop() {
    let (mut first, mut second) = (LocalDoubleMap::new(1), DoubleMap::new(1));
    first.bind(Rc::new(1), Rc::new(11)).unwrap();
    second.bind(Rc::new(2), Rc::new(22)).unwrap();
    second.bind(Rc::new(3), Rc::new(33)).unwrap();
    let observer = first.observer();
    let cloned = observer.clone();
    first.exchange(&mut second);
    assert_eq!(observer.snapshot().size, 2);
    assert_eq!(cloned.snapshot(), observer.snapshot());
    drop(first);
    std::thread::spawn(move || {
        assert_eq!(observer.snapshot(), cloned.snapshot());
        assert_eq!(observer.snapshot().size, 0);
        assert_eq!(observer.snapshot().first_table, 0);
    })
    .join()
    .unwrap();
}
#[test]
fn indexed_observer_clones_follow_original_owner_after_exchange_and_drop() {
    let (mut first, mut second) = (LocalIndexedMap::new(1), IndexedMap::new(1));
    first.add(Rc::new(1));
    second.add(Rc::new(2));
    second.add(Rc::new(3));
    let observer = first.observer();
    let cloned = observer.clone();
    first.exchange(&mut second);
    assert_eq!(observer.snapshot().size, 2);
    assert_eq!(cloned.snapshot(), observer.snapshot());
    drop(first);
    std::thread::spawn(move || {
        assert_eq!(observer.snapshot(), cloned.snapshot());
        assert_eq!(observer.snapshot().size, 0);
        assert_eq!(observer.snapshot().index_table, 0);
    })
    .join()
    .unwrap();
}
#[test]
fn double_observer_snapshots_can_race_owner_edits_without_payload_reads() {
    let mut map = DoubleMap::<usize, usize>::new(1);
    let observer = map.observer();
    let worker = std::thread::spawn(move || {
        for _ in 0..8 {
            let snapshot = observer.snapshot();
            assert!(snapshot.size <= 8);
            assert_eq!(snapshot.first_buckets.len(), snapshot.second_buckets.len());
        }
    });
    for key in 0..8 {
        map.bind(key, key + 10).unwrap();
    }
    map.clear(true);
    worker.join().unwrap();
}
#[test]
fn indexed_observer_snapshots_can_race_owner_edits_without_payload_reads() {
    let mut map = IndexedMap::<usize>::new(1);
    let observer = map.observer();
    let worker = std::thread::spawn(move || {
        for _ in 0..8 {
            let snapshot = observer.snapshot();
            assert!(snapshot.size <= 8);
            assert!(snapshot.indices.len() >= snapshot.size);
        }
    });
    for key in 0..8 {
        map.add(key);
    }
    map.clear(true);
    worker.join().unwrap();
}
