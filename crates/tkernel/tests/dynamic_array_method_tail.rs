//! Native signed overloads and construction order for segmented arrays.
use rustycad_tkernel::ncollection::{BaseAllocator, DynamicArray, OccAllocator};
use std::{
    cell::RefCell,
    panic::{AssertUnwindSafe, catch_unwind},
};
mod dynamic_array_method_tail {
    pub mod oracle;
    pub mod storage;
}
thread_local! {
    // Default has no parameters; isolate its injected test context per test thread.
    static PROBE: RefCell<ProbeContext> = RefCell::new(ProbeContext::default());
}
#[derive(Default)]
struct ProbeContext {
    defaults: usize,
    fail_default: usize,
    fail_copy: bool,
    copied: usize,
    allocator: Option<BaseAllocator>,
}
#[derive(Debug)]
struct TailItem {
    value: i32,
    nested: usize,
}
impl Default for TailItem {
    fn default() -> Self {
        PROBE.with(|state| {
            let mut state = state.borrow_mut();
            state.defaults += 1;
            assert_ne!(
                state.defaults, state.fail_default,
                "injected default failure"
            );
            Self {
                value: 0,
                nested: nested_address(&state),
            }
        })
    }
}
impl Clone for TailItem {
    fn clone(&self) -> Self {
        PROBE.with(|state| {
            let mut state = state.borrow_mut();
            state.copied += 1;
            assert!(!state.fail_copy, "injected copy failure");
            Self {
                value: self.value,
                nested: nested_address(&state),
            }
        })
    }
}
fn nested_address(context: &ProbeContext) -> usize {
    context
        .allocator
        .as_ref()
        .map_or(0, |a| a.allocate_raw(8).unwrap().as_ptr() as usize)
}
fn setup(fail_default: usize, allocator: Option<BaseAllocator>) {
    PROBE.with(|state| {
        *state.borrow_mut() = ProbeContext {
            fail_default,
            allocator,
            ..ProbeContext::default()
        }
    });
}
#[test]
fn appended_reserves_storage_before_default_construction() {
    let allocator = BaseAllocator::incremental(4096);
    setup(0, Some(allocator.clone()));
    let mut array = DynamicArray::<TailItem>::with_allocator(2, Some(allocator));
    let item = array.appended();
    assert!((item as *const TailItem as usize) < item.nested);
    assert_eq!(array.size(), 1);
}
#[test]
fn borrowed_append_reserves_storage_before_copying() {
    let allocator = BaseAllocator::incremental(4096);
    setup(0, Some(allocator.clone()));
    let mut array = DynamicArray::with_allocator(2, Some(allocator));
    let item = array.append_ref(&TailItem {
        value: 7,
        nested: 0,
    });
    assert_eq!(item.value, 7);
    assert!((item as *const TailItem as usize) < item.nested);
}
#[test]
fn failed_appended_preserves_empty_size_and_can_retry() {
    setup(1, None);
    let mut array = DynamicArray::<TailItem>::with_increment(2);
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            array.appended();
        }))
        .is_err()
    );
    assert_eq!(array.size(), 0);
    setup(0, None);
    array.appended();
    assert_eq!(array.size(), 1);
}
#[test]
fn owned_set_value_commits_size_only_after_all_gap_defaults() {
    setup(2, None);
    let mut array = DynamicArray::with_increment(2);
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            array
                .set_value(
                    3,
                    TailItem {
                        value: 7,
                        nested: 0,
                    },
                )
                .unwrap();
        }))
        .is_err()
    );
    assert_eq!(array.size(), 0);
    setup(0, None);
    array
        .set_value(
            3,
            TailItem {
                value: 7,
                nested: 0,
            },
        )
        .unwrap();
    assert_eq!(array.size(), 4);
    assert_eq!(array.last().unwrap().value, 7);
}
#[test]
fn borrowed_set_value_commits_after_requested_copy() {
    setup(0, None);
    PROBE.with(|state| state.borrow_mut().fail_copy = true);
    let mut array = DynamicArray::with_increment(2);
    let item = TailItem {
        value: 7,
        nested: 0,
    };
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            array.set_value_ref(3, &item).unwrap();
        }))
        .is_err()
    );
    assert_eq!(array.size(), 0);
    assert_eq!(PROBE.with(|state| state.borrow().defaults), 3);
    setup(0, None);
    array.set_value_ref(3, &item).unwrap();
    assert_eq!(array.size(), 4);
    assert_eq!(array.last().unwrap().value, 7);
}
#[test]
fn borrowed_set_value_allocates_all_blocks_before_gap_construction() {
    let allocator = BaseAllocator::incremental(4096);
    setup(0, Some(allocator.clone()));
    let mut array = DynamicArray::with_allocator(2, Some(allocator));
    let target = array
        .set_value_ref(
            4,
            &TailItem {
                value: 9,
                nested: 0,
            },
        )
        .unwrap() as *const TailItem as usize;
    assert!(target < array[0].nested);
    assert_eq!(PROBE.with(|state| state.borrow().copied), 1);
}
#[test]
fn failed_borrowed_replacement_preserves_existing_element() {
    setup(0, None);
    let mut array = DynamicArray::with_increment(2);
    array.append(TailItem {
        value: 5,
        nested: 0,
    });
    PROBE.with(|state| state.borrow_mut().fail_copy = true);
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            array
                .set_value_ref(
                    0,
                    &TailItem {
                        value: 9,
                        nested: 0,
                    },
                )
                .unwrap();
        }))
        .is_err()
    );
    assert_eq!(array[0].value, 5);
    assert_eq!(array.size(), 1);
}
#[test]
fn negative_set_value_rejects_before_copying_or_default_construction() {
    setup(0, None);
    let mut array = DynamicArray::with_increment(2);
    assert!(
        array
            .set_value_ref(
                -1,
                &TailItem {
                    value: 2,
                    nested: 0
                }
            )
            .is_err()
    );
    assert_eq!(
        PROBE.with(|state| (state.borrow().defaults, state.borrow().copied)),
        (0, 0)
    );
}
