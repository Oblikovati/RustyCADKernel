//! Borrowed `IndexedMap::Add` keeps native lookup and key-copy ordering.
use rustycad_tkernel::ncollection::{HashPolicy, IndexedMap};
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

fn key(value: i32, events: &Rc<RefCell<Vec<char>>>) -> Key {
    Key {
        value,
        events: Rc::clone(events),
    }
}

#[test]
fn borrowed_add_hashes_before_copying_on_insert() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut map = IndexedMap::with_hasher(Policy(Rc::clone(&events)), 101);
    let incoming = key(7, &events);

    assert_eq!(map.add_ref(&incoming), 1);
    assert_eq!(*events.borrow(), ['H', 'K']);
    assert_eq!(map.find_key(1).unwrap().value, 7);
}

#[test]
fn borrowed_add_hit_does_not_clone_the_key() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut map = IndexedMap::with_hasher(Policy(Rc::clone(&events)), 101);
    let incoming = key(7, &events);
    assert_eq!(map.add_ref(&incoming), 1);
    events.borrow_mut().clear();

    assert_eq!(map.add_ref(&incoming), 1);
    assert_eq!(*events.borrow(), ['H', 'E']);
    assert_eq!(map.len(), 1);
}
