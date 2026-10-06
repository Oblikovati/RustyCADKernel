use rustycad_tkernel::ncollection::{Array1, Array2};
use std::mem::MaybeUninit;

// No Default: the native aggregate has no element-construction operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UninitCell(pub i32);

pub fn write_every_slot(slots: &mut [MaybeUninit<UninitCell>]) {
    for (index, slot) in slots.iter_mut().enumerate() {
        slot.write(UninitCell(index as i32 * 7 + 11));
    }
}
fn values_row(values: &[UninitCell]) -> String {
    values
        .iter()
        .map(|cell| cell.0.to_string())
        .collect::<Vec<_>>()
        .join(",")
}
pub fn linear_row(values: &Array1<'_, UninitCell>, stable: bool) -> String {
    format!(
        "{}:{};{};{};{}",
        values.lower(),
        values.upper(),
        u8::from(values.is_deletable()),
        u8::from(stable),
        values_row(values.as_slice())
    )
}
pub fn matrix_row(values: &Array2<'_, UninitCell>, stable: bool) -> String {
    let bounds = format!(
        "{}:{}:{}:{}:{}:{}",
        values.lower_row(),
        values.upper_row(),
        values.lower_col(),
        values.upper_col(),
        values.array1().lower(),
        values.array1().upper()
    );
    format!(
        "{bounds};{};{};{}",
        u8::from(values.is_deletable()),
        u8::from(stable),
        values_row(values.as_slice())
    )
}
pub fn initialized_linear(mut slots: Array1<'_, MaybeUninit<UninitCell>>) -> String {
    write_every_slot(slots.as_mut_slice());
    let pointer = slots.data_ptr().cast::<UninitCell>();
    // SAFETY: the complete actual flat slice was initialized immediately above.
    let values = unsafe { slots.assume_init() };
    linear_row(&values, pointer == values.data_ptr())
}
pub fn initialized_matrix(mut slots: Array2<'_, MaybeUninit<UninitCell>>) -> String {
    write_every_slot(slots.as_mut_slice());
    let pointer = slots.data_ptr().cast::<UninitCell>();
    // SAFETY: every actual backing slot is initialized, regardless of rectangle size.
    let values = unsafe { slots.assume_init() };
    matrix_row(&values, pointer == values.data_ptr())
}
pub fn assert_native_row(label: &str, actual: &str) {
    let prefix = format!("{label}=");
    let expected = include_str!("../../../../tools/oracle/tkernel_array_uninitialized.expected")
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap();
    assert_eq!(actual, expected, "native row {label}");
}
