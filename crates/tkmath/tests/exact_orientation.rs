// SPDX-License-Identifier: GPL-2.0-only
//! Independent rational evidence for the extension predicate (issue #10).

use rustycad_tkmath::predicates::{NonFiniteCoordinate, OrientationSign, orient2d};

fn expected_sign(number: i32) -> OrientationSign {
    match number {
        -1 => OrientationSign::Negative,
        0 => OrientationSign::Zero,
        1 => OrientationSign::Positive,
        _ => panic!("sign {number}; expected -1, 0, or 1"),
    }
}

fn parse_fixture(line: &str) -> ([[f64; 2]; 3], i32) {
    let fields: Vec<&str> = line.split(',').collect();
    assert_eq!(fields.len(), 7, "fixture {line}; expected seven columns");
    let coordinates: Vec<f64> = fields[..6]
        .iter()
        .map(|word| f64::from_bits(u64::from_str_radix(word, 16).unwrap()))
        .collect();
    (
        [
            [coordinates[0], coordinates[1]],
            [coordinates[2], coordinates[3]],
            [coordinates[4], coordinates[5]],
        ],
        fields[6].parse().unwrap(),
    )
}

#[test]
fn exact_orientation_matches_independent_rational_corpus_and_permutations() {
    let corpus = include_str!("exact_orientation/oracle.csv");
    assert_eq!(corpus.lines().count(), 392);
    for line in corpus.lines().skip(1) {
        assert_permutations(line);
    }
}

fn assert_permutations(line: &str) {
    let ([a, b, c], sign) = parse_fixture(line);
    for (points, parity) in [
        ([a, b, c], 1),
        ([b, c, a], 1),
        ([c, a, b], 1),
        ([a, c, b], -1),
        ([b, a, c], -1),
        ([c, b, a], -1),
    ] {
        assert_eq!(
            orient2d(points[0], points[1], points[2]),
            Ok(expected_sign(sign * parity)),
            "fixture: {line}"
        );
    }
}

#[test]
fn exact_orientation_preserves_sign_across_all_binary_exponents() {
    for exponent in 0_u64..2047 {
        let scale = f64::from_bits((exponent << 52) | 1);
        assert_eq!(
            orient2d([0., 0.], [scale, 0.], [0., scale]),
            Ok(OrientationSign::Positive),
            "exponent {exponent}"
        );
        assert_eq!(
            orient2d([-scale, -scale], [0., 0.], [scale, scale]),
            Ok(OrientationSign::Zero),
            "exponent {exponent}"
        );
    }
}

#[test]
fn exact_orientation_rejects_nonfinite_inputs_with_coordinate_identity() {
    for point in 0..3 {
        for axis in 0..2 {
            assert_invalid_coordinate(point, axis);
        }
    }
}

fn assert_invalid_coordinate(point: usize, axis: usize) {
    for value in [f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
        let mut triangle = [[0., 0.]; 3];
        triangle[point][axis] = value;
        let error = orient2d(triangle[0], triangle[1], triangle[2]).unwrap_err();
        assert_eq!((error.point, error.axis), (point, axis));
        assert_eq!(error.value.to_bits(), value.to_bits());
        assert_eq!(
            error.to_string(),
            format!("point {point} axis {axis} is {value}; expected a finite binary64 coordinate")
        );
        assert!(std::error::Error::source(&error).is_none());
    }
}

#[test]
fn exact_orientation_reports_the_first_invalid_coordinate() {
    assert_eq!(
        orient2d([1., f64::INFINITY], [f64::NEG_INFINITY, 0.], [0., 0.]),
        Err(NonFiniteCoordinate {
            point: 0,
            axis: 1,
            value: f64::INFINITY
        })
    );
}

#[test]
fn exact_corpus_exercises_failures_of_rounded_determinants() {
    let disagreements = include_str!("exact_orientation/oracle.csv")
        .lines()
        .skip(1)
        .filter(|line| {
            let ([a, b, c], sign) = parse_fixture(line);
            let determinant = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
            let rounded_sign = i32::from(determinant > 0.) - i32::from(determinant < 0.);
            !determinant.is_finite() || rounded_sign != sign
        })
        .count();
    assert!(
        disagreements > 50,
        "only {disagreements} difficult cases; expected >50"
    );
}
