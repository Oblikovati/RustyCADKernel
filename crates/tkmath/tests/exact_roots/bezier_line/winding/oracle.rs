// SPDX-License-Identifier: GPL-2.0-only
use super::*;
#[test]
fn winding_matches_independent_global_root_order_and_multiplicity() {
    let expected = include_str!("oracle.tsv");
    assert_eq!(expected.lines().count(), 16);
    for row in expected.lines() {
        check_expected(row);
    }
}
fn check_expected(row: &str) {
    let (name, expected) = row.split_once('\t').unwrap();
    let geometry = include_str!("../rays/oracle.tsv")
        .lines()
        .find(|line| line.split('\t').next() == Some(name))
        .unwrap();
    let expected = if expected == "boundary" {
        BezierWinding::Boundary
    } else {
        BezierWinding::Clear(expected.parse().unwrap())
    };
    let actual = super::super::fixture::with_ray_fixture(geometry, |request| {
        check_reversed(request, expected);
        winding(request)
    });
    assert_eq!(actual, expected, "{name}");
}
#[test]
fn winding_matches_native_ordinary_curve_contributions() {
    let expected = include_str!("occt-8.0.0.tsv");
    assert_eq!(expected.lines().count(), 6);
    for row in expected.lines() {
        check_expected(row);
    }
}

fn check_reversed(request: BezierLineQuery<'_>, expected: BezierWinding) {
    let controls: Vec<[f64; 2]> = request.controls.iter().rev().copied().collect();
    let weights: Option<Vec<f64>> = request
        .weights
        .map(|weights| weights.iter().rev().copied().collect());
    let reversed = BezierLineQuery {
        controls: &controls,
        weights: weights.as_deref(),
        interval: [1. - request.interval[1], 1. - request.interval[0]],
        ..request
    };
    let expected = match expected {
        BezierWinding::Boundary => BezierWinding::Boundary,
        BezierWinding::Clear(count) => BezierWinding::Clear(-count),
    };
    assert_eq!(winding(reversed), expected);
}
