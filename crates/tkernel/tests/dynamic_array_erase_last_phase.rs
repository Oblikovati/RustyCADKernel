//! The destructor reads a scoped reference to actual owner metadata, not a copied snapshot.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::DynamicArray;
use std::{
    cell::RefCell,
    fmt::Write,
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};
#[derive(Default)]
struct EraseProbe {
    used: *const usize,
    slot: *const EraseItem,
    events: String,
    fail: bool,
    drops: Vec<i32>,
}
struct EraseItem {
    value: i32,
    probe: Rc<RefCell<EraseProbe>>,
}
impl Drop for EraseItem {
    fn drop(&mut self) {
        let mut probe = self.probe.borrow_mut();
        probe.drops.push(self.value);
        if probe.used.is_null() {
            return;
        }
        // The callback installs this pointer only while its disjoint metadata borrow is live.
        let used = unsafe { *probe.used };
        let same = std::ptr::eq(self, probe.slot);
        write!(
            probe.events,
            "D{}@{used}:{};",
            self.value,
            usize::from(same)
        )
        .unwrap();
        assert!(
            !probe.fail,
            "destructor {}: expected nonpanicking native domain",
            self.value
        );
    }
}
struct EraseContext(Rc<RefCell<EraseProbe>>);
impl Drop for EraseContext {
    fn drop(&mut self) {
        self.0.borrow_mut().used = std::ptr::null();
    }
}
fn erase(array: &mut DynamicArray<EraseItem>, probe: &Rc<RefCell<EraseProbe>>) {
    let before = array.size();
    let expected = array
        .last()
        .map(|item| item as *const EraseItem)
        .unwrap_or(std::ptr::null());
    // The callback destroys exactly once and its guard clears all borrowed metadata on unwind.
    unsafe {
        array
            .erase_last_with_destruction(|slot, used| {
                {
                    let mut state = probe.borrow_mut();
                    state.used = used;
                    state.slot = expected;
                }
                let _context = EraseContext(Rc::clone(probe));
                slot.drop_in_place();
            })
            .unwrap();
    }
    write!(probe.borrow_mut().events, "S{before},{};", array.size()).unwrap();
}
fn fixture(count: i32) -> (DynamicArray<EraseItem>, Rc<RefCell<EraseProbe>>) {
    let probe = Rc::new(RefCell::new(EraseProbe::default()));
    let mut array = DynamicArray::with_increment(2);
    for value in 1..=count {
        array.append(EraseItem {
            value,
            probe: Rc::clone(&probe),
        });
    }
    (array, probe)
}
fn actual_row(count: i32) -> String {
    let (mut array, probe) = fixture(count);
    let erased = array
        .last()
        .map(|item| item as *const EraseItem)
        .unwrap_or(std::ptr::null());
    erase(&mut array, &probe);
    let appended = array.append(EraseItem {
        value: 77,
        probe: Rc::clone(&probe),
    });
    let retained = count == 0 || std::ptr::eq(erased, appended);
    erase(&mut array, &probe);
    while !array.is_empty() {
        erase(&mut array, &probe);
    }
    erase(&mut array, &probe);
    format!(
        "erase_{count} {} retained={}",
        probe.borrow().events,
        usize::from(retained)
    )
}
#[test]
fn original_slot_and_actual_size_match_native_destructor_trace() {
    let expected = include_str!("../../../tools/oracle/tkernel_dynamic_erase_last_phase.expected");
    for (count, row) in expected.lines().enumerate() {
        assert_eq!(actual_row(count as i32), row);
    }
}
#[test]
fn panic_recovers_size_and_does_not_destroy_last_twice() {
    let (mut array, probe) = fixture(3);
    probe.borrow_mut().fail = true;
    assert!(catch_unwind(AssertUnwindSafe(|| erase(&mut array, &probe))).is_err());
    assert_eq!(array.size(), 2);
    assert_eq!(probe.borrow().events, "D3@3:1;");
    assert!(probe.borrow().used.is_null());
    probe.borrow_mut().fail = false;
    array.clear(true);
    assert_eq!(probe.borrow().drops, [3, 1, 2]);
}
#[test]
fn empty_array_never_calls_destructor_and_needs_no_traits() {
    let mut array = DynamicArray::<EraseItem>::with_increment(2);
    // No live value exists and no callback may execute.
    unsafe {
        array
            .erase_last_with_destruction(|_, _| panic!("empty: expected no callback"))
            .unwrap();
    }
    assert_eq!(array.size(), 0);
}
#[test]
fn hidden_initialized_tail_is_rejected_without_changes() {
    let mut array = DynamicArray::with_increment(4);
    array.append(1_i32);
    array[3] = 9;
    // Validation rejects before the destructive callback takes ownership.
    let result = unsafe {
        array.erase_last_with_destruction(|_, _| panic!("hidden tail: expected rejection"))
    };
    assert!(result.is_err());
    assert_eq!(array.size(), 1);
    assert_eq!(array[3], 9);
}
#[test]
fn trivial_item_is_erased_without_releasing_its_block() {
    let mut array = DynamicArray::with_increment(2);
    let before = array.append(4_u32) as *const u32;
    // Trivial native destructors are no-ops; ending the logical lifetime needs no callback action.
    unsafe {
        array
            .erase_last_with_destruction(|_, size| assert_eq!(*size, 1))
            .unwrap();
    }
    let after = array.append(5_u32) as *const u32;
    assert_eq!(before, after);
    assert_eq!(array.size(), 1);
}
