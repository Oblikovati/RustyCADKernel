//! Mat3/Mat4 comparison dispatch matches native element `!=` calls and short-circuit order.
use rustycad_tkernel::ncollection::{Mat3, Mat4};
use std::cell::Cell;

thread_local! {
    static EQ_CALLS: Cell<usize> = const { Cell::new(0) };
    static NE_CALLS: Cell<usize> = const { Cell::new(0) };
}

#[derive(Clone, Copy, Debug)]
struct ComparisonProbe(i32);

impl PartialEq for ComparisonProbe {
    fn eq(&self, other: &Self) -> bool {
        EQ_CALLS.with(|calls| calls.set(calls.get() + 1));
        self.0 == other.0
    }
    #[allow(clippy::partialeq_ne_impl)] // Distinct counters verify native dispatch calls `!=`.
    fn ne(&self, other: &Self) -> bool {
        NE_CALLS.with(|calls| calls.set(calls.get() + 1));
        self.0 != other.0
    }
}

impl From<u8> for ComparisonProbe {
    fn from(value: u8) -> Self {
        Self(i32::from(value))
    }
}

fn record_comparison(
    label: &str,
    expected: bool,
    calls: usize,
    compare: impl FnOnce() -> bool,
) -> String {
    EQ_CALLS.with(|calls| calls.set(0));
    NE_CALLS.with(|calls| calls.set(0));
    let result = compare();
    assert_eq!(result, expected, "{label} result");
    EQ_CALLS.with(|count| assert_eq!(count.get(), 0, "{label} eq calls"));
    NE_CALLS.with(|count| assert_eq!(count.get(), calls, "{label} ne calls"));
    format!(
        "{label} {} eq=0 ne={calls} expected-ne={calls}",
        u8::from(result)
    )
}

fn mat3_values(mismatch: Option<usize>) -> Mat3<ComparisonProbe> {
    Mat3::from_data(std::array::from_fn(|index| {
        ComparisonProbe(if Some(index) == mismatch { 2 } else { 1 })
    }))
}

fn mat4_values(mismatch: Option<usize>) -> Mat4<ComparisonProbe> {
    Mat4::from_data(std::array::from_fn(|index| {
        ComparisonProbe(if Some(index) == mismatch { 2 } else { 1 })
    }))
}

fn comparison_rows(
    case: &str,
    expected: bool,
    calls: usize,
    is_equal: impl FnOnce() -> bool,
    equal: impl FnOnce() -> bool,
    not_equal: impl FnOnce() -> bool,
) -> Vec<String> {
    let equal_call = record_comparison(&format!("{case} IsEqual"), expected, calls, is_equal);
    let operator_equal = record_comparison(&format!("{case} operator=="), expected, calls, equal);
    let operator_not_equal =
        record_comparison(&format!("{case} operator!="), !expected, calls, not_equal);
    vec![equal_call, operator_equal, operator_not_equal]
}

fn mat3_case(case: &str, mismatch: Option<usize>) -> Vec<String> {
    let first = mat3_values(None);
    let second = mat3_values(mismatch);
    let expected = mismatch.is_none();
    let calls = mismatch.map_or(9, |index| index + 1);
    comparison_rows(
        case,
        expected,
        calls,
        || first.is_equal(&second),
        || first == second,
        || first != second,
    )
}

fn mat4_case(case: &str, mismatch: Option<usize>) -> Vec<String> {
    let first = mat4_values(None);
    let second = mat4_values(mismatch);
    let expected = mismatch.is_none();
    let calls = mismatch.map_or(16, |index| index + 1);
    comparison_rows(
        case,
        expected,
        calls,
        || first.is_equal(&second),
        || first == second,
        || first != second,
    )
}

#[test]
fn mat3_equality_dispatch_matches_native() {
    let mut actual = mat3_case("Mat3-equal", None);
    actual.extend(mat3_case("Mat3-mismatch", Some(4)));
    assert_eq!(
        actual.join("\n"),
        include_str!("fixtures/ncollection_mat_comparison.expected")
            .lines()
            .take(6)
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn mat4_equality_dispatch_matches_native() {
    let mut actual = mat4_case("Mat4-equal", None);
    actual.extend(mat4_case("Mat4-mismatch", Some(4)));
    assert_eq!(
        actual.join("\n"),
        include_str!("fixtures/ncollection_mat_comparison.expected")
            .lines()
            .skip(6)
            .collect::<Vec<_>>()
            .join("\n")
    );
}
