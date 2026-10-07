use rustycad_tkernel::ncollection::{Array2, ArrayError};
use std::panic::{AssertUnwindSafe, catch_unwind};

fn mutate_value(value: &mut i32) -> i32 {
    *value += 100;
    *value
}
fn assignment_operation(
    array: &mut Array2<'_, i32>,
    row: i32,
    col: i32,
    op: i32,
) -> Result<i32, ArrayError> {
    if op == 4 {
        array.set_value_ref(row, col, &77)?;
        return Ok(77);
    }
    if op == 5 {
        let mut donor = 78;
        array.set_value_with_assignment(row, col, &mut donor, |t, s| *t = *s)?;
        return Ok(donor);
    }
    array
        .emplace_value_with_assignment(row, col, || 79, |t, s| *t = *s)
        .copied()
}
fn operation(array: &mut Array2<'_, i32>, index: (i32, i32), op: i32) -> Result<i32, ArrayError> {
    match op {
        0 => array.value(index.0, index.1).copied(),
        1 => array.change_value(index.0, index.1).map(mutate_value),
        // The native operator() pair; Rust has no Index impl (plan D3), so the checked
        // accessors carry them and report the same Standard_OutOfRange as an error.
        2 => array.value(index.0, index.1).copied(),
        3 => array.change_value(index.0, index.1).map(mutate_value),
        _ => assignment_operation(array, index.0, index.1, op),
    }
}
fn defined_values(values: &[i32]) -> String {
    values
        .iter()
        .map(i32::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
fn exercise(array: &mut Array2<'_, i32>, index: (i32, i32), op: i32) -> String {
    let pointer = array.data_ptr();
    let outcome = catch_unwind(AssertUnwindSafe(|| operation(array, index, op)));
    let (value, error) = match outcome {
        Ok(Ok(value)) => (value, 0),
        _ => (0, 1),
    };
    format!(
        "{value};{error};{};{};{}",
        u8::from(pointer == array.data_ptr()),
        u8::from(array.is_deletable()),
        defined_values(array.as_slice())
    )
}
pub fn native_row(label: &str, actual: &str) {
    let prefix = format!("{label}=");
    let expected = include_str!("../../../../tools/oracle/tkernel_array2_index_limits.expected")
        .lines()
        .find_map(|row| row.strip_prefix(&prefix))
        .unwrap();
    assert_eq!(actual, expected, "native row {label}");
}
fn check_operation(label: &str, lower: (i32, i32), index: (i32, i32), op: i32, owned: bool) {
    let mut buffer = [1, 2, 3, 4, 5, 6, 7, 8, 9];
    let mut array = if owned {
        Array2::with_size(3, 3)
    } else {
        Array2::from_buffer(&mut buffer, 3, 3).unwrap()
    };
    array
        .as_mut_slice()
        .copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    array.update_lower_row(lower.0);
    array.update_lower_col(lower.1);
    let label = format!("{label}_{op}_{}", u8::from(owned));
    native_row(&label, &exercise(&mut array, index, op));
}
pub fn check_case(label: &str, lower: (i32, i32), index: (i32, i32)) {
    for op in 0..7 {
        check_operation(label, lower, index, op, false);
        check_operation(label, lower, index, op, true);
    }
}
