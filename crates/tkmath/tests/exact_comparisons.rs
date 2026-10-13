// SPDX-License-Identifier: GPL-2.0-only
//! Supplied-coordinate ordering, checked against independent exact rational arithmetic.
use rustycad_tkmath::predicates::{compare_line_projections_2d, compare_squared_distances_2d};
use std::cmp::Ordering;
fn fixture(line: &str) -> ([[f64; 2]; 4], [Ordering; 2]) {
    let fields: Vec<&str> = line.split(',').collect();
    assert_eq!(fields.len(), 10);
    let points = std::array::from_fn(|point| {
        std::array::from_fn(|axis| {
            f64::from_bits(u64::from_str_radix(fields[2 * point + axis], 16).unwrap())
        })
    });
    let expected = [ordering(fields[8]), ordering(fields[9])];
    (points, expected)
}
fn ordering(value: &str) -> Ordering {
    match value {
        "-1" => Ordering::Less,
        "0" => Ordering::Equal,
        "1" => Ordering::Greater,
        _ => panic!("ordering {value:?}: expected -1, 0, or 1"),
    }
}
#[test]
fn comparisons_match_296_independent_rational_cases_and_operand_reversals() {
    let rows: Vec<&str> = include_str!("exact_comparisons/oracle.csv")
        .lines()
        .skip(1)
        .collect();
    assert_eq!(rows.len(), 296);
    for row in rows {
        assert_case(row);
    }
}
fn assert_case(row: &str) {
    let ([a, b, c, d], [distance, projection]) = fixture(row);
    assert_eq!(compare_squared_distances_2d(a, b, c), Ok(distance), "{row}");
    assert_eq!(
        compare_squared_distances_2d(a, c, b),
        Ok(distance.reverse()),
        "{row}"
    );
    assert_projection_reversals([a, b, c, d], projection, row);
}
fn assert_projection_reversals(points: [[f64; 2]; 4], expected: Ordering, row: &str) {
    let [a, b, c, d] = points;
    for (arguments, wanted) in [
        ([a, b, c, d], expected),
        ([b, a, c, d], expected.reverse()),
        ([a, b, d, c], expected.reverse()),
        ([b, a, d, c], expected),
    ] {
        let [start, end, first, second] = arguments;
        assert_eq!(
            compare_line_projections_2d(start, end, first, second),
            Ok(wanted),
            "{row}"
        );
    }
}
#[test]
fn signs_survive_every_binary64_exponent_without_overflow_or_underflow() {
    for exponent in 0_u64..2047 {
        let scale = f64::from_bits((exponent << 52) | 1);
        assert_eq!(
            compare_squared_distances_2d([0., 0.], [scale, 0.], [scale, scale]),
            Ok(Ordering::Less)
        );
        assert_eq!(
            compare_line_projections_2d([-scale, 0.], [scale, 0.], [scale, scale], [0., 0.]),
            Ok(Ordering::Greater)
        );
        assert_eq!(
            compare_line_projections_2d([-scale, 0.], [scale, 0.], [scale, scale], [scale, -scale]),
            Ok(Ordering::Equal)
        );
    }
}
#[test]
fn exact_ties_signed_zeros_and_zero_directions_remain_explicit_equalities() {
    assert_eq!(
        compare_squared_distances_2d([0., 0.], [3., 4.], [-3., -4.]),
        Ok(Ordering::Equal)
    );
    assert_eq!(
        compare_squared_distances_2d([-0., 0.], [0., -0.], [-0., -0.]),
        Ok(Ordering::Equal)
    );
    assert_equal_projections();
}
fn assert_equal_projections() {
    assert_eq!(
        compare_line_projections_2d([1., 2.], [1., 2.], [f64::MAX, 0.], [-f64::MAX, 0.]),
        Ok(Ordering::Equal)
    );
    assert_eq!(
        compare_line_projections_2d([0., 0.], [1., 1.], [1., 0.], [0., 1.]),
        Ok(Ordering::Equal)
    );
}
#[test]
fn nonfinite_rejection_names_every_argument_and_coordinate_before_arithmetic() {
    for point in 0..4 {
        for axis in 0..2 {
            assert_nonfinite(point, axis);
        }
    }
}
fn assert_nonfinite(point: usize, axis: usize) {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut points = [[0., 0.]; 4];
        points[point][axis] = value;
        let projection =
            compare_line_projections_2d(points[0], points[1], points[2], points[3]).unwrap_err();
        assert_eq!((projection.point, projection.axis), (point, axis));
        assert_eq!(projection.value.to_bits(), value.to_bits());
        if point < 3 {
            let distance =
                compare_squared_distances_2d(points[0], points[1], points[2]).unwrap_err();
            assert_eq!((distance.point, distance.axis), (point, axis));
            assert_eq!(distance.value.to_bits(), value.to_bits());
        }
    }
}
#[test]
fn first_nonfinite_coordinate_wins_even_for_an_algebraically_zero_direction() {
    let a = [1., f64::INFINITY];
    let b = [f64::NAN, 0.];
    let distance = compare_squared_distances_2d(a, b, b).unwrap_err();
    let projection = compare_line_projections_2d(a, a, b, b).unwrap_err();
    assert_eq!((distance.point, distance.axis), (0, 1));
    assert_eq!((projection.point, projection.axis), (0, 1));
}
#[test]
fn corpus_exposes_false_ties_and_nonfinite_rounded_metric_results() {
    let failures = include_str!("exact_comparisons/oracle.csv")
        .lines()
        .skip(1)
        .filter(|row| rounded_disagrees(row))
        .count();
    assert!(
        failures > 30,
        "only {failures} difficult cases: expected more than 30"
    );
}
fn rounded_disagrees(row: &str) -> bool {
    let ([a, b, c, d], expected) = fixture(row);
    let distance = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2))
        - ((c[0] - a[0]).powi(2) + (c[1] - a[1]).powi(2));
    let projection = (b[0] - a[0]) * (c[0] - d[0]) + (b[1] - a[1]) * (c[1] - d[1]);
    [distance, projection]
        .into_iter()
        .zip(expected)
        .any(|(actual, wanted)| !actual.is_finite() || actual.partial_cmp(&0.) != Some(wanted))
}
#[test]
fn ordinary_cases_also_match_independent_occt_metric_comparisons() {
    let corpus: Vec<&str> = include_str!("exact_comparisons/oracle.csv")
        .lines()
        .skip(1)
        .collect();
    let rows: Vec<&str> = include_str!("exact_comparisons/occt-8.0.0.csv")
        .lines()
        .collect();
    assert_eq!(rows.len(), 8);
    for row in rows {
        assert_occt_case(row, &corpus);
    }
}
fn assert_occt_case(row: &str, corpus: &[&str]) {
    let fields: Vec<&str> = row.split(',').collect();
    assert_eq!(fields.len(), 3);
    let index: usize = fields[0].parse().unwrap();
    let ([a, b, c, d], _) = fixture(corpus[index]);
    assert_eq!(
        compare_squared_distances_2d(a, b, c),
        Ok(ordering(fields[1]))
    );
    assert_eq!(
        compare_line_projections_2d(a, b, c, d),
        Ok(ordering(fields[2]))
    );
}
#[test]
fn finite_rounded_ties_do_not_replace_true_orderings() {
    let small = 2.0_f64.powi(-27);
    assert_eq!(1.0 + small * small, 1.0);
    assert_eq!(
        compare_squared_distances_2d([0., 0.], [1., 0.], [1., small]),
        Ok(Ordering::Less)
    );
    let displacement = 2.0_f64.powi(-54);
    assert_eq!((1.0 - displacement) - 1.0, 0.0);
    assert_eq!(
        compare_line_projections_2d([0., 0.], [1., 1.], [1., -1.], [displacement, 0.]),
        Ok(Ordering::Less)
    );
}
