use super::super::*;
fn actual_set(
    name: &str,
    index: i32,
    source: usize,
    failure: &'static str,
    moving: bool,
) -> String {
    let mut array = fixture_with(2, 4, failure);
    let original = &array[source] as *const BorrowedItem;
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        invoke_set(&mut array, index, source, moving).map(|value| value as *const BorrowedItem)
    }));
    let returned = match outcome {
        Ok(Ok(pointer)) => pointer == &array[index as usize],
        _ => {
            event("E".into());
            false
        }
    };
    let stable = original == &array[source];
    super::internal::observed_internal_row(name, &array, returned, stable)
}
fn invoke_set(
    array: &mut DynamicArray<BorrowedItem>,
    index: i32,
    source: usize,
    moving: bool,
) -> Result<&mut BorrowedItem, ArrayError> {
    if moving {
        if index % 2 == 1 && index > 0 {
            return array.set_value_moved_from_index_with_assignment(
                index as usize,
                source,
                super::moving::move_construct,
                move_assign,
            );
        }
        return array.set_value_moved_from_index_with_assignment(
            index,
            source,
            super::moving::move_construct,
            move_assign,
        );
    }
    if index % 2 == 1 && index > 0 {
        return array.set_value_from_index_with_assignment(
            index as usize,
            source,
            copy_construct,
            move_assign,
        );
    }
    array.set_value_from_index_with_assignment(index, source, copy_construct, move_assign)
}
fn expected_set(name: &str) -> &'static str {
    include_str!("../../../../tools/oracle/tkernel_dynamic_borrowed_methods.expected")
        .lines()
        .find(|row| row.split_whitespace().next() == Some(name))
        .unwrap()
}
fn matrix(moving: bool) {
    for index in 0..8 {
        for source in 0..4 {
            let name = format!("alias_set_m{}_i{index}_s{source}", usize::from(moving));
            assert_eq!(
                actual_set(&name, index, source, "", moving),
                expected_set(&name)
            );
        }
    }
}
#[test]
fn internal_set_copy_matrix() {
    matrix(false);
}
#[test]
fn internal_set_move_matrix() {
    matrix(true);
}
macro_rules! set_case {
    ($name:ident,$index:expr,$source:expr,$failure:expr,$moving:expr) => {
        #[test]
        fn $name() {
            assert_eq!(
                actual_set(stringify!($name), $index, $source, $failure, $moving),
                expected_set(stringify!($name))
            );
        }
    };
}
set_case!(set_alias_copy_ctor, 0, 1, "construct", false);
set_case!(set_alias_self_copy_ctor, 1, 1, "construct", false);
set_case!(set_alias_move_ctor, 0, 1, "move_construct", true);
set_case!(set_alias_self_move_ctor, 1, 1, "move_construct", true);
set_case!(set_alias_copy_assign, 0, 1, "move", false);
set_case!(set_alias_self_copy_assign, 1, 1, "move", false);
set_case!(set_alias_move_assign, 0, 1, "move", true);
set_case!(set_alias_self_move_assign, 1, 1, "move", true);
set_case!(set_alias_gap_copy_ctor, 7, 1, "construct", false);
set_case!(set_alias_gap_move_ctor, 7, 1, "move_construct", true);
set_case!(set_alias_gap_copy_default, 7, 1, "default", false);
set_case!(set_alias_gap_move_default, 7, 1, "default", true);
set_case!(set_alias_negative_copy, -1, 1, "", false);
set_case!(set_alias_negative_move, -1, 1, "", true);
#[test]
fn invalid_set_source_precedes_allocation_and_callbacks() {
    let mut array = fixture("");
    for source in [-1, 2, i32::MAX] {
        assert!(
            array
                .set_value_from_index_with_assignment(7, source, copy_construct, move_assign)
                .is_err()
        );
        assert!(
            array
                .set_value_moved_from_index_with_assignment(
                    7,
                    source,
                    super::moving::move_construct,
                    move_assign
                )
                .is_err()
        );
    }
    assert_eq!(array.size(), 2);
    assert_eq!(
        BORROWED_PROBE.with(|probe| probe.borrow().events.clone()),
        ""
    );
}
#[test]
fn internal_set_requires_no_clone_trait() {
    #[derive(Default)]
    struct NoClone(i32);
    let mut array = DynamicArray::with_increment(2);
    array.append(NoClone(12));
    array
        .set_value_from_index_with_assignment(0_i32, 0_usize, |v| NoClone(v.0), |a, b| a.0 = b.0)
        .unwrap();
    array
        .set_value_moved_from_index_with_assignment(3_usize, 0_i32, std::mem::take, std::mem::swap)
        .unwrap();
    assert_eq!(array.iter().map(|v| v.0).collect::<Vec<_>>(), [0, 0, 0, 12]);
}
