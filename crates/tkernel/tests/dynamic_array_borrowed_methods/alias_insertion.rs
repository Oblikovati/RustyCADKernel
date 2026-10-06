use super::super::*;
fn copy_assign_self(slot: &mut BorrowedItem) {
    event("Q".into());
    let failure = event(format!("P{},{}", slot.value, slot.value));
    assert_ne!(
        failure, "assign",
        "self copy value{}: expected success",
        slot.value
    );
}
fn move_assign_self(slot: &mut BorrowedItem) {
    event("S".into());
    let failure = event(format!("M{},{}", slot.value, slot.value));
    slot.value = -7;
    finish_move(failure, slot.value);
    assert_ne!(failure, "self", "self move value-7: expected success");
}
fn invoke_insert<I: TryInto<usize> + Copy + std::fmt::Debug>(
    array: &mut DynamicArray<BorrowedItem>,
    index: I,
    source: usize,
    after: bool,
    moving: bool,
) -> Result<*const BorrowedItem, ArrayError> {
    let result = if moving {
        invoke_moved(array, index, source, after)
    } else {
        invoke_copied(array, index, source, after)
    };
    result.map(|slot| slot as *const BorrowedItem)
}
fn invoke_copied<I: TryInto<usize> + Copy + std::fmt::Debug>(
    array: &mut DynamicArray<BorrowedItem>,
    index: I,
    source: usize,
    after: bool,
) -> Result<&mut BorrowedItem, ArrayError> {
    let (shift, copy, same) = (move_assign, copy_assign, copy_assign_self);
    if after {
        return array.insert_after_from_index_with_assignment(index, source, shift, copy, same);
    }
    array.insert_before_from_index_with_assignment(index, source, shift, copy, same)
}
fn invoke_moved<I: TryInto<usize> + Copy + std::fmt::Debug>(
    array: &mut DynamicArray<BorrowedItem>,
    index: I,
    source: usize,
    after: bool,
) -> Result<&mut BorrowedItem, ArrayError> {
    let (shift, transfer, same) = (move_assign, move_assign, move_assign_self);
    if after {
        return array
            .insert_after_moved_from_index_with_assignment(index, source, shift, transfer, same);
    }
    array.insert_before_moved_from_index_with_assignment(index, source, shift, transfer, same)
}
fn returned_slot(
    array: &mut DynamicArray<BorrowedItem>,
    index: i32,
    source: usize,
    after: bool,
    moving: bool,
    wide: bool,
) -> bool {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        if wide {
            invoke_insert(array, index as usize, source, after, moving)
        } else {
            invoke_insert(array, index, source, after, moving)
        }
    }));
    if let Ok(Ok(slot)) = outcome {
        return slot == &array[index as usize + usize::from(after)];
    }
    event("E".into());
    false
}
fn actual_alias_row(
    name: &str,
    index: i32,
    source: usize,
    after: bool,
    moving: bool,
    failure: &'static str,
    wide: bool,
) -> String {
    let mut array = fixture_with(2, 4, failure);
    if failure == "final_move" {
        array[source].value = 99;
    }
    let original = &array[source] as *const BorrowedItem;
    let returned = returned_slot(&mut array, index, source, after, moving, wide);
    let stable = original == &array[source];
    super::internal::observed_internal_row(name, &array, returned, stable)
}
fn assert_native_row(name: &str, actual: &str) {
    let prefix = format!("{name} ");
    let expected =
        include_str!("../../../../tools/oracle/tkernel_dynamic_borrowed_methods.expected")
            .lines()
            .find(|row| row.starts_with(&prefix))
            .unwrap();
    assert_eq!(actual, expected);
}
fn check_matrix(after: bool, moving: bool) {
    for (index, source) in (0..4).flat_map(|index| (0..4).map(move |source| (index, source))) {
        let name = format!(
            "alias_a{}_m{}_i{index}_s{source}",
            usize::from(after),
            usize::from(moving)
        );
        let row = actual_alias_row(&name, index, source, after, moving, "", index % 2 == 1);
        assert_native_row(&name, &row);
    }
}
#[test]
fn copy_before_alias_matrix() {
    check_matrix(false, false);
}
#[test]
fn copy_after_alias_matrix() {
    check_matrix(true, false);
}
#[test]
fn move_before_alias_matrix() {
    check_matrix(false, true);
}
#[test]
fn move_after_alias_matrix() {
    check_matrix(true, true);
}
macro_rules! alias_failure {
    ($name:ident,$index:expr,$source:expr,$after:expr,$moving:expr,$failure:expr,$wide:expr) => {
        #[test]
        fn $name() {
            let row = actual_alias_row(
                stringify!($name),
                $index,
                $source,
                $after,
                $moving,
                $failure,
                $wide,
            );
            assert_native_row(stringify!($name), &row);
        }
    };
}
alias_failure!(alias_copy_before_throw, 0, 1, false, false, "assign", false);
alias_failure!(alias_copy_self_throw, 0, 0, false, false, "assign", true);
alias_failure!(alias_copy_after_throw, 1, 0, true, false, "assign", true);
alias_failure!(
    alias_move_before_shift_throw,
    0,
    3,
    false,
    true,
    "move",
    false
);
alias_failure!(alias_move_after_shift_throw, 0, 1, true, true, "move", true);
alias_failure!(
    alias_move_before_self_throw,
    0,
    0,
    false,
    true,
    "self",
    false
);
alias_failure!(alias_move_after_self_throw, 0, 1, true, true, "self", true);
alias_failure!(
    alias_move_final_throw,
    2,
    0,
    false,
    true,
    "final_move",
    true
);
alias_failure!(
    alias_default_before_throw,
    0,
    1,
    false,
    false,
    "default",
    false
);
alias_failure!(alias_default_after_throw, 0, 1, true, true, "default", true);
alias_failure!(alias_before_negative, -1, 0, false, false, "", false);
alias_failure!(alias_after_end, 4, 0, true, true, "", true);
#[test]
fn invalid_source_does_not_run_default_or_assignment_callbacks() {
    let mut array = fixture_with(2, 4, "");
    for source in [4_usize, usize::MAX] {
        assert!(invoke_insert(&mut array, 0, source, false, false).is_err());
        assert!(invoke_insert(&mut array, 0, source, true, true).is_err());
    }
    assert_eq!(array.size(), 4);
    assert_eq!(
        BORROWED_PROBE.with(|probe| probe.borrow().events.clone()),
        ""
    );
}
