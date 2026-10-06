use super::super::*;
#[test]
fn append_borrowed_needs_neither_clone_nor_default() {
    struct ConstructorOnly(i32);
    let source = ConstructorOnly(41);
    let mut array = DynamicArray::with_increment(1);
    let stored = array.append_ref_with_constructor(&source, |item| ConstructorOnly(item.0));
    assert_eq!(stored.0, 41);
    assert_eq!(source.0, 41);
}
#[test]
fn constructor_panic_retains_block_for_borrowed_retry() {
    let mut array = fixture("construct");
    let source = BorrowedItem { value: 99 };
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            array.append_ref_with_constructor(&source, copy_construct);
        }))
        .is_err()
    );
    assert_eq!(array.size(), 2);
    BORROWED_PROBE.with(|probe| probe.borrow_mut().failure = "");
    array.append_ref_with_constructor(&source, copy_construct);
    assert_eq!(
        BORROWED_PROBE.with(|probe| probe.borrow().events.clone()),
        "A;C99;C99;"
    );
    assert_eq!(array.size(), 3);
}
#[test]
fn gap_construction_never_calls_existing_slot_assignment() {
    let mut array = DynamicArray::<i32>::with_increment(2);
    array
        .set_value_ref_with_assignment(
            3,
            &9,
            |value| *value,
            |_, _| panic!("new index 3: expected no assignment"),
        )
        .unwrap();
    assert_eq!(array.iter().copied().collect::<Vec<_>>(), [0, 0, 0, 9]);
}
#[test]
fn borrowed_set_preserves_initialized_tail_after_constructor_panic() {
    let mut array = DynamicArray::<i32>::with_increment(4);
    array.append(10);
    array[3] = 77;
    let failed = catch_unwind(AssertUnwindSafe(|| fail_tail_copy(&mut array)));
    assert!(failed.is_err());
    assert_eq!(array.size(), 1);
    assert_eq!(array.value(3), Ok(&77));
    array
        .set_value_ref_with_assignment(3, &9, |value| *value, |to, from| *to = *from)
        .unwrap();
    assert_eq!(array.iter().copied().collect::<Vec<_>>(), [10, 0, 0, 9]);
}

fn fail_tail_copy(array: &mut DynamicArray<i32>) {
    array
        .set_value_ref_with_assignment(
            3,
            &9,
            |_| panic!("source 9: expected injected failure"),
            |to, from| *to = *from,
        )
        .unwrap();
}
