//! Defined Jacobi D0123 behavior when native output arrays share storage.
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::plib::{BasisRows, JacobiPolynomial};
use std::fmt::Write;

const INITIAL: f64 = 777.0;
const COUNT: usize = 5;

fn jacobi() -> JacobiPolynomial {
    JacobiPolynomial::new(8, Shape::C1).unwrap()
}

fn emit(label: &str, values: &[f64]) -> String {
    let mut line: String = format!("{label} ");
    for value in values {
        write!(line, "{} ", value.to_bits()).unwrap();
    }
    line
}

#[test]
fn overlapping_output_spans_match_native_write_order() {
    let expected: Vec<String> =
        include_str!("../../../tools/oracle/tkmath_plib_jacobi_overlap.expected")
            .lines()
            .map(str::to_owned)
            .collect();
    assert_eq!(
        [
            all_alias(),
            vec![selected_alias(), offset_alias(), disjoint()],
        ]
        .concat(),
        expected
    );
}

fn all_alias() -> Vec<String> {
    [-1, 0, 1, 2, 3, 4]
        .into_iter()
        .map(all_alias_order)
        .collect()
}

fn all_alias_order(order: i32) -> String {
    let mut values: [f64; COUNT] = [INITIAL; COUNT];
    jacobi()
        .d0123_overlapping(order, 0.37, &mut values, [0; 4])
        .unwrap();
    emit(&format!("all {order}"), &values)
}

fn selected_alias() -> String {
    let mut values: [f64; COUNT * 3] = [INITIAL; COUNT * 3];
    jacobi()
        .d0123_overlapping(2, 0.37, &mut values, [0, COUNT, 0, 2 * COUNT])
        .unwrap();
    emit("selected", &values)
}

fn offset_alias() -> String {
    let mut values: [f64; COUNT + 3] = [INITIAL; COUNT + 3];
    jacobi()
        .d0123_overlapping(3, 0.37, &mut values, [0, 2, 1, 3])
        .unwrap();
    emit("offset", &values)
}

fn disjoint() -> String {
    let mut values: [f64; COUNT * 4] = [INITIAL; COUNT * 4];
    jacobi()
        .d0123_overlapping(3, 0.37, &mut values, [0, COUNT, 2 * COUNT, 3 * COUNT])
        .unwrap();
    emit("disjoint", &values)
}

#[test]
fn disjoint_overlapping_api_matches_separate_slice_api() {
    let mut separate: [Vec<f64>; 4] = std::array::from_fn(|_| vec![INITIAL; COUNT]);
    let [value, d1, d2, d3] = &mut separate;
    jacobi()
        .d0123(
            3,
            0.37,
            BasisRows {
                rows: [value, d1, d2, d3],
            },
        )
        .unwrap();
    let mut indexed: [f64; COUNT * 4] = [INITIAL; COUNT * 4];
    jacobi()
        .d0123_overlapping(3, 0.37, &mut indexed, [0, COUNT, 2 * COUNT, 3 * COUNT])
        .unwrap();
    assert_eq!(indexed.as_slice(), separate.concat());
}

#[test]
fn short_overlapping_buffer_fails_without_mutation() {
    let mut values: [f64; COUNT + 1] = [INITIAL; COUNT + 1];
    let before: [u64; COUNT + 1] = values.map(f64::to_bits);
    let error = jacobi()
        .d0123_overlapping(3, 0.37, &mut values, [0, 0, 1, 2])
        .unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert!(error.message().contains("row 3 starts 2, needs 5 values"));
    assert!(error.message().contains("buffer length 6"));
    assert_eq!(values.map(f64::to_bits), before);
}
