// SPDX-License-Identifier: GPL-2.0-only
//! Owned 8d4009c02 behavior, pinned before extracting weight admission (ADR 0025).
use super::tests::default_plane_embedding;
use super::*;

#[test]
fn legacy_metric_homogeneous_coefficients_preserve_weighted_magnitudes() {
    let poles = [[0., 0.], [1., 2.]];
    let curve = MetricCurve::Bezier(MetricBezier {
        poles: &poles,
        weights: Some(&[2., 1.]),
    });
    let actual = coefficients::embedded(
        curve,
        default_plane_embedding(),
        "owned baseline",
        &mut || false,
    )
    .unwrap();
    assert_eq!(actual[0], [0, 1].map(BigInt::from));
    assert_eq!(actual[1], [0, 2].map(BigInt::from));
    assert_eq!(actual[2], [0, 0].map(BigInt::from));
    assert_eq!(actual[3], [2, -1].map(BigInt::from));
}
#[test]
fn legacy_metric_rejects_signed_zero_and_nonfinite_weights_before_conversion() {
    for rejected in [-1., 0., f64::INFINITY] {
        let weights = [2., rejected];
        let curve = MetricCurve::Bezier(MetricBezier {
            poles: &[[0., 0.], [1., 2.]],
            weights: Some(&weights),
        });
        let error = coefficients::embedded(
            curve,
            default_plane_embedding(),
            "owned baseline",
            &mut || false,
        )
        .unwrap_err();
        assert!(
            matches!(error, ResidualError::Value { role: "weight", index: 1, value } if value == rejected)
        );
    }
}
