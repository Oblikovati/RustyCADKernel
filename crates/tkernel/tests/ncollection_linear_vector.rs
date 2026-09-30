//! Legacy LinearVector declarations; each retains its original input sequence.
#![allow(non_snake_case, clippy::approx_constant)]
use rustycad_tkernel::ncollection::{Array1, LinearVector};
use rustycad_tkernel::tcollection::AsciiString;

fn integers(values: &[i32]) -> LinearVector<i32> {
    let mut vector = LinearVector::default();
    for &value in values {
        vector.append(value);
    }
    vector
}
fn strings(values: &[&str]) -> LinearVector<String> {
    let mut vector = LinearVector::default();
    for value in values {
        vector.append((*value).to_owned());
    }
    vector
}
fn same_values<T: std::fmt::Debug + PartialEq>(vector: &LinearVector<T>, expected: &[T]) {
    assert_eq!(vector.len(), expected.len());
    assert_eq!(vector.as_slice(), expected);
}
struct MoveOnly {
    value: i32,
}
struct MultiArg {
    integer: i32,
    real: f64,
}
const _: fn(&mut LinearVector<i32>) -> Array1<'_, i32> = LinearVector::to_array1;
const _: fn(&LinearVector<i32>) -> &[i32] = LinearVector::as_slice;
include!("ncollection_linear_vector/basic.rs");
include!("ncollection_linear_vector/mutation.rs");
include!("ncollection_linear_vector/storage.rs");
