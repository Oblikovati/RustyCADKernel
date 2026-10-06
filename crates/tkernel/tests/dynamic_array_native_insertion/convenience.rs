use super::super::*;
#[test]
fn borrowed_before_uses_copy_assignment_after_default_staging() {
    let mut array = insertion_fixture();
    let source = InsertionItem { value: 99 };
    let inserted = array.insert_before_ref(1_usize, &source).unwrap() as *const InsertionItem;
    assert_eq!(observed_events(), "A;D;C(0,99);");
    assert_eq!(observed_values(&array), [10, 99, 20, 30, 40]);
    assert_eq!(inserted, &array[1] as *const InsertionItem);
    assert_eq!(source.value, 99);
}
#[test]
fn borrowed_after_last_uses_copy_assignment_without_shift() {
    let mut array = insertion_fixture();
    array
        .insert_after_ref(3_i32, &InsertionItem { value: 99 })
        .unwrap();
    assert_eq!(observed_events(), "A;D;C(0,99);");
    assert_eq!(observed_values(&array), [10, 20, 30, 40, 99]);
}
#[test]
fn owned_before_and_after_construct_one_default_each() {
    let mut array = insertion_fixture();
    array
        .insert_before(0_i32, InsertionItem { value: 5 })
        .unwrap();
    array
        .insert_after(4_usize, InsertionItem { value: 50 })
        .unwrap();
    assert_eq!(observed_events(), "A;D;D;");
    assert_eq!(observed_values(&array), [5, 10, 20, 30, 40, 50]);
}
#[test]
fn default_panic_retains_allocated_block_without_publishing_size() {
    let mut array = insertion_fixture();
    INSERTION_PROBE.with(|probe| probe.borrow_mut().fail_default = true);
    let failed = catch_unwind(AssertUnwindSafe(|| {
        array.insert_before(1, InsertionItem { value: 99 }).unwrap();
    }));
    assert!(failed.is_err());
    assert_eq!(array.size(), 4);
    assert_eq!(observed_values(&array), [10, 20, 30, 40]);
    INSERTION_PROBE.with(|probe| probe.borrow_mut().fail_default = false);
    array
        .insert_after_ref(3, &InsertionItem { value: 99 })
        .unwrap();
    assert_eq!(observed_events(), "A;D;D;C(0,99);");
}
#[test]
fn borrowed_copy_panic_keeps_published_size_and_shifted_values() {
    let mut array = insertion_fixture();
    INSERTION_PROBE.with(|probe| probe.borrow_mut().fail_copy = true);
    let source = InsertionItem { value: 99 };
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            array.insert_before_ref(1, &source).unwrap();
        }))
        .is_err()
    );
    assert_eq!(array.size(), 5);
    assert_eq!(observed_values(&array), [10, 0, 20, 30, 40]);
    assert_eq!(observed_events(), "A;D;C(0,99);");
}
#[test]
fn borrowed_bounds_fail_before_default_or_assignment() {
    let mut array = insertion_fixture();
    let source = InsertionItem { value: 99 };
    assert!(array.insert_before_ref(-1_i32, &source).is_err());
    assert!(array.insert_after_ref(4_usize, &source).is_err());
    assert!(array.insert_before_ref(usize::MAX, &source).is_err());
    assert_eq!(observed_events(), "");
    assert_eq!(observed_values(&array), [10, 20, 30, 40]);
}
#[test]
fn custom_hook_receives_distinct_initialized_slots_and_returns_inserted_reference() {
    let mut array = insertion_fixture();
    let inserted = array
        .insert_after_with_assignment(0_usize, move_assignment, |target| {
            target.value = 99;
        })
        .unwrap() as *const InsertionItem;
    assert_eq!(observed_values(&array), [10, 99, 20, 30, 40]);
    assert_eq!(inserted, &array[1] as *const InsertionItem);
    assert_eq!(observed_events(), "A;D;M(0,40);M(-7,30);M(-7,20);");
}
fn heap_fixture(increment: usize) -> DynamicArray<i32> {
    let mut array = DynamicArray::with_increment(increment);
    for value in [10, 20, 30, 40] {
        array.append(value);
    }
    array
}
fn heap_move_assignment(target: &mut i32, source: &mut i32) {
    *target = *source;
    *source = -7;
}
fn fail_second_heap_move(calls: &mut usize, target: &mut i32, source: &mut i32) {
    *calls += 1;
    assert_ne!(
        *calls, 2,
        "move call 2: expected injected assignment failure"
    );
    heap_move_assignment(target, source);
}
fn heap_failure_prefix(increment: usize) {
    let mut array = heap_fixture(increment);
    let mut calls = 0;
    let shift =
        |target: &mut i32, source: &mut i32| fail_second_heap_move(&mut calls, target, source);
    let failure = catch_unwind(AssertUnwindSafe(|| {
        array
            .insert_before_with_assignment(1, shift, |target| *target = 99)
            .unwrap();
    }));
    assert!(failure.is_err());
    let values = array.iter().copied().collect::<Vec<_>>();
    assert_eq!(values, [10, 20, 30, -7, 40]);
    assert_eq!(array.size(), 5);
}
#[test]
fn strict_heap_assignment_stages_preserve_initialized_slots() {
    for increment in [1, 2, 4] {
        let mut array = heap_fixture(increment);
        array
            .insert_before_with_assignment(1, heap_move_assignment, |target| *target = 99)
            .unwrap();
        assert_eq!(
            array.iter().copied().collect::<Vec<_>>(),
            [10, 99, 20, 30, 40]
        );
        heap_failure_prefix(increment);
    }
}
