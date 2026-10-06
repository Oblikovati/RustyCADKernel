use rustycad_tkernel::ncollection::{BaseAllocator, List};
use std::cell::{Cell, RefCell};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;
struct DropItem {
    value: i32,
    trace: Rc<RefCell<Vec<i32>>>,
    panic_once: Rc<Cell<bool>>,
}
impl Drop for DropItem {
    fn drop(&mut self) {
        self.trace.borrow_mut().push(self.value);
        if self.value == 2 && self.panic_once.replace(false) {
            panic!("injected drop failure for value 2");
        }
    }
}
fn fixture(fails: bool) -> (List<DropItem>, Rc<RefCell<Vec<i32>>>, BaseAllocator) {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let panic_once = Rc::new(Cell::new(fails));
    let arena = BaseAllocator::incremental(4096);
    let mut list = List::with_allocator(arena.clone());
    for value in 1..=4 {
        list.append(DropItem {
            value,
            trace: trace.clone(),
            panic_once: panic_once.clone(),
        });
    }
    (list, trace, arena)
}
fn values(list: &List<DropItem>) -> Vec<i32> {
    list.iter().map(|item| item.value).collect()
}
#[test]
fn destruction_follows_native_chain_after_owner_order_diverges() {
    let (mut list, trace, arena) = fixture(false);
    list.reverse();
    let mut cursor = list.cursor();
    cursor.next(&list);
    list.remove(&mut cursor).unwrap();
    list.clear();
    assert_eq!(&*trace.borrow(), &[3, 4, 2, 1]);
    assert!(arena.reset(false).is_ok());
}
#[test]
fn clear_panic_resets_actual_base_and_reclaims_every_allocation() {
    let (mut list, trace, arena) = fixture(true);
    assert!(catch_unwind(AssertUnwindSafe(|| list.clear())).is_err());
    assert!(list.is_empty());
    assert_eq!(list.base_storage().size(), 0);
    assert!(list.base_storage().first().is_null());
    assert!(list.base_storage().last().is_null());
    trace.borrow_mut().sort_unstable();
    assert_eq!(&*trace.borrow(), &[1, 2, 3, 4]);
    assert!(arena.reset(false).is_ok());
}
#[test]
fn removed_destructor_panic_preserves_cursor_and_live_tail() {
    let (mut list, trace, arena) = fixture(true);
    let mut cursor = list.cursor();
    cursor.next(&list);
    assert!(catch_unwind(AssertUnwindSafe(|| list.remove(&mut cursor))).is_err());
    assert_eq!(values(&list), [1, 3, 4]);
    assert_eq!(cursor.value(&list).unwrap().value, 3);
    assert_eq!(list.base_storage().size(), 3);
    list.clear();
    assert_eq!(&*trace.borrow(), &[2, 1, 3, 4]);
    assert!(arena.reset(false).is_ok());
}
#[test]
fn move_assignment_destroys_destination_before_touching_source() {
    let (mut destination, _, arena) = fixture(true);
    let (mut source, _, source_arena) = fixture(false);
    let source_head = source.base_storage().first();
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            destination.move_assign(&mut source);
        }))
        .is_err()
    );
    assert_eq!(values(&source), [1, 2, 3, 4]);
    assert_eq!(source.base_storage().first(), source_head);
    assert!(destination.is_empty());
    assert!(arena.reset(false).is_ok());
    source.clear();
    assert!(source_arena.reset(false).is_ok());
}
#[test]
fn retain_callback_failure_keeps_prior_removals_and_live_base() {
    let mut list: List<i32> = [1, 2, 3, 4].into_iter().collect();
    let result = list.try_retain(|value| if *value == 3 { Err(3) } else { Ok(*value != 2) });
    assert_eq!(result, Err(3));
    assert_eq!(list.iter().copied().collect::<Vec<_>>(), [1, 3, 4]);
    assert_eq!(list.base_storage().size(), 3);
    assert_eq!(list.last(), Some(&4));
}

fn edge_removal_panic(first: bool) {
    let (mut list, trace, arena) = fixture(true);
    list.retain(|item| {
        if first {
            item.value >= 2
        } else {
            item.value <= 2
        }
    });
    let mut cursor = list.cursor();
    if !first {
        cursor.next(&list);
    }
    assert!(catch_unwind(AssertUnwindSafe(|| list.remove(&mut cursor))).is_err());
    let expected = if first { vec![3, 4] } else { vec![1] };
    assert_eq!(values(&list), expected);
    assert_eq!(list.base_storage().size(), expected.len());
    assert_eq!(list.last().unwrap().value, *expected.last().unwrap());
    finish_edge_cleanup(&mut list, &trace, &arena);
}
fn finish_edge_cleanup(
    list: &mut List<DropItem>,
    trace: &RefCell<Vec<i32>>,
    arena: &BaseAllocator,
) {
    list.clear();
    trace.borrow_mut().sort_unstable();
    assert_eq!(&*trace.borrow(), &[1, 2, 3, 4]);
    assert!(arena.reset(false).is_ok());
}
#[test]
fn first_and_last_drop_panics_complete_native_metadata_updates() {
    edge_removal_panic(true);
    edge_removal_panic(false);
}
