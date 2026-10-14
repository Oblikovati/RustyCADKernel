// SPDX-License-Identifier: GPL-2.0-only
use super::super::*;

fn world_identity() -> MetricEmbedding<3> {
    MetricEmbedding {
        linear: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        origin: [0.; 3],
        scale: 1.,
    }
}
fn closed_cubic_side() -> CertifiedPolynomialSideIntegral {
    let poles = [[0., 0., 0.], [4., 0., 0.], [0., 4., 0.], [0., 0., 0.]];
    let curve = MetricCurve::Bezier(MetricBezier {
        poles: &poles,
        weights: None,
    });
    certify_polynomial_side_integral(
        curve,
        world_identity(),
        world_identity(),
        [0., 0., 2.],
        Default::default(),
        || false,
    )
    .unwrap()
}
fn independent_interval(quantity: &str) -> [f64; 2] {
    let fixture = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/prism_complete_measures/closed_cubic.tsv"
    ));
    let row = fixture
        .lines()
        .find(|line| line.starts_with(quantity))
        .unwrap();
    let bits: Vec<u64> = row
        .split('\t')
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect();
    [f64::from_bits(bits[0]), f64::from_bits(bits[1])]
}
#[test]
fn closed_cubic_side_contains_independent_midpoint_error_interval() {
    let report = closed_cubic_side();
    let oracle = independent_interval("side");
    assert!(report.enclosure()[0] <= oracle[0] && oracle[1] <= report.enclosure()[1]);
    assert!(oracle[1] - oracle[0] < 8e-8);
}
#[test]
fn total_sum_contains_independent_side_plus_exact_two_cap_interval() {
    let side = closed_cubic_side();
    let cap = [
        f64::from_bits(0x4003333333333333),
        f64::from_bits(0x4003333333333334),
    ];
    let report =
        sum_nonnegative_enclosures(&[cap, cap, side.enclosure()], Default::default(), || false)
            .unwrap();
    let oracle = independent_interval("total");
    assert!(report.enclosure()[0] <= oracle[0] && oracle[1] <= report.enclosure()[1]);
    assert_eq!(report.components(), &[cap, cap, side.enclosure()]);
}
