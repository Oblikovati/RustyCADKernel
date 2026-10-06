use rustycad_tkernel::ncollection::{
    BasicMapIteratorSource, HashPolicy, OrderedDataMap, OrderedMap,
};
use std::{cell::Cell, rc::Rc};
struct Payload {
    key: i32,
    drops: Rc<Cell<usize>>,
}
impl Drop for Payload {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}
struct PayloadHash;
impl HashPolicy<Payload> for PayloadHash {
    fn hash(&self, key: &Payload) -> usize {
        key.key as usize
    }
    fn equal(&self, a: &Payload, b: &Payload) -> bool {
        a.key == b.key
    }
}
fn payload(key: i32, drops: &Rc<Cell<usize>>) -> Payload {
    Payload {
        key,
        drops: Rc::clone(drops),
    }
}
#[test]
fn non_clone_set_payload_moves_graph_and_drops_exactly_once() {
    let drops = Rc::new(Cell::new(0));
    let mut source = OrderedMap::with_hasher(PayloadHash, 101);
    source.add(payload(1, &drops));
    let first = source.basic_first_node();
    let target = OrderedMap::move_from_with(&mut source, |_| PayloadHash);
    assert_eq!(target.basic_first_node(), first);
    drop(source);
    assert_eq!(drops.get(), 0);
    drop(target);
    assert_eq!(drops.get(), 1);
}
#[test]
fn non_clone_pair_payload_moves_registry_and_preserves_safe_cursor_rejection() {
    let drops = Rc::new(Cell::new(0));
    let mut source = OrderedDataMap::with_hasher(PayloadHash, 101);
    source.bind(payload(1, &drops), payload(2, &drops));
    let cursor = source.cursor();
    let target = OrderedDataMap::move_from_with(&mut source, |_| PayloadHash);
    assert!(cursor.value(&source).is_err());
    assert!(cursor.value(&target).is_err());
    assert_eq!(target.cursor().value(&target).unwrap().key, 2);
    drop(source);
    assert_eq!(drops.get(), 0);
    drop(target);
    assert_eq!(drops.get(), 2);
}
