use super::super::*;
fn move_construct(source: &mut BorrowedItem) -> BorrowedItem {
    let failure = event(format!("K{}", source.value));
    let value = source.value;
    source.value = -7;
    assert_ne!(
        failure, "move_construct",
        "move source99: expected successful construction"
    );
    BorrowedItem { value }
}
pub(super) fn operate_moved<'a, I: TryInto<usize> + Copy + std::fmt::Debug>(
    array: &'a mut DynamicArray<BorrowedItem>,
    source: &mut BorrowedItem,
    operation: u8,
    index: I,
) -> Result<&'a mut BorrowedItem, ArrayError> {
    match operation {
        0 => Ok(array.append_moved_with_constructor(source, move_construct)),
        1 => array.set_value_moved_with_assignment(index, source, move_construct, move_assign),
        2 => array.insert_before_moved_with_assignment(index, source, move_assign, move_assign),
        _ => array.insert_after_moved_with_assignment(index, source, move_assign, move_assign),
    }
}
macro_rules! moved_case {
    ($name:ident,$operation:expr,$index:expr,$failure:expr,$wide:expr) => {
        #[test]
        fn $name() {
            let expected =
                include_str!("../../../../tools/oracle/tkernel_dynamic_borrowed_methods.expected")
                    .lines()
                    .find(|row| row.starts_with(concat!(stringify!($name), " ")))
                    .unwrap();
            assert_eq!(
                super::oracle::actual_row(
                    stringify!($name),
                    $operation,
                    $index,
                    $failure,
                    $wide,
                    true
                ),
                expected
            );
        }
    };
}
moved_case!(moved_append, 0, 0, "", false);
moved_case!(moved_append_ctor_fail, 0, 0, "move_construct", true);
moved_case!(moved_set_existing, 1, 0, "", false);
moved_case!(moved_set_existing_wide, 1, 1, "", true);
moved_case!(moved_set_ctor_fail, 1, 0, "move_construct", false);
moved_case!(moved_set_assign_fail, 1, 0, "move", true);
moved_case!(moved_set_gap, 1, 4, "", false);
moved_case!(moved_set_gap_wide, 1, 4, "", true);
moved_case!(moved_set_gap_ctor_fail, 1, 4, "move_construct", false);
moved_case!(moved_set_default_fail, 1, 4, "default", true);
moved_case!(moved_set_negative, 1, -1, "", false);
moved_case!(moved_before, 2, 0, "", false);
moved_case!(moved_before_wide, 2, 1, "", true);
moved_case!(moved_after, 3, 0, "", false);
moved_case!(moved_after_wide, 3, 1, "", true);
moved_case!(moved_before_final_fail, 2, 0, "final_move", false);
moved_case!(moved_after_shift_fail, 3, 0, "move", true);
moved_case!(moved_before_default_fail, 2, 0, "default", false);
moved_case!(moved_before_negative, 2, -1, "", false);
moved_case!(moved_after_end, 3, 2, "", true);
#[test]
fn moved_append_has_no_clone_or_default_requirement() {
    struct MoveOnly(i32);
    let mut donor = MoveOnly(42);
    let mut array = DynamicArray::with_increment(1);
    let value = array.append_moved_with_constructor(&mut donor, |source| {
        let value = source.0;
        source.0 = -7;
        MoveOnly(value)
    });
    assert_eq!(value.0, 42);
    assert_eq!(donor.0, -7);
}
