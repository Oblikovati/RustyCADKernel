// SPDX-License-Identifier: GPL-2.0-only
//! Exact original brackets and native maps remain distinct from rounded construction scalars.
use super::*;
use crate::predicates::roots::DyadicBound;

fn identity_embedding<const DIM: usize>() -> MetricEmbedding<DIM> {
    let linear = std::array::from_fn(|row| {
        std::array::from_fn(|column| if row == column { 1. } else { 0. })
    });
    MetricEmbedding {
        linear,
        origin: [0.; 3],
        scale: 1.,
    }
}
fn transported_point_query(point: [f64; 3], limit: f64) -> TransportedResidualQuery<'static> {
    let mut point_frame = identity_embedding::<2>();
    point_frame.linear = [[0.; 2]; 3];
    point_frame.origin = point;
    TransportedResidualQuery {
        maps: [
            MetricParameterMap::Endpoints([2., 5.]),
            MetricParameterMap::Identity,
        ],
        residual: PlanarResidualQuery {
            curve: MetricCurve::Line {
                origin: [0.; 3],
                direction: [1., 0., 0.],
            },
            curve_embedding: identity_embedding(),
            pcurve: MetricCurve::Line {
                origin: [0.; 2],
                direction: [0.; 2],
            },
            plane_embedding: point_frame,
            plane_placement: identity_embedding(),
            interval: [0., 1.],
            spatial_limit: limit,
            options: super::super::RootIsolationOptions::new(1e-9),
        },
    }
}
fn half_bounds() -> [DyadicBound; 2] {
    let half = DyadicBound::from_f64(0.5).unwrap();
    [half.clone(), half]
}
#[test]
fn exact_contact_subinterval_uses_original_native_affine_parameterization() {
    let report = certify_transported_residual_on_bounds(
        transported_point_query([3.5, 0., 0.], 0.),
        half_bounds(),
        || false,
    );
    assert!(
        matches!(report.decision, Ok(ResidualDecision::Within { proven_squared_upper }) if proven_squared_upper.compare_f64(0.).unwrap().is_eq())
    );
    let full = certify_transported_residual(transported_point_query([3.5, 0., 0.], 0.), || false);
    assert!(matches!(
        full.decision,
        Ok(ResidualDecision::Exceeds { .. })
    ));
    assert_eq!(report.bisections, 0);
}
#[test]
fn approximate_point_does_not_change_original_exact_parameter_bracket() {
    let bracket = half_bounds();
    let report = certify_transported_residual_on_bounds(
        transported_point_query([3.4999999999999, 0., 0.], 0.),
        bracket.clone(),
        || false,
    );
    assert!(matches!(
        report.decision,
        Ok(ResidualDecision::Exceeds { .. })
    ));
    let allowed = certify_transported_residual_on_bounds(
        transported_point_query([3.4999999999999, 0., 0.], 1e-8),
        bracket.clone(),
        || false,
    );
    assert!(matches!(
        allowed.decision,
        Ok(ResidualDecision::Within { .. })
    ));
    assert_eq!(bracket, half_bounds());
}
#[test]
fn exact_domain_rejection_and_cancellation_keep_actual_failure_and_work() {
    let outside = [
        DyadicBound::from_f64(-0.01).unwrap(),
        DyadicBound::from_f64(0.5).unwrap(),
    ];
    let rejected = certify_transported_residual_on_bounds(
        transported_point_query([3.5, 0., 0.], 0.),
        outside,
        || false,
    );
    assert!(matches!(
        rejected.decision,
        Err(ResidualError::ExactDomain { .. })
    ));
    let stopped = certify_transported_residual_on_bounds(
        transported_point_query([3.5, 0., 0.], 0.),
        half_bounds(),
        || true,
    );
    assert!(matches!(
        stopped.decision,
        Err(ResidualError::Roots(
            super::super::RootIsolationError::Cancelled
        ))
    ));
    assert_eq!(stopped.bisections, 0);
}
