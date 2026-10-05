//! Focused borrowed-copy DataMap binding semantics.
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::ncollection::HashPolicy;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Default)]
struct Counts {
    clones: usize,
    key_clones: usize,
    drops: usize,
    hashes: usize,
    order: Vec<&'static str>,
}

#[derive(Debug)]
struct Item {
    value: i32,
    counts: Rc<RefCell<Counts>>,
}

impl Item {
    fn new(value: i32, counts: &Rc<RefCell<Counts>>) -> Self {
        Self {
            value,
            counts: Rc::clone(counts),
        }
    }
}

impl Clone for Item {
    fn clone(&self) -> Self {
        let mut counts = self.counts.borrow_mut();
        counts.clones += 1;
        counts.order.push("value");
        drop(counts);
        Self::new(self.value, &self.counts)
    }
}

impl Drop for Item {
    fn drop(&mut self) {
        self.counts.borrow_mut().drops += 1;
    }
}

#[derive(Debug)]
struct Key {
    value: i32,
    counts: Rc<RefCell<Counts>>,
}

impl Clone for Key {
    fn clone(&self) -> Self {
        let mut counts = self.counts.borrow_mut();
        counts.key_clones += 1;
        counts.order.push("key");
        drop(counts);
        Self {
            value: self.value,
            counts: Rc::clone(&self.counts),
        }
    }
}

struct KeyHash(Rc<RefCell<Counts>>);

impl HashPolicy<Key> for KeyHash {
    fn hash(&self, key: &Key) -> usize {
        self.0.borrow_mut().hashes += 1;
        key.value as usize
    }
    fn equal(&self, first: &Key, second: &Key) -> bool {
        first.value == second.value
    }
}

#[test]
fn bind_ref_copies_only_after_miss_and_keeps_source_live() {
    let counts = Rc::new(RefCell::new(Counts::default()));
    let value = Item::new(7, &counts);
    let mut map = DataMap::<i32, Item>::default();

    assert!(map.bind_ref(&4, &value));

    assert_eq!(counts.borrow().clones, 1);
    assert_eq!(map.find(&4).unwrap().value, 7);
    assert_eq!(value.value, 7);
}

#[test]
fn bind_ref_replacement_clones_value_and_keeps_slot_address() {
    let counts = Rc::new(RefCell::new(Counts::default()));
    let replacement = Item::new(9, &counts);
    let mut map = DataMap::<i32, Item>::default();
    map.bind(4, Item::new(3, &counts));
    let original_slot = map.find(&4).unwrap() as *const Item;

    assert!(!map.bind_ref(&4, &replacement));

    assert_eq!(counts.borrow().clones, 1);
    assert_eq!(counts.borrow().drops, 1);
    assert_eq!(map.find(&4).unwrap() as *const Item, original_slot);
    assert_eq!(map.find(&4).unwrap().value, 9);
}

#[test]
fn bind_ref_looks_up_before_copy_and_constructs_item_before_key() {
    let counts = Rc::new(RefCell::new(Counts::default()));
    let key = Key {
        value: 6,
        counts: Rc::clone(&counts),
    };
    let value = Item::new(12, &counts);
    let mut map = DataMap::with_hasher(KeyHash(Rc::clone(&counts)), 1);

    assert!(map.bind_ref(&key, &value));

    assert_eq!(counts.borrow().order, ["value", "key"]);
    assert_eq!(counts.borrow().key_clones, 1);
    assert_eq!(counts.borrow().hashes, 1);
    counts.borrow_mut().order.clear();
    counts.borrow_mut().hashes = 0;
    assert!(!map.bind_ref(&key, &value));
    assert_eq!(counts.borrow().order, ["value"]);
    assert_eq!(counts.borrow().key_clones, 1);
    assert_eq!(counts.borrow().hashes, 1);
}
