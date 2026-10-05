//! Borrowed `IndexedDataMap::Add` preserves OCCT's lookup and construction order.
use rustycad_tkernel::ncollection::{HashPolicy, IndexedDataMap};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
struct Key {
    value: i32,
    events: Rc<RefCell<Vec<char>>>,
}

impl Clone for Key {
    fn clone(&self) -> Self {
        self.events.borrow_mut().push('K');
        Self {
            value: self.value,
            events: Rc::clone(&self.events),
        }
    }
}

#[derive(Debug)]
struct Item {
    value: i32,
    events: Rc<RefCell<Vec<char>>>,
}

impl Clone for Item {
    fn clone(&self) -> Self {
        self.events.borrow_mut().push('V');
        Self {
            value: self.value,
            events: Rc::clone(&self.events),
        }
    }
}

struct Policy(Rc<RefCell<Vec<char>>>);

impl HashPolicy<Key> for Policy {
    fn hash(&self, key: &Key) -> usize {
        self.0.borrow_mut().push('H');
        key.value as usize
    }

    fn equal(&self, left: &Key, right: &Key) -> bool {
        self.0.borrow_mut().push('E');
        left.value == right.value
    }
}

#[test]
fn borrowed_add_hashes_before_cloning_item_then_key() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut map = IndexedDataMap::with_hasher(Policy(Rc::clone(&events)), 101);
    let key = Key {
        value: 7,
        events: Rc::clone(&events),
    };
    let item = Item {
        value: 42,
        events: Rc::clone(&events),
    };

    assert_eq!(map.add_ref(&key, &item), 1);
    assert_eq!(*events.borrow(), ['H', 'V', 'K']);
    assert_eq!(map.find_from_index(1).unwrap().value, 42);
}

#[test]
fn borrowed_add_hit_keeps_item_and_skips_both_clones() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut map = IndexedDataMap::with_hasher(Policy(Rc::clone(&events)), 101);
    let key = Key {
        value: 7,
        events: Rc::clone(&events),
    };
    let first = Item {
        value: 42,
        events: Rc::clone(&events),
    };
    let ignored = Item {
        value: 99,
        events: Rc::clone(&events),
    };

    assert_eq!(map.add_ref(&key, &first), 1);
    events.borrow_mut().clear();
    assert_eq!(map.add_ref(&key, &ignored), 1);
    assert_eq!(*events.borrow(), ['H', 'E']);
    assert_eq!(map.find_from_index(1).unwrap().value, 42);
}
