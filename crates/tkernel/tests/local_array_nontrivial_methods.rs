//! Native NCollection_LocalArray nontrivial constructor/move/destructor traces.
use rustycad_tkernel::ncollection::LocalArray;
use std::cell::Cell;
use std::cell::RefCell;

thread_local! {
    static MOVE_EVENTS: RefCell<String> = const { RefCell::new(String::new()) };
    static PANIC_ONCE: Cell<bool> = const { Cell::new(false) };
}

struct MoveObserved {
    value: i32,
}

impl Default for MoveObserved {
    fn default() -> Self {
        record("C,".into());
        Self { value: 0 }
    }
}

impl Drop for MoveObserved {
    fn drop(&mut self) {
        record(format!("D{},", self.value));
    }
}

fn record(event: String) {
    MOVE_EVENTS.with_borrow_mut(|events| events.push_str(&event));
}

fn move_observed(source: &mut MoveObserved) -> MoveObserved {
    record(format!("M{},", source.value));
    MoveObserved {
        value: std::mem::replace(&mut source.value, -1),
    }
}

fn take_events() -> String {
    MOVE_EVENTS.with_borrow_mut(std::mem::take)
}

type ObservedArray = LocalArray<MoveObserved, 4>;

fn filled(count: usize, base: i32) -> ObservedArray {
    let mut array = ObservedArray::new(count);
    for (index, item) in array.iter_mut().enumerate() {
        item.value = base + i32::try_from(index).unwrap();
    }
    array
}

fn check_row(label: &str, array: &ObservedArray, retained: bool) {
    let contents: String = array
        .iter()
        .map(|item| format!("{},", item.value))
        .collect();
    let row = format!(
        "{label}:{}:{}:{}:{contents}",
        array.len(),
        u8::from(retained),
        take_events()
    );
    let expected = include_str!("../../../tools/oracle/tkernel_local_nontrivial_methods.expected");
    assert_eq!(
        Some(row.as_str()),
        expected
            .lines()
            .find(|line| line.starts_with(&format!("{label}:")))
    );
}

fn reallocate_case(label: &str, initial: usize, shrunk: usize, count: usize, copy: bool) {
    let mut array = filled(initial, 10);
    array.reallocate_with_move(shrunk, true, move_observed);
    let before = array.data_ptr();
    take_events();
    array.reallocate_with_move(count, copy, move_observed);
    check_row(label, &array, array.data_ptr() == before);
    array.deallocate();
    check_row(&format!("{label}-drop"), &array, false);
}

#[test]
fn nontrivial_inline_growth_preserves_old_values_with_either_copy_flag() {
    reallocate_case("inline-grow", 2, 2, 4, true);
    reallocate_case("inline-grow-no-copy", 2, 2, 4, false);
}

#[test]
fn nontrivial_inline_to_heap_moves_then_defaults_then_destroys() {
    reallocate_case("inline-heap", 2, 2, 6, true);
}

#[test]
fn nontrivial_heap_to_heap_moves_even_when_storage_could_be_retained() {
    reallocate_case("heap-heap", 6, 6, 8, true);
}

#[test]
fn nontrivial_retained_heap_can_return_to_inline() {
    reallocate_case("heap-inline", 6, 1, 3, true);
}

#[test]
fn nontrivial_inline_to_heap_without_copy_constructs_before_destroying() {
    reallocate_case("inline-heap-no-copy", 2, 2, 6, false);
}

#[test]
fn nontrivial_heap_to_heap_without_copy_constructs_before_destroying() {
    reallocate_case("heap-heap-no-copy", 6, 6, 8, false);
}

#[test]
fn nontrivial_heap_to_inline_without_copy_constructs_before_destroying() {
    reallocate_case("heap-inline-no-copy", 6, 1, 3, false);
}

#[test]
fn shrink_destroys_in_native_forward_order() {
    reallocate_case("shrink", 6, 6, 2, true);
}

#[test]
fn same_size_does_not_construct_move_or_destroy() {
    reallocate_case("same", 3, 3, 3, false);
}

fn construct_case(label: &str, count: usize) {
    let mut source = filled(count, 10);
    let before = source.data_ptr();
    take_events();
    let destination = ObservedArray::move_construct_with(&mut source, move_observed);
    check_row(label, &destination, destination.data_ptr() == before);
    check_row(
        &format!("{label}-source"),
        &source,
        source.data_ptr() == before,
    );
}

#[test]
fn construct_inline_invokes_moves_before_destroying_sources() {
    construct_case("construct-inline", 3);
}

#[test]
fn construct_heap_transfers_allocation_without_callbacks() {
    construct_case("construct-heap", 6);
}

