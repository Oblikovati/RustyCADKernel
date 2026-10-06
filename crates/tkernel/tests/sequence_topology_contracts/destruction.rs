use rustycad_tkernel::ncollection::{BaseAllocator, Sequence};
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
fn fixture(fails: bool) -> (Sequence<DropItem>, Rc<RefCell<Vec<i32>>>, BaseAllocator) {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let panic_once = Rc::new(Cell::new(fails));
    let arena = BaseAllocator::incremental(4096);
    let mut sequence = Sequence::with_allocator(arena.clone());
    for value in 1..=4 {
        sequence.append(DropItem {
            value,
            trace: trace.clone(),
            panic_once: panic_once.clone(),
        });
    }
    (sequence, trace, arena)
}
#[test]
fn destruction_follows_native_links_after_owner_registry_order_diverges() {
    let (mut sequence, trace, arena) = fixture(false);
    sequence.reverse();
    sequence.remove(2).unwrap();
    sequence.clear(None);
    assert_eq!(&*trace.borrow(), &[3, 4, 2, 1]);
    assert!(arena.reset(false).is_ok());
}
#[test]
fn clear_drop_panic_resets_metadata_and_frees_remaining_owners_once() {
    let (mut sequence, trace, arena) = fixture(true);
    assert!(catch_unwind(AssertUnwindSafe(|| sequence.clear(None))).is_err());
    assert!(sequence.is_empty());
    assert!(sequence.first().is_none());
    assert!(sequence.base_access().current_node().is_null());
    trace.borrow_mut().sort_unstable();
    assert_eq!(&*trace.borrow(), &[1, 2, 3, 4]);
    assert!(arena.reset(false).is_ok());
}
#[test]
fn range_drop_panic_leaves_live_survivors_and_reclaims_orphans_on_clear() {
    let (mut sequence, trace, arena) = fixture(true);
    assert!(catch_unwind(AssertUnwindSafe(|| sequence.remove_range(2, 3))).is_err());
    assert_eq!(
        sequence.iter().map(|item| item.value).collect::<Vec<_>>(),
        [1, 4]
    );
    sequence.clear(None);
    trace.borrow_mut().sort_unstable();
    assert_eq!(&*trace.borrow(), &[1, 2, 3, 4]);
    assert!(arena.reset(false).is_ok());
}
