//! Native List allocates each node before invoking user construction or copying.
use rustycad_tkernel::ncollection::{BaseAllocator, List, ListCursor};
use std::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

fn live_nodes(arena: &BaseAllocator) -> usize {
    let failure = arena.reset(false).unwrap_err();
    failure
        .message()
        .split_whitespace()
        .next()
        .unwrap()
        .parse()
        .unwrap()
}
struct ObservedCopy {
    value: i32,
    arena: BaseAllocator,
    copies: Rc<RefCell<Vec<usize>>>,
    fails: bool,
}
impl Clone for ObservedCopy {
    fn clone(&self) -> Self {
        self.copies.borrow_mut().push(live_nodes(&self.arena));
        assert!(
            !self.fails,
            "copy value {} failed; expected successful copy",
            self.value
        );
        Self {
            value: self.value,
            arena: self.arena.clone(),
            copies: self.copies.clone(),
            fails: false,
        }
    }
}
fn observed(value: i32, arena: &BaseAllocator, copies: &Rc<RefCell<Vec<usize>>>) -> ObservedCopy {
    ObservedCopy {
        value,
        arena: arena.clone(),
        copies: copies.clone(),
        fails: value == 2,
    }
}
fn values(list: &List<ObservedCopy>) -> Vec<i32> {
    list.iter().map(|item| item.value).collect()
}

fn checked_construction(arena: &BaseAllocator, expected: usize, value: i32) -> i32 {
    assert_eq!(live_nodes(arena), expected);
    value
}
#[test]
fn every_emplacement_allocates_before_constructor() {
    let arena = BaseAllocator::incremental(1024);
    let mut list = List::with_allocator(arena.clone());
    let appended = list.emplace_append(|| checked_construction(&arena, 1, 1));
    assert_eq!(*appended, 1);
    let prepended = list.emplace_prepend(|| checked_construction(&arena, 2, 0));
    assert_eq!(*prepended, 0);
    let cursor = list.cursor();
    let before = list.emplace_before(&cursor, || checked_construction(&arena, 3, 2));
    assert_eq!(*before.unwrap(), 2);
    let after = list.emplace_after(&cursor, || checked_construction(&arena, 4, 3));
    assert_eq!(*after.unwrap(), 3);
    assert_eq!(list.iter().copied().collect::<Vec<_>>(), [2, 0, 3, 1]);
}
#[test]
fn all_borrowed_overloads_copy_after_allocation() {
    let arena = BaseAllocator::incremental(1024);
    let copies = Rc::new(RefCell::new(Vec::new()));
    let item = observed(7, &arena, &copies);
    let mut list = List::with_allocator(arena);
    list.append_ref(&item);
    list.prepend_ref(&item);
    let mut cursor = list.cursor();
    list.insert_before_ref(&item, &cursor).unwrap();
    list.insert_after_ref(&item, &cursor).unwrap();
    list.append_at_ref(&item, &mut cursor);
    list.insert_before_at_ref(&item, &mut cursor).unwrap();
    assert_eq!(&*copies.borrow(), &[1, 2, 3, 4, 5, 6]);
    assert_eq!(cursor.value(&list).unwrap().value, 7);
    assert_eq!(list.len(), 6);
}
fn panic_emplacement(mode: usize, list: &mut List<i32>, arena: &BaseAllocator) {
    let cursor = list.cursor();
    let construct = || {
        assert_eq!(live_nodes(arena), 2);
        panic!("injected construction failure")
    };
    let _ = match mode {
        0 => list.emplace_append(construct),
        1 => list.emplace_prepend(construct),
        2 => list.emplace_before(&cursor, construct).unwrap(),
        _ => list.emplace_after(&cursor, construct).unwrap(),
    };
}
#[test]
fn constructor_failure_frees_uninitialized_node_and_preserves_chain() {
    for mode in 0..4 {
        let arena = BaseAllocator::incremental(1024);
        let mut list = List::from_iter_in([9], arena.clone());
        let node = list.cursor().current_node();
        assert!(
            catch_unwind(AssertUnwindSafe(|| panic_emplacement(
                mode, &mut list, &arena
            )))
            .is_err()
        );
        assert_eq!(list.iter().copied().collect::<Vec<_>>(), [9]);
        assert_eq!(list.cursor().current_node(), node);
        assert_eq!(live_nodes(&arena), 1);
    }
}
fn copy_collection(
    mode: usize,
    destination: &mut List<ObservedCopy>,
    source: &mut List<ObservedCopy>,
) {
    let cursor = destination.cursor();
    match mode {
        0 => destination.append_list(source),
        1 => destination.prepend_list(source),
        2 => destination.insert_list_before(source, &cursor).unwrap(),
        _ => {
            destination.assign(source);
        }
    }
}
fn check_partial_copy(mode: usize) {
    let arena = BaseAllocator::incremental(1024);
    let copies = Rc::new(RefCell::new(Vec::new()));
    let items = [1, 2, 3].map(|value| observed(value, &arena, &copies));
    let mut source: List<_> = items.into_iter().collect();
    let mut destination = List::from_iter_in([observed(9, &arena, &copies)], arena.clone());
    let failed = catch_unwind(AssertUnwindSafe(|| {
        copy_collection(mode, &mut destination, &mut source)
    }));
    assert!(failed.is_err());
    assert_eq!(values(&source), [1, 2, 3]);
    let expected = match mode {
        0 => vec![9, 1],
        3 => vec![1],
        _ => vec![1, 9],
    };
    assert_eq!(values(&destination), expected);
    assert_eq!(&*copies.borrow(), if mode == 3 { &[1, 2] } else { &[2, 3] });
    assert_eq!(live_nodes(&arena), destination.len());
}
#[test]
fn failing_collection_copies_keep_native_prefix_and_allocation_order() {
    for mode in 0..4 {
        check_partial_copy(mode);
    }
}
#[test]
fn clone_uses_same_allocator_and_allocates_before_each_copy() {
    let arena = BaseAllocator::incremental(1024);
    let copies = Rc::new(RefCell::new(Vec::new()));
    let source = List::from_iter_in(
        [observed(1, &arena, &copies), observed(3, &arena, &copies)],
        arena.clone(),
    );
    let copy = source.clone();
    assert_eq!(&*copies.borrow(), &[3, 4]);
    assert_eq!(copy.allocator(), &arena);
    assert_eq!(values(&copy), [1, 3]);
    assert_ne!(copy.cursor().current_node(), source.cursor().current_node());
}
#[test]
fn failed_borrowed_copy_precedes_invalid_cursor_validation() {
    let arena = BaseAllocator::incremental(1024);
    let copies = Rc::new(RefCell::new(Vec::new()));
    let item = observed(2, &arena, &copies);
    let mut list = List::with_allocator(arena.clone());
    let failed = catch_unwind(AssertUnwindSafe(|| {
        list.insert_before_ref(&item, &ListCursor::default())
            .unwrap();
    }));
    assert!(failed.is_err());
    assert_eq!(&*copies.borrow(), &[1]);
    assert!(list.is_empty());
    assert!(arena.reset(false).is_ok());
}
