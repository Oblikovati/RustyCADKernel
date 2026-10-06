use super::super::*;
fn actual_row(name: &str, operation: u8, index: i32, failure: &'static str, wide: bool) -> String {
    let mut array = fixture(failure);
    let source = BorrowedItem { value: 99 };
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        let result = if wide {
            operate(&mut array, &source, operation, index as usize)
        } else {
            operate(&mut array, &source, operation, index)
        };
        result.map(|value| value as *const BorrowedItem)
    }));
    let returned = returned_address(&array, outcome, operation, index);
    let events = BORROWED_PROBE.with(|probe| probe.borrow().events.clone());
    let mut row = format!("{name} {events} {} {}", array.size(), usize::from(returned));
    for value in array.iter() {
        write!(row, " {}", value.value).unwrap();
    }
    write!(row, " source={}", source.value).unwrap();
    row
}
fn returned_address(
    array: &DynamicArray<BorrowedItem>,
    outcome: std::thread::Result<Result<*const BorrowedItem, ArrayError>>,
    operation: u8,
    index: i32,
) -> bool {
    if let Ok(Ok(pointer)) = outcome {
        let offset = if operation == 0 {
            2
        } else {
            index as usize + usize::from(operation == 3)
        };
        return pointer == &array[offset];
    }
    event("E".into());
    false
}
macro_rules! native_case {
    ($name:ident, $operation:expr, $index:expr, $failure:expr, $wide:expr) => {
        #[test]
        fn $name() {
            let expected =
                include_str!("../../../../tools/oracle/tkernel_dynamic_borrowed_methods.expected")
                    .lines()
                    .find(|row| row.starts_with(concat!(stringify!($name), " ")))
                    .unwrap();
            assert_eq!(
                actual_row(stringify!($name), $operation, $index, $failure, $wide),
                expected
            );
        }
    };
}
native_case!(append, 0, 0, "", false);
native_case!(append_copy_throw, 0, 0, "construct", true);
native_case!(set_existing, 1, 0, "", false);
native_case!(set_existing_wide, 1, 1, "", true);
native_case!(set_copy_throw, 1, 0, "construct", false);
native_case!(set_move_throw, 1, 0, "move", true);
native_case!(set_gap, 1, 4, "", false);
native_case!(set_gap_wide, 1, 4, "", true);
native_case!(set_gap_copy_throw, 1, 4, "construct", false);
native_case!(set_gap_default_throw, 1, 4, "default", true);
native_case!(set_negative, 1, -1, "", false);
native_case!(before, 2, 0, "", false);
native_case!(before_wide, 2, 1, "", true);
native_case!(after, 3, 0, "", false);
native_case!(after_wide, 3, 1, "", true);
native_case!(before_copy_throw, 2, 0, "assign", false);
native_case!(after_move_throw, 3, 0, "move", true);
native_case!(before_default_throw, 2, 0, "default", false);
native_case!(before_negative, 2, -1, "", false);
native_case!(after_end, 3, 2, "", true);
