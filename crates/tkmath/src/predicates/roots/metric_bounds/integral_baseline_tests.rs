// SPDX-License-Identifier: GPL-2.0-only
//! Pre-extension owned baselines for existing magnitude-preserving metric primitives.
use super::super::polynomial::IntegerPolynomial;
use super::*;

#[test]
fn cubic_coefficients_and_derivatives_keep_their_exact_magnitudes() {
    let poles = [[0., 0.], [4., 0.], [0., 4.], [0., 0.]];
    let curve = MetricCurve::Bezier(MetricBezier {
        poles: &poles,
        weights: None,
    });
    let actual =
        coefficients::embedded(curve, baseline_frame(), "baseline", &mut || false).unwrap();
    assert_eq!(actual[0], [0, 12, -24, 12].map(BigInt::from));
    assert_eq!(actual[1], [0, 0, 12, -12].map(BigInt::from));
    assert_eq!(actual[3], [1, 0, 0, 0].map(BigInt::from));
    let derivative = IntegerPolynomial(actual[0].clone()).derivative();
    assert_eq!(derivative.0, [12, -48, 36].map(BigInt::from));
}

#[test]
fn fixed_affine_sweep_jacobian_baseline_is_exactly_two() {
    let frame = MetricEmbedding {
        linear: [[1., 0.], [0., 1.], [0., 0.]],
        origin: [0.; 3],
        scale: 1.,
    };
    let placement = MetricEmbedding {
        linear: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        origin: [0.; 3],
        scale: 1.,
    };
    let actual = affine_prism_jacobian(frame, placement, [0., 0., 2.], || false).unwrap();
    assert_eq!(actual.compare_f64(2.), Ok(std::cmp::Ordering::Equal));
}

fn baseline_frame() -> MetricEmbedding<2> {
    MetricEmbedding {
        linear: [[1., 0.], [0., 1.], [0., 0.]],
        origin: [0.; 3],
        scale: 1.,
    }
}
