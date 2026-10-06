use super::super::*;
fn append_internal(array: &mut DynamicArray<BorrowedItem>, source: usize, moving: bool) -> bool {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        let result = if moving {
            array.append_moved_from_index_with_constructor(source, super::moving::move_construct)
        } else {
            array.append_from_index_with_constructor(source, copy_construct)
        };
        result.map(|value| value as *const BorrowedItem)
    }));
    if let Ok(Ok(pointer)) = outcome {
        return pointer == array.last().unwrap();
    }
    event("E".into());
    false
}
fn actual_internal(
    name: &str,
    increment: usize,
    count: usize,
    source: usize,
    failure: &'static str,
    moving: bool,
    retry: bool,
) -> String {
    let mut array = fixture_with(increment, count, failure);
    let original = &array[source] as *const BorrowedItem;
    let mut returned = append_internal(&mut array, source, moving);
    if retry {
        BORROWED_PROBE.with(|probe| probe.borrow_mut().failure = "");
        returned = append_internal(&mut array, source, moving);
    }
    let stable = original == &array[source];
    observed_internal_row(name, &array, returned, stable)
}
pub(super) fn observed_internal_row(
    name: &str,
    array: &DynamicArray<BorrowedItem>,
    returned: bool,
    stable: bool,
) -> String {
    let events = BORROWED_PROBE.with(|probe| probe.borrow().events.clone());
    format!(
        "{name} {events} {} {} {}{}",
        array.size(),
        usize::from(returned),
        usize::from(stable),
        internal_values(array)
    )
}
fn internal_values(array: &DynamicArray<BorrowedItem>) -> String {
    let mut values = String::new();
    for item in array.iter() {
        write!(values, " {}", item.value).unwrap();
    }
    values
}
macro_rules! internal_case {
    ($name:ident,$increment:expr,$count:expr,$source:expr,$failure:expr,$moving:expr,$retry:expr) => {
        #[test]
        fn $name() {
            let expected =
                include_str!("../../../../tools/oracle/tkernel_dynamic_borrowed_methods.expected")
                    .lines()
                    .find(|row| row.starts_with(concat!(stringify!($name), " ")))
                    .unwrap();
            assert_eq!(
                actual_internal(
                    stringify!($name),
                    $increment,
                    $count,
                    $source,
                    $failure,
                    $moving,
                    $retry
                ),
                expected
            );
        }
    };
}
internal_case!(self_copy_same, 4, 1, 0, "", false, false);
internal_case!(self_move_same, 4, 1, 0, "", true, false);
internal_case!(self_copy_new_block, 2, 2, 0, "", false, false);
internal_case!(self_move_new_block, 2, 2, 1, "", true, false);
internal_case!(self_copy_table_growth, 2, 4, 0, "", false, false);
internal_case!(self_move_table_growth, 2, 4, 3, "", true, false);
internal_case!(self_copy_later_block, 2, 5, 3, "", false, false);
internal_case!(self_move_later_block, 2, 5, 3, "", true, false);
internal_case!(self_copy_same_throw, 4, 1, 0, "construct", false, false);
internal_case!(self_move_same_throw, 4, 1, 0, "move_construct", true, false);
internal_case!(self_copy_table_throw, 2, 4, 0, "construct", false, false);
internal_case!(
    self_move_table_throw,
    2,
    4,
    3,
    "move_construct",
    true,
    false
);
internal_case!(self_copy_retry, 2, 4, 0, "construct", false, true);
internal_case!(self_move_retry, 2, 4, 3, "move_construct", true, true);
#[test]
fn invalid_internal_source_is_checked_before_allocations_or_callbacks() {
    let mut array = fixture("");
    for index in [-1_i64, 2, i64::MAX] {
        assert_invalid_source(&mut array, index);
    }
    assert!(
        array
            .append_from_index_with_constructor(usize::MAX, copy_construct)
            .is_err()
    );
    assert_eq!(array.size(), 2);
    assert_eq!(
        BORROWED_PROBE.with(|probe| probe.borrow().events.clone()),
        ""
    );
}
fn assert_invalid_source(array: &mut DynamicArray<BorrowedItem>, index: i64) {
    assert!(
        array
            .append_from_index_with_constructor(index, copy_construct)
            .is_err()
    );
    assert!(
        array
            .append_moved_from_index_with_constructor(index, super::moving::move_construct)
            .is_err()
    );
}
#[test]
fn empty_internal_source_does_not_allocate_or_construct() {
    let mut array = fixture_with(2, 0, "");
    assert_invalid_source(&mut array, 0);
    assert_eq!(array.storage_extension().block_count(), 0);
    assert_eq!(
        BORROWED_PROBE.with(|probe| probe.borrow().events.clone()),
        ""
    );
}
#[test]
fn internal_append_does_not_require_clone_or_default() {
    struct OwnItem(i32);
    let mut array = DynamicArray::with_increment(1);
    array.append(OwnItem(42));
    array
        .append_from_index_with_constructor(0_i32, |item| OwnItem(item.0))
        .unwrap();
    array
        .append_moved_from_index_with_constructor(1_usize, |item| {
            let value = item.0;
            item.0 = -7;
            OwnItem(value)
        })
        .unwrap();
    assert_eq!(
        array.iter().map(|item| item.0).collect::<Vec<_>>(),
        [42, -7, 42]
    );
}
