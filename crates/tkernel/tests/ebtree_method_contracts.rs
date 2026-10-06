//! Editable-tree map updates preserve native key-copy and singleton lookup contracts.
use rustycad_tkernel::ncollection::{BaseAllocator, BoundingVolume, EBTree, HashKey, TreeSelector};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Default)]
struct Bounds {
    low: f64,
    high: f64,
}
impl Bounds {
    fn interval(low: f64, high: f64) -> Self {
        Self { low, high }
    }
}
impl BoundingVolume for Bounds {
    fn add(&mut self, other: &Self) {
        self.low = self.low.min(other.low);
        self.high = self.high.max(other.high);
    }
    fn is_out(&self, other: &Self) -> bool {
        self.high < other.low || other.high < self.low
    }
    fn square_extent(&self) -> f64 {
        (self.high - self.low).powi(2)
    }
}
#[derive(Debug, Default)]
struct Key {
    value: i32,
    copies: Rc<RefCell<Vec<i32>>>,
    hashes: Rc<RefCell<Vec<i32>>>,
}
impl Clone for Key {
    fn clone(&self) -> Self {
        self.copies.borrow_mut().push(self.value);
        Self {
            value: self.value,
            copies: Rc::clone(&self.copies),
            hashes: Rc::clone(&self.hashes),
        }
    }
}
impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl HashKey for Key {
    fn occt_hash(&self) -> usize {
        self.hashes.borrow_mut().push(self.value);
        self.value as usize
    }
}
fn keyed(value: i32, copies: &Rc<RefCell<Vec<i32>>>, hashes: &Rc<RefCell<Vec<i32>>>) -> Key {
    Key {
        value,
        copies: Rc::clone(copies),
        hashes: Rc::clone(hashes),
    }
}

#[test]
fn ebtree_rebinding_previous_leaf_does_not_clone_neighbor_key() {
    let copies = Rc::new(RefCell::new(Vec::new()));
    let hashes = Rc::new(RefCell::new(Vec::new()));
    let first = keyed(1, &copies, &hashes);
    let second = keyed(2, &copies, &hashes);
    let mut tree = EBTree::default();
    assert!(tree.add_ref(&first, &Bounds::interval(0., 1.)));
    assert_eq!(*copies.borrow(), [1, 1]);
    copies.borrow_mut().clear();
    assert!(tree.add_ref(&second, &Bounds::interval(2., 3.)));
    assert_eq!(copies.borrow().iter().filter(|&&key| key == 1).count(), 1);
    assert_eq!(copies.borrow().iter().filter(|&&key| key == 2).count(), 2);
    assert_eq!(tree.find_node(&first).unwrap().object().value, 1);
    assert_eq!(tree.find_node(&second).unwrap().object().value, 2);
}

#[test]
fn ebtree_survivor_rebinding_changes_pointer_without_copying_map_key() {
    let copies = Rc::new(RefCell::new(Vec::new()));
    let hashes = Rc::new(RefCell::new(Vec::new()));
    let first = keyed(1, &copies, &hashes);
    let second = keyed(2, &copies, &hashes);
    let mut tree = EBTree::default();
    tree.add_ref(&first, &Bounds::interval(0., 1.));
    tree.add_ref(&second, &Bounds::interval(2., 3.));
    let root = tree.as_ubtree().root().unwrap() as *const _;
    copies.borrow_mut().clear();
    assert!(tree.remove(&second));
    assert_eq!(*copies.borrow(), [1]);
    assert_eq!(tree.find_node(&first).unwrap() as *const _, root);
    assert!(tree.find_node(&first).unwrap().is_root());
    assert!(!tree.contains(&second));
}

#[test]
fn ebtree_singleton_remove_uses_one_lookup_then_clear() {
    let copies = Rc::new(RefCell::new(Vec::new()));
    let hashes = Rc::new(RefCell::new(Vec::new()));
    let object = keyed(1, &copies, &hashes);
    let mut tree = EBTree::default();
    tree.add_ref(&object, &Bounds::interval(0., 1.));
    hashes.borrow_mut().clear();
    copies.borrow_mut().clear();
    assert!(tree.remove(&object));
    assert_eq!(*hashes.borrow(), [1]);
    assert!(copies.borrow().is_empty());
    assert!(tree.is_empty());
    assert!(!tree.remove(&object));
}

#[test]
fn ebtree_borrowed_duplicate_does_not_copy_or_change_geometry() {
    let copies = Rc::new(RefCell::new(Vec::new()));
    let hashes = Rc::new(RefCell::new(Vec::new()));
    let object = keyed(1, &copies, &hashes);
    let mut tree = EBTree::default();
    tree.add_ref(&object, &Bounds::interval(0., 1.));
    copies.borrow_mut().clear();
    hashes.borrow_mut().clear();
    assert!(!tree.add_ref(&object, &Bounds::interval(10., 20.)));
    assert!(copies.borrow().is_empty());
    assert_eq!(*hashes.borrow(), [1]);
    assert_eq!(tree.find_node(&object).unwrap().bounds().high, 1.);
}

#[derive(Default)]
struct Collect {
    objects: Vec<i32>,
}
impl TreeSelector<i32, Bounds> for Collect {
    fn reject(&self, _: &Bounds) -> bool {
        false
    }
    fn accept(&mut self, object: &i32) -> bool {
        self.objects.push(*object);
        true
    }
    fn stop(&self) -> bool {
        false
    }
}

#[test]
fn ebtree_dynamic_selector_and_returning_move_preserve_index_and_storage() {
    let arena = BaseAllocator::incremental(4096);
    let mut source = EBTree::<i32, Bounds>::with_allocator(arena.clone());
    source.add(1, Bounds::interval(0., 1.));
    source.add(2, Bounds::interval(2., 3.));
    let first = source.find_node(&1).unwrap() as *const _;
    let mut target = EBTree::default();
    target.add(9, Bounds::interval(9., 10.));
    let destination = &raw const target;
    assert_eq!(target.move_assign(&mut source) as *const _, destination);
    assert_eq!(target.find_node(&1).unwrap() as *const _, first);
    assert_eq!(target.as_ubtree().allocator(), &arena);
    assert!(source.is_empty());
    assert!(!source.contains(&1));
    assert!(!target.contains(&9));
    assert_eq!(target.move_assign_self() as *const _, destination);
    let mut selected = Collect::default();
    let dynamic: &mut dyn TreeSelector<i32, Bounds> = &mut selected;
    assert_eq!(target.select(dynamic), 2);
    assert_eq!(selected.objects, [1, 2]);
}

#[test]
fn ebtree_filler_duplicate_borrowed_dispatch_skips_object_copies() {
    use rustycad_tkernel::ncollection::UBTreeFiller;
    let copies = Rc::new(RefCell::new(Vec::new()));
    let hashes = Rc::new(RefCell::new(Vec::new()));
    let object = keyed(1, &copies, &hashes);
    let mut tree = EBTree::default();
    tree.add_ref(&object, &Bounds::interval(0., 1.));
    {
        let mut filler = UBTreeFiller::new(&mut tree, None, true);
        filler.add_ref(&object, &Bounds::interval(10., 20.));
        copies.borrow_mut().clear();
        assert_eq!(filler.fill(), 1);
        assert!(copies.borrow().is_empty());
    }
    assert_eq!(tree.find_node(&object).unwrap().bounds().high, 1.);
}
