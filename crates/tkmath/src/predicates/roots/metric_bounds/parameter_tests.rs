// SPDX-License-Identifier: GPL-2.0-only
use super::tests::{default_plane_embedding, identity_embedding, query};
use super::*;
use crate::predicates::roots::{RootIsolationError, RootIsolationOptions};
fn transported<'a>(
    q: PlanarResidualQuery<'a>,
    map: MetricParameterMap,
) -> TransportedResidualQuery<'a> {
    TransportedResidualQuery {
        residual: q,
        maps: [map, MetricParameterMap::Identity],
    }
}
fn exact_bounds(a: f64, b: f64) -> [DyadicBound; 2] {
    [
        DyadicBound::from_f64(a).unwrap(),
        DyadicBound::from_f64(b).unwrap(),
    ]
}
#[test]
fn affine_bracket_keeps_endpoint_difference_that_binary64_subtraction_loses() {
    let map = MetricParameterMap::Endpoints([1., 1e16]);
    let actual = map.image_bounds(&exact_bounds(0.5, 0.5)).unwrap();
    assert!(actual[0] > DyadicBound::from_f64(5e15).unwrap());
    assert_eq!(
        map.image_bounds(&exact_bounds(0., 1.)).unwrap(),
        exact_bounds(1., 1e16)
    );
}
#[test]
fn negative_span_orders_image_without_changing_occurrence_direction() {
    assert_eq!(
        MetricParameterMap::Endpoints([3., -1.])
            .image_bounds(&exact_bounds(0.25, 0.75))
            .unwrap(),
        exact_bounds(0., 2.)
    );
}
#[test]
fn constant_and_identity_maps_keep_exact_singleton_and_subnormal() {
    let bounds = exact_bounds(f64::from_bits(1), 0.5);
    assert_eq!(
        MetricParameterMap::Identity.image_bounds(&bounds).unwrap(),
        bounds
    );
    assert_eq!(
        MetricParameterMap::Endpoints([2., 2.])
            .image_bounds(&bounds)
            .unwrap(),
        exact_bounds(2., 2.)
    );
}
fn affine_line<'a>(
    planar: &'a [[f64; 2]],
    origin: f64,
    native: [f64; 2],
    limit: f64,
) -> TransportedResidualQuery<'a> {
    let q = PlanarResidualQuery {
        curve: MetricCurve::Line {
            origin: [origin, 0., 0.],
            direction: [1., 0., 0.],
        },
        curve_embedding: identity_embedding(),
        pcurve: MetricCurve::Bezier(MetricBezier {
            poles: planar,
            weights: None,
        }),
        plane_embedding: default_plane_embedding(),
        plane_placement: identity_embedding(),
        interval: [0., 1.],
        spatial_limit: limit,
        options: RootIsolationOptions::new(0.001),
    };
    transported(q, MetricParameterMap::Endpoints(native))
}
#[test]
fn native_line_to_rounded_endpoint_basis_reports_real_construction_error() {
    let planar = [[1e16, 0.], [1e16, 0.]];
    let report = certify_transported_residual(affine_line(&planar, 1e16, [0., 1.], 0.), || false);
    assert!(matches!(
        report.decision,
        Ok(ResidualDecision::Exceeds { .. })
    ));
    let accepted = certify_transported_residual(affine_line(&planar, 1e16, [0., 1.], 1.), || false);
    assert!(matches!(
        accepted.decision,
        Ok(ResidualDecision::Within { .. })
    ));
}
#[test]
fn native_span_substitution_is_exact_even_when_rounded_span_would_pass() {
    let planar = [[1., 0.], [1e16, 0.]];
    let actual = certify_transported_residual(affine_line(&planar, 0., [1., 1e16], 0.), || false);
    assert!(matches!(
        actual.decision,
        Ok(ResidualDecision::Within { .. })
    ));
}
#[test]
fn both_rational_bezier_parameter_maps_use_actual_positive_weights() {
    let poles = [[0., 0., 0.], [2., 4., 0.], [4., 0., 0.]];
    let planar = [[0., 0.], [2., 4.], [4., 0.]];
    let weights = [1., 3., 2.];
    let q = super::tests::weighted_query(&poles, &planar, &weights, 0.);
    let q = TransportedResidualQuery {
        residual: q,
        maps: [MetricParameterMap::Endpoints([0.25, 0.75]); 2],
    };
    assert!(matches!(
        certify_transported_residual(q, || false).decision,
        Ok(ResidualDecision::Within { .. })
    ));
}
fn nonlinear_query<'a>(
    space: &'a [[f64; 3]],
    planar: &'a [[f64; 2]],
    budget: u16,
) -> TransportedResidualQuery<'a> {
    let mut q = query(space, planar, 2.);
    q.options.max_bisections = budget;
    q.options.max_width = 1e-6;
    transported(q, MetricParameterMap::Identity)
}
#[test]
fn failed_stationary_isolation_retains_actual_completed_split_count() {
    let space = [[0., 0., 0.], [0., 0., 1.], [0., 0., 0.3], [0., 0., 0.]];
    let planar = [[0., 0.]; 4];
    let report = certify_transported_residual(nonlinear_query(&space, &planar, 1), || false);
    assert!(report.decision.is_err());
    assert_eq!(report.bisections, 1);
    let retry = certify_transported_residual(nonlinear_query(&space, &planar, 1000), || false);
    assert!(matches!(
        retry.decision,
        Ok(ResidualDecision::Within { .. })
    ));
    assert!(retry.bisections > 1);
}
#[test]
fn transported_bezier_domain_and_nonfinite_map_are_typed_incomplete() {
    let space = [[0., 0., 0.], [1., 0., 0.]];
    let planar = [[0., 0.], [1., 0.]];
    let outside = transported(
        query(&space, &planar, 0.),
        MetricParameterMap::Endpoints([-0.1, 1.]),
    );
    assert!(matches!(
        certify_transported_residual(outside, || false).decision,
        Err(ResidualError::ParameterDomain { .. })
    ));
    let bad = transported(
        query(&space, &planar, 0.),
        MetricParameterMap::Endpoints([0., f64::NAN]),
    );
    assert!(matches!(
        certify_transported_residual(bad, || false).decision,
        Err(ResidualError::Value { .. })
    ));
}
#[test]
fn cancellation_before_capture_reports_zero_real_work_and_retry_succeeds() {
    let planar = [[2., 0.], [3., 0.]];
    let cancelled = certify_transported_residual(affine_line(&planar, 0., [2., 3.], 0.), || true);
    assert!(matches!(
        cancelled.decision,
        Err(ResidualError::Roots(RootIsolationError::Cancelled))
    ));
    assert_eq!(cancelled.bisections, 0);
    assert!(
        certify_transported_residual(affine_line(&planar, 0., [2., 3.], 0.), || false)
            .decision
            .is_ok()
    );
}

