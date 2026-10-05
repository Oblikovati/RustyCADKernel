//! Supplemental assignment contracts; original OCCT declaration counts are unchanged.
use rustycad_tkernel::ncollection::{BaseAllocator, DataMap, HashPolicy};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

type CopyEvents = Rc<RefCell<Vec<char>>>;

#[derive(Debug)]
struct CopyTracked {
    number: i32,
    kind: char,
    events: CopyEvents,
    fail_copy: Rc<Cell<bool>>,
}

impl Clone for CopyTracked {
    fn clone(&self) -> Self {
        self.events.borrow_mut().push(self.kind);
        assert!(!self.fail_copy.get(), "injected {} copy failure", self.kind);
        Self {
            number: self.number,
            kind: self.kind,
            events: Rc::clone(&self.events),
            fail_copy: Rc::clone(&self.fail_copy),
        }
    }
}

#[derive(Clone)]
struct AssignmentHash {
    salt: usize,
    events: CopyEvents,
}

impl HashPolicy<CopyTracked> for AssignmentHash {
    fn hash(&self, key: &CopyTracked) -> usize {
        self.events.borrow_mut().push('H');
        key.number as usize + self.salt
    }
    fn equal(&self, first: &CopyTracked, second: &CopyTracked) -> bool {
        first.number == second.number
    }
}

fn assignment_source(events: &CopyEvents) -> DataMap<CopyTracked, CopyTracked, AssignmentHash> {
    let policy = AssignmentHash {
        salt: 1,
        events: Rc::clone(events),
    };
    let mut source = DataMap::with_hasher(policy, 101);
    source.bind(copy_tracked(4, 'K', events), copy_tracked(9, 'V', events));
    source
}

fn copy_tracked(number: i32, kind: char, events: &CopyEvents) -> CopyTracked {
    CopyTracked {
        number,
        kind,
        events: Rc::clone(events),
        fail_copy: Rc::new(Cell::new(false)),
    }
}

fn assignment_target(
    events: &CopyEvents,
    arena: &BaseAllocator,
) -> DataMap<CopyTracked, CopyTracked, AssignmentHash> {
    let policy = AssignmentHash {
        salt: 17,
        events: Rc::clone(events),
    };
    DataMap::with_allocator(policy, 101, arena.clone())
}

#[test]
fn assignment_returns_destination_and_copies_value_before_key_using_destination_policy() {
    let events: CopyEvents = Rc::new(RefCell::new(Vec::new()));
    let source = assignment_source(&events);
    let arena = BaseAllocator::incremental(256);
    let mut target = assignment_target(&events, &arena);
    events.borrow_mut().clear();
    let identity = std::ptr::from_ref(&target);
    assert_eq!(
        std::ptr::from_mut(target.assign_returning(&source)),
        identity.cast_mut()
    );
    assert_eq!(*events.borrow(), ['H', 'V', 'K']);
    assert_eq!(target.hasher().salt, 17);
    assert_eq!(target.allocator(), &arena);
    assert_eq!(target.items().next().unwrap().1.number, 9);
    assert_eq!(source.len(), 1);
}

#[test]
fn self_assignment_preserves_node_address_and_performs_no_hash_or_copy() {
    let events: CopyEvents = Rc::new(RefCell::new(Vec::new()));
    let mut map = assignment_source(&events);
    let identity = std::ptr::from_ref(&map);
    let stored_value = std::ptr::from_ref(map.items().next().unwrap().1);
    events.borrow_mut().clear();
    map.items().next().unwrap().0.fail_copy.set(true);
    assert_eq!(std::ptr::from_mut(map.assign_self()), identity.cast_mut());
    assert_eq!(
        std::ptr::from_ref(map.items().next().unwrap().1),
        stored_value
    );
    assert!(events.borrow().is_empty());
}

#[test]
fn copy_failure_leaves_destination_empty_and_releases_preallocated_node() {
    let events: CopyEvents = Rc::new(RefCell::new(Vec::new()));
    let source = assignment_source(&events);
    source.items().next().unwrap().0.fail_copy.set(true);
    let arena = BaseAllocator::incremental(256);
    let mut target = assignment_target(&events, &arena);
    target.bind(copy_tracked(2, 'K', &events), copy_tracked(3, 'V', &events));
    events.borrow_mut().clear();
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| target.assign(&source)));
    assert!(failure.is_err());
    assert!(target.is_empty());
    assert_eq!(*events.borrow(), ['H', 'V', 'K']);
    arena.reset(false).unwrap();
}

#[test]
fn move_assignment_returns_destination_and_exchanges_nodes_policy_and_allocator() {
    let first_arena = BaseAllocator::incremental(256);
    let second_arena = BaseAllocator::incremental(256);
    let mut first = DataMap::<i32, i32>::default();
    let mut second = DataMap::<i32, i32>::default();
    first.clear_with_allocator(Some(first_arena.clone()));
    second.clear_with_allocator(Some(second_arena.clone()));
    first.bind(1, 7);
    second.bind(2, 9);
    let identity = std::ptr::from_ref(&first);
    let stored = std::ptr::from_ref(second.find(&2).unwrap());
    let returned = std::ptr::from_mut(first.move_from_returning(&mut second));
    assert_eq!(returned, identity.cast_mut());
    assert_eq!(std::ptr::from_ref(first.find(&2).unwrap()), stored);
    assert_eq!(first.allocator(), &second_arena);
    assert_eq!(second.allocator(), &first_arena);
    assert_eq!(second.find(&1), Ok(&7));
}

#[test]
fn assigning_empty_source_clears_bindings_and_returns_destination() {
    let source = DataMap::<i32, i32>::default();
    let mut target = DataMap::default();
    target.bind(1, 7);
    assert!(target.assign_returning(&source).is_empty());
}

#[test]
fn clone_uses_borrowed_node_copy_order_and_preserves_source_policy() {
    let events: CopyEvents = Rc::new(RefCell::new(Vec::new()));
    let source = assignment_source(&events);
    events.borrow_mut().clear();
    let copy = source.clone();
    assert_eq!(*events.borrow(), ['H', 'V', 'K']);
    assert_eq!(copy.hasher().salt, source.hasher().salt);
    assert_eq!(copy.allocator(), source.allocator());
    assert_eq!(copy.items().next().unwrap().1.number, 9);
}

#[test]
fn move_assignment_exchanges_stateful_hash_policies_without_copying_nodes() {
    let events: CopyEvents = Rc::new(RefCell::new(Vec::new()));
    let mut source = assignment_source(&events);
    let arena = BaseAllocator::incremental(256);
    let mut target = assignment_target(&events, &arena);
    target.bind(copy_tracked(2, 'K', &events), copy_tracked(3, 'V', &events));
    events.borrow_mut().clear();
    target.move_from_returning(&mut source);
    assert_eq!(target.hasher().salt, 1);
    assert_eq!(source.hasher().salt, 17);
    assert!(events.borrow().is_empty());
    assert_eq!(source.items().next().unwrap().1.number, 3);
}
