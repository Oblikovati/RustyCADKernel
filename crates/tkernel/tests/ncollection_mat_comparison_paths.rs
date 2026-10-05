//! Mat3/Mat4 comparison dispatch matches native element `!=` calls and short-circuit order.
use rustycad_tkernel::ncollection::{Mat3, Mat4};
use std::cell::Cell;

#[derive(Clone, Copy, Debug)]
struct ComparisonProbe<'a> {
    value: i32,
    eq_calls: &'a Cell<usize>,
    ne_calls: &'a Cell<usize>,
}

struct ComparisonCounts {
    eq: Cell<usize>,
    ne: Cell<usize>,
}

impl PartialEq for ComparisonProbe<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.eq_calls.set(self.eq_calls.get() + 1);
        self.value == other.value
    }
    #[allow(clippy::partialeq_ne_impl)] // The probe must distinguish native `!=` dispatch.
    fn ne(&self, other: &Self) -> bool {
        self.ne_calls.set(self.ne_calls.get() + 1);
        self.value != other.value
    }
}

fn probe<'a>(
    value: i32,
    eq_calls: &'a Cell<usize>,
    ne_calls: &'a Cell<usize>,
) -> ComparisonProbe<'a> {
    ComparisonProbe {
        value,
        eq_calls,
        ne_calls,
    }
}

fn record_comparison(
    label: &str,
    expected: bool,
    calls: usize,
    counts: &ComparisonCounts,
    compare: impl FnOnce() -> bool,
) -> String {
    counts.eq.set(0);
    counts.ne.set(0);
    let result = compare();
    assert_eq!(result, expected, "{label} result");
    assert_eq!(counts.eq.get(), 0, "{label} eq calls");
    assert_eq!(counts.ne.get(), calls, "{label} ne calls");
    format!(
        "{label} {} eq=0 ne={calls} expected-ne={calls}",
        u8::from(result)
    )
}

fn mat3_values<'a>(
    mismatch: Option<usize>,
    eq: &'a Cell<usize>,
    ne: &'a Cell<usize>,
) -> Mat3<ComparisonProbe<'a>> {
    Mat3::from_data(std::array::from_fn(|index| {
        probe(if Some(index) == mismatch { 2 } else { 1 }, eq, ne)
    }))
}

fn mat4_values<'a>(
    mismatch: Option<usize>,
    eq: &'a Cell<usize>,
    ne: &'a Cell<usize>,
) -> Mat4<ComparisonProbe<'a>> {
    Mat4::from_data(std::array::from_fn(|index| {
        probe(if Some(index) == mismatch { 2 } else { 1 }, eq, ne)
    }))
}

fn comparison_rows(
    case: &str,
    expected: bool,
    calls: usize,
    counts: &ComparisonCounts,
    is_equal: impl FnOnce() -> bool,
    equal: impl FnOnce() -> bool,
    not_equal: impl FnOnce() -> bool,
) -> Vec<String> {
    let native_label = format!("{case} IsEqual");
    let equal_label = format!("{case} operator==");
    let unequal_label = format!("{case} operator!=");
    let native = record_comparison(&native_label, expected, calls, counts, is_equal);
    let equal = record_comparison(&equal_label, expected, calls, counts, equal);
    let unequal = record_comparison(&unequal_label, !expected, calls, counts, not_equal);
    vec![native, equal, unequal]
}

fn mat3_case(case: &str, mismatch: Option<usize>) -> Vec<String> {
    let counts = ComparisonCounts {
        eq: Cell::new(0),
        ne: Cell::new(0),
    };
    let first = mat3_values(None, &counts.eq, &counts.ne);
    let second = mat3_values(mismatch, &counts.eq, &counts.ne);
    comparison_rows(
        case,
        mismatch.is_none(),
        mismatch.map_or(9, |index| index + 1),
        &counts,
        || first.is_equal(&second),
        || first == second,
        || first != second,
    )
}

fn mat4_case(case: &str, mismatch: Option<usize>) -> Vec<String> {
    let counts = ComparisonCounts {
        eq: Cell::new(0),
        ne: Cell::new(0),
    };
    let first = mat4_values(None, &counts.eq, &counts.ne);
    let second = mat4_values(mismatch, &counts.eq, &counts.ne);
    comparison_rows(
        case,
        mismatch.is_none(),
        mismatch.map_or(16, |index| index + 1),
        &counts,
        || first.is_equal(&second),
        || first == second,
        || first != second,
    )
}

fn parity_rust_item(upstream_id: &str) -> &'static str {
    let mappings = include_str!("../../../porting/parity.json");
    let marker = format!("\"upstream_id\": \"{upstream_id}\"");
    let row = mappings
        .split(&marker)
        .nth(1)
        .expect("mapping selector exists");
    let row_end = row.find("\n    },").expect("mapping row ends");
    row[..row_end]
        .lines()
        .find_map(|line| line.trim().strip_prefix("\"rust_item\": \""))
        .and_then(|item| item.strip_suffix("\","))
        .expect("mapping row has one Rust target")
}

#[test]
fn mat3_equality_dispatch_matches_native() {
    let mut actual = mat3_case("Mat3-equal", None);
    actual.extend(mat3_case("Mat3-mismatch", Some(4)));
    let expected = include_str!("fixtures/ncollection_mat_comparison.expected")
        .lines()
        .take(6)
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(actual.join("\n"), expected);
}

#[test]
fn mat4_equality_dispatch_matches_native() {
    let mut actual = mat4_case("Mat4-equal", None);
    actual.extend(mat4_case("Mat4-mismatch", Some(4)));
    let expected = include_str!("fixtures/ncollection_mat_comparison.expected")
        .lines()
        .skip(6)
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(actual.join("\n"), expected);
}

const MATRIX_OPERATOR_BINDINGS: [(&str, &str); 4] = [
    (
        "NCollection_Mat3::operator==(const NCollection_Mat3<Element_t> &) const noexcept",
        "rustycad_tkernel::ncollection::Mat3::<core::cmp::PartialEq>::eq",
    ),
    (
        "NCollection_Mat3::operator!=(const NCollection_Mat3<Element_t> &) const noexcept",
        "rustycad_tkernel::ncollection::Mat3::<core::cmp::PartialEq>::ne",
    ),
    (
        "NCollection_Mat4::operator==(const NCollection_Mat4<Element_t> &) const noexcept",
        "rustycad_tkernel::ncollection::Mat4::<core::cmp::PartialEq>::eq",
    ),
    (
        "NCollection_Mat4::operator!=(const NCollection_Mat4<Element_t> &) const noexcept",
        "rustycad_tkernel::ncollection::Mat4::<core::cmp::PartialEq>::ne",
    ),
];

#[test]
fn parity_selects_the_matching_matrix_trait_methods() {
    for (selector, expected) in MATRIX_OPERATOR_BINDINGS {
        assert_eq!(parity_rust_item(selector), expected, "{selector}");
    }
}
