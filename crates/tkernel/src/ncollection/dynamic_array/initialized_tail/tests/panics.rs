use super::*;
#[test]
fn partial_slot_construction_marks_uncertainty_before_unwind() {
    let mut target = DynamicArray::<i32>::with_increment(4);
    reserve_blocks(&mut target, 1);
    let mut attempted = 0;
    let failure = catch_unwind(AssertUnwindSafe(|| {
        target.allocated_slot_with(3, || {
            attempted += 1;
            assert_ne!(attempted, 3, "default construction failure");
            10
        });
    }));
    assert!(failure.is_err());
    assert_eq!(target.blocks[0].len(), 2);
    assert!(target.needs_tail_scan);
    assert_eq!(assert_tail_state(&target), 1);
}
#[test]
fn emplace_constructor_failures_preserve_clean_and_uncertain_gates() {
    let mut target = DynamicArray::<i32>::with_increment(2);
    let failure = catch_unwind(AssertUnwindSafe(|| {
        target.emplace_value(3, || panic!("requested")).ok();
    }));
    assert!(failure.is_err());
    assert!(!target.needs_tail_scan);
    assert_tail_state(&target);
    target[3] = 33;
    let failure = catch_unwind(AssertUnwindSafe(|| {
        target.emplace_value(4, || panic!("requested")).ok();
    }));
    assert!(failure.is_err());
    assert!(target.needs_tail_scan);
    assert_eq!(target[3], 33);
    assert_tail_state(&target);
}
pub(super) struct PanicDropItem {
    pub value: i32,
    pub fail_value: Rc<Cell<i32>>,
    pub trace: Rc<RefCell<Vec<i32>>>,
}
impl Drop for PanicDropItem {
    fn drop(&mut self) {
        self.trace.borrow_mut().push(self.value);
        if self.fail_value.get() == self.value {
            self.fail_value.set(-1);
            panic!("destructor {}", self.value);
        }
    }
}
pub(super) fn panic_items(fail: i32) -> (DynamicArray<PanicDropItem>, Rc<RefCell<Vec<i32>>>) {
    let fail_value = Rc::new(Cell::new(fail));
    let trace = Rc::new(RefCell::new(Vec::new()));
    let mut target = DynamicArray::with_increment(2);
    for value in 1..=3 {
        target.append(PanicDropItem {
            value,
            fail_value: Rc::clone(&fail_value),
            trace: Rc::clone(&trace),
        });
    }
    (target, trace)
}
#[test]
fn clear_panic_holes_remain_on_the_exact_scan_path() {
    let (mut target, trace) = panic_items(1);
    assert!(catch_unwind(AssertUnwindSafe(|| target.clear(false))).is_err());
    assert_eq!(target.blocks[0].len(), 0);
    assert_eq!(target.used, 3);
    assert!(target.needs_tail_scan);
    assert_eq!(assert_tail_state(&target), 2);
    assert!(!target.has_initialized_tail());
    target.clear(true);
    assert!(!target.needs_tail_scan);
    assert_eq!(*trace.borrow(), [1, 2, 3]);
}
#[test]
fn erase_panic_keeps_uncertainty_and_never_double_drops() {
    let (mut target, trace) = panic_items(3);
    assert!(catch_unwind(AssertUnwindSafe(|| target.erase_last())).is_err());
    assert!(target.needs_tail_scan);
    assert_eq!(target.used, 2);
    assert_eq!(assert_tail_state(&target), 2);
    target.clear(true);
    assert_eq!(*trace.borrow(), [3, 1, 2]);
    assert!(!target.needs_tail_scan);
}
fn append_hidden_items(target: &mut DynamicArray<PanicDropItem>, trace: &Rc<RefCell<Vec<i32>>>) {
    reserve_blocks(target, 3);
    let fail = Rc::clone(&target.blocks[0][0].fail_value);
    let mut next = 3;
    target.allocated_slot_with(5, || {
        next += 1;
        PanicDropItem {
            value: next,
            fail_value: Rc::clone(&fail),
            trace: Rc::clone(trace),
        }
    });
}
#[test]
fn holes_and_hidden_values_cannot_be_replaced_by_total_length() {
    let (mut target, trace) = panic_items(1);
    append_hidden_items(&mut target, &trace);
    assert!(catch_unwind(AssertUnwindSafe(|| target.clear(false))).is_err());
    assert_eq!(
        target.blocks.iter().map(|block| block.len()).sum::<usize>(),
        target.used
    );
    assert!(target.has_initialized_tail());
    assert_eq!(assert_tail_state(&target), 3);
    target.clear(true);
    assert_eq!(*trace.borrow(), [1, 2, 3, 4, 5]);
}
