use super::fixture::{TraceLog, caught};
use rustycad_tkernel::ncollection::SparseArray;
#[test]
fn rust_clone_from_builds_replacement_before_destroying_target() {
    let trace = TraceLog::new();
    let source = trace.array(4, &[(1, 11)]);
    let mut target = trace.array(4, &[(1, 1), (64, 64)]);
    trace.clear();
    target.clone_from(&source);
    assert_eq!(trace.events(), "C11,D1,D64,");
    assert_eq!((target.len(), target.capacity()), (1, 16));
}
#[test]
fn rust_clone_failure_cleans_new_items_and_preserves_destination() {
    let trace = TraceLog::new();
    let source = trace.array(4, &[(1, 11), (2, 22)]);
    let mut target = trace.array(4, &[(64, 64)]);
    trace.clear();
    trace.reject_create.set(22);
    assert!(caught(|| target.clone_from(&source)));
    assert_eq!(trace.events(), "C11,C22,D11,");
    assert_eq!(
        (target.len(), target.capacity(), target[64].value),
        (1, 128, 64)
    );
}
#[test]
fn rust_clone_retains_empty_allocated_blocks_and_capacity() {
    let trace = TraceLog::new();
    let mut source = SparseArray::new(4);
    trace.reject_create.set(99);
    assert!(caught(|| {
        source.set_value_ref(33, &trace.item(99));
    }));
    let copy = source.clone();
    assert_eq!((copy.len(), copy.capacity()), (0, 64));
    assert_eq!(
        (copy.cursor().index(), copy.legacy_cursor().index()),
        (64, 68)
    );
}
#[test]
fn rust_clone_retains_table_when_all_blocks_have_been_removed() {
    let mut source = SparseArray::new(4);
    source.set_value(33, 99);
    source.unset_value(33);
    let copy = source.clone();
    assert_eq!(
        (copy.len(), copy.capacity(), copy.legacy_cursor().index()),
        (0, 64, 64)
    );
}
#[test]
fn rust_unset_destructor_panic_finishes_counters_and_frees_last_block() {
    let trace = TraceLog::new();
    let mut array = trace.array(4, &[(1, 11)]);
    trace.clear();
    trace.reject_drop.set(11);
    assert!(caught(|| {
        array.unset_value(1);
    }));
    assert_eq!((array.len(), array.has_value(1)), (0, false));
    assert_eq!(array.legacy_cursor().index(), array.capacity());
    array.clear();
    assert_eq!(trace.events(), "D11,");
}
#[test]
fn rust_clear_destructor_panic_drains_every_item_once() {
    let trace = TraceLog::new();
    let mut array = trace.array(4, &[(0, 10), (1, 11), (5, 15)]);
    trace.clear();
    trace.reject_drop.set(10);
    assert!(caught(|| array.clear()));
    assert_eq!(trace.events(), "D10,D11,D15,");
    assert_eq!((array.len(), array.capacity()), (0, 0));
    array.set_value(0, trace.item(20));
    assert_eq!(array[0].value, 20);
}
#[test]
fn rust_owner_drop_destructor_panic_never_destroys_an_item_twice() {
    let trace = TraceLog::new();
    let array = trace.array(4, &[(0, 10), (1, 11), (5, 15)]);
    trace.clear();
    trace.reject_drop.set(10);
    assert!(caught(|| drop(array)));
    assert_eq!(trace.events(), "D10,D11,D15,");
}
#[test]
fn rust_assignment_destructor_panic_completes_native_transitions() {
    let trace = TraceLog::new();
    let source = trace.array(4, &[(1, 11)]);
    let mut target = trace.array(4, &[(0, 10), (1, 1), (5, 15)]);
    trace.clear();
    trace.reject_drop.set(10);
    assert!(caught(|| {
        target.assign(&source);
    }));
    assert_eq!(trace.events(), "D10,A1>11,D15,");
    assert_eq!(
        (target.len(), target.has_value(0), target[1].value),
        (1, false, 11)
    );
}
#[test]
fn rust_assignment_drop_then_clone_panic_preserves_live_items() {
    let trace = TraceLog::new();
    let source = trace.array(4, &[(1, 11)]);
    let mut target = trace.array(4, &[(0, 10), (5, 15)]);
    trace.clear();
    trace.reject_drop.set(10);
    trace.reject_create.set(11);
    assert!(caught(|| {
        target.assign(&source);
    }));
    assert_eq!(trace.events(), "D10,C11,");
    assert_eq!(
        (target.len(), target.has_value(0), target.has_value(1)),
        (1, false, false)
    );
    target.clear();
    assert_eq!(trace.events(), "D10,C11,D15,");
}
#[test]
fn rust_owned_replacement_drop_panic_leaves_new_value_live() {
    let trace = TraceLog::new();
    let mut target = trace.array(4, &[(0, 10)]);
    trace.clear();
    trace.reject_drop.set(10);
    assert!(caught(|| {
        target.set_value(0, trace.item(20));
    }));
    assert_eq!((target.len(), target[0].value), (1, 20));
    target.clear();
    assert_eq!(trace.events(), "D10,D20,");
}
#[derive(Clone, Debug, PartialEq)]
#[repr(align(128))]
struct AlignedSparseItem(i32);
#[test]
fn rust_overaligned_values_keep_stable_aligned_addresses() {
    let mut array = SparseArray::new(4);
    let address = std::ptr::from_mut(array.set_value(1, AlignedSparseItem(1)));
    array.set_value(4096, AlignedSparseItem(2));
    array.set_value_ref(1, &AlignedSparseItem(3));
    assert_eq!(address, std::ptr::from_mut(array.change_value(1).unwrap()));
    assert_eq!(address.addr() % 128, 0);
    assert_eq!(array.clone()[1], AlignedSparseItem(3));
}
#[test]
fn rust_mutable_iteration_can_retain_disjoint_references() {
    let mut array = SparseArray::new(4);
    array.set_value(0, AlignedSparseItem(1));
    array.set_value(5, AlignedSparseItem(2));
    let mut entries = array.entries_mut();
    let (first_key, first) = entries.next().unwrap();
    let (second_key, second) = entries.next().unwrap();
    first.0 = 11;
    second.0 = 22;
    assert_eq!((first_key, second_key, first.0, second.0), (0, 5, 11, 22));
    assert!(entries.next().is_none());
}
thread_local! { static SPARSE_ZST_DROPS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
#[derive(Clone)]
#[repr(align(8))]
struct SparseZeroSized;
impl Drop for SparseZeroSized {
    fn drop(&mut self) {
        SPARSE_ZST_DROPS.with(|count| count.set(count.get() + 1));
    }
}
#[test]
fn rust_zero_sized_items_have_aligned_slots_and_individual_destruction() {
    SPARSE_ZST_DROPS.with(|count| count.set(0));
    let mut array = SparseArray::new(4);
    array.set_value(0, SparseZeroSized);
    array.set_value(1, SparseZeroSized);
    assert!(
        array
            .iter()
            .all(|value| std::ptr::from_ref(value).addr() % 8 == 0)
    );
    let mut entries = array.iter_mut();
    let first = entries.next().unwrap();
    let second = entries.next().unwrap();
    assert_ne!(std::ptr::from_mut(first), std::ptr::from_mut(second));
    drop(entries);
    array.clear();
    SPARSE_ZST_DROPS.with(|count| assert_eq!(count.get(), 2));
}
#[test]
fn rust_overflowing_safe_inputs_panic_before_unsafe_allocation() {
    let mut huge_block = SparseArray::new(usize::MAX);
    assert!(caught(|| {
        huge_block.set_value(0, 1u64);
    }));
    let mut huge_index = SparseArray::new(4);
    assert!(caught(|| {
        huge_index.set_value(usize::MAX, 1u64);
    }));
    assert_eq!((huge_block.len(), huge_index.len()), (0, 0));
}
#[test]
fn rust_plain_values_use_native_inline_stride() {
    let mut array = SparseArray::new(4);
    array.set_value(0, 10u32);
    array.set_value(1, 11u32);
    let first = std::ptr::from_ref(array.value(0).unwrap());
    let second = std::ptr::from_ref(array.value(1).unwrap());
    assert_eq!(second.addr() - first.addr(), size_of::<u32>());
}
#[derive(Clone)]
#[repr(align(128))]
struct AlignedDropping(super::fixture::TraceItem);
#[test]
fn rust_overaligned_destructor_panics_free_indirect_allocations() {
    let trace = TraceLog::new();
    let mut array = SparseArray::new(4);
    array.set_value(0, AlignedDropping(trace.item(10)));
    array.set_value(1, AlignedDropping(trace.item(11)));
    assert_eq!(array[1].0.value, 11);
    trace.clear();
    trace.reject_drop.set(10);
    assert!(caught(|| array.clear()));
    assert_eq!(trace.events(), "D10,D11,");
    assert_eq!((array.len(), array.capacity()), (0, 0));
}
struct SecondarySparsePanic;
impl Drop for SecondarySparsePanic {
    fn drop(&mut self) {
        panic!("secondary panic payload destruction");
    }
}
struct RepeatedSparseDropPanic(usize);
impl Drop for RepeatedSparseDropPanic {
    fn drop(&mut self) {
        if self.0 == 0 {
            std::panic::panic_any(0usize);
        }
        std::panic::panic_any(SecondarySparsePanic);
    }
}
#[test]
fn rust_multiple_drop_panics_and_panicking_payloads_are_drained() {
    let mut array = SparseArray::new(4);
    array.set_value(0, RepeatedSparseDropPanic(0));
    array.set_value(1, RepeatedSparseDropPanic(1));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| array.clear()));
    assert_eq!(*result.unwrap_err().downcast::<usize>().unwrap(), 0);
    assert_eq!((array.len(), array.capacity()), (0, 0));
}