#[test]
fn transported_scaled_mirrored_placements_keep_native_coefficients_and_separate_origins() {
    let planar = [[2., 0.], [5., 0.]];
    let mut query = affine_line(&planar, 0., [2., 5.], 0.);
    let placement = MetricEmbedding {
        linear: [[0., 1., 0.], [1., 0., 0.], [0., 0., 1.]],
        origin: [7., 11., 13.],
        scale: -2.,
    };
    query.residual.curve_embedding = placement;
    query.residual.plane_placement = placement;
    assert!(matches!(
        certify_transported_residual(query, || false).decision,
        Ok(ResidualDecision::Within { .. })
    ));
}
struct CancelAfterPolls {
    remaining: usize,
}
impl CancelAfterPolls {
    fn poll(&mut self) -> bool {
        if self.remaining == 0 {
            return true;
        }
        self.remaining -= 1;
        false
    }
}
#[test]
fn cancellation_after_real_splits_preserves_work_without_admitting_a_decision() {
    let space = [[0., 0., 0.], [0., 0., 1.], [0., 0., 0.3], [0., 0., 0.]];
    let planar = [[0., 0.]; 4];
    let found = cancelled_after_split(&space, &planar);
    assert!(matches!(
        found.decision,
        Err(ResidualError::Roots(RootIsolationError::Cancelled))
            | Err(ResidualError::Signs(
                crate::predicates::roots::signs::RootSignError::Root {
                    cause: RootIsolationError::Cancelled,
                    ..
                }
            ))
    ));
}

fn cancelled_after_split(space: &[[f64; 3]], planar: &[[f64; 2]]) -> TransportedResidualReport {
    (1..1024)
        .find_map(|count| {
            let mut cancellation = CancelAfterPolls { remaining: count };
            let report = certify_transported_residual(nonlinear_query(space, planar, 1000), || {
                cancellation.poll()
            });
            (report.bisections > 0 && report.decision.is_err()).then_some(report)
        })
        .expect("expected actual split followed by cancellation")
}

#[test]
fn malformed_parameter_evidence_is_incomplete_and_constant_zero_budget_is_complete() {
    let reversed = exact_bounds(1., 0.);
    assert!(matches!(
        MetricParameterMap::Identity.image_bounds(&reversed),
        Err(ResidualError::ParameterBracket { .. })
    ));
    let planar = [[2., 0.], [2., 0.]];
    let mut q = affine_line(&planar, 0., [2., 2.], 0.);
    q.residual.options.max_bisections = 0;
    let result = certify_transported_residual(q, || false);
    assert_eq!(result.bisections, 0);
    assert!(matches!(
        result.decision,
        Ok(ResidualDecision::Within { .. })
    ));
}