fn assign_case(label: &str, initial: usize, count: usize) {
    let mut destination = filled(initial, 20);
    let mut source = filled(count, 10);
    let before = source.data_ptr();
    take_events();
    destination.move_from_with(&mut source, move_observed);
    check_row(label, &destination, destination.data_ptr() == before);
    check_row(
        &format!("{label}-source"),
        &source,
        source.data_ptr() == before,
    );
}

#[test]
fn assign_inline_to_inline_destroys_destination_before_moves() {
    assign_case("assign-inline-inline", 2, 3);
}

#[test]
fn assign_inline_to_heap_destroys_destination_before_moves() {
    assign_case("assign-heap-inline", 6, 3);
}

#[test]
fn assign_heap_to_inline_transfers_allocation_after_destination_drops() {
    assign_case("assign-inline-heap", 2, 6);
}

#[test]
fn assign_heap_to_heap_transfers_allocation_after_destination_drops() {
    assign_case("assign-heap-heap", 6, 7);
}

#[test]
fn destructor_visits_values_in_native_ascending_order() {
    let array = filled(3, 30);
    take_events();
    drop(array);
    assert_eq!(take_events(), "D30,D31,D32,");
}

#[test]
fn ordinary_shrink_and_allocate_use_the_same_forward_destruction() {
    let mut array = filled(6, 10);
    take_events();
    array.allocate(2);
    assert_eq!(take_events(), "D12,D13,D14,D15,");
    array.allocate(5);
    assert_eq!(take_events(), "C,C,C,C,C,D10,D11,");
}

#[test]
fn move_adapter_accepts_borrowed_values() {
    let owner = String::from("borrowed");
    let mut source = LocalArray::<&str, 4>::new(3);
    source[0] = &owner;
    let moved = LocalArray::move_construct_with(&mut source, std::mem::take);
    assert_eq!(moved[0], owner);
    assert!(source.is_empty());
}

#[derive(Default)]
struct PanicObserved {
    value: i32,
}

impl Drop for PanicObserved {
    fn drop(&mut self) {
        record(format!("D{},", self.value));
        if self.value == 11 && PANIC_ONCE.replace(false) {
            panic!("destructor value 11: expected non-panicking native destructor");
        }
    }
}

fn panicking_array(count: usize) -> LocalArray<PanicObserved, 4> {
    let mut array = LocalArray::<PanicObserved, 4>::new(count);
    for (index, item) in array.iter_mut().enumerate() {
        item.value = 10 + i32::try_from(index).unwrap();
    }
    take_events();
    PANIC_ONCE.set(true);
    array
}

#[test]
fn panicking_destructor_releases_each_inline_or_heap_value_once() {
    for count in [3, 6] {
        let array = panicking_array(count);
        assert!(std::panic::catch_unwind(|| drop(array)).is_err());
        let expected: String = (10..10 + count).map(|value| format!("D{value},")).collect();
        assert_eq!(take_events(), expected);
    }
}

#[test]
fn panicking_deallocate_leaves_no_live_values_to_drop_twice() {
    let mut array = panicking_array(6);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| array.deallocate()));
    assert!(result.is_err());
    assert!(array.is_empty());
    assert_eq!(take_events(), "D10,D11,D12,D13,D14,D15,");
    drop(array);
    assert!(take_events().is_empty());
}

#[test]
fn public_local_array_auto_traits_are_preserved() {
    fn thread_traits<T: Send + Sync + std::panic::UnwindSafe + std::panic::RefUnwindSafe>() {}
    fn unpin<T: Unpin>() {}
    thread_traits::<LocalArray<i32, 4>>();
    unpin::<LocalArray<i32, 4>>();
}

#[derive(Default)]
struct AddressObserved {
    original: *const AddressObserved,
}

impl Drop for AddressObserved {
    fn drop(&mut self) {
        if !self.original.is_null() {
            assert!(std::ptr::eq(self.original, self));
        }
    }
}

#[test]
fn removed_values_are_destroyed_in_place_in_inline_and_heap_buffers() {
    for count in [3, 6] {
        let mut array = LocalArray::<AddressObserved, 4>::new(count);
        for item in array.iter_mut() {
            item.original = std::ptr::from_ref(item);
        }
        array.reallocate(1, true);
        array.deallocate();
        assert!(array.is_empty());
    }
}

#[test]
fn nontrivial_relocation_destroys_moved_sources_in_place() {
    let mut array = LocalArray::<AddressObserved, 4>::new(3);
    for item in array.iter_mut() {
        item.original = std::ptr::from_ref(item);
    }
    array.reallocate_with_move(6, true, |_| AddressObserved::default());
    assert_eq!(array.len(), 6);
}
