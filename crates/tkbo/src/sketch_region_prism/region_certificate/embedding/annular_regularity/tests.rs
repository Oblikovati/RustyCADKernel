// SPDX-License-Identifier: GPL-2.0-only
//! Closed-form derivative fixtures exercise budgets without granting annular binding authority.
use super::*;
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use rustycad_tkmath::predicates::roots::{RootIsolationOptions, metric_bounds::*};

fn space(poles: Vec<[f64; 3]>) -> CapturedSpaceRepresentation {
    CapturedSpaceRepresentation {
        curve: CapturedMetricCurve::Bezier {
            poles,
            weights: None,
        },
        interval: [0., 1.],
        placement: MetricEmbedding {
            linear: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
            origin: [0.; 3],
            scale: 1.,
        },
        same_parameter: true,
    }
}
fn options(total_bisections: u32) -> AnnularRegularityOptions {
    AnnularRegularityOptions {
        boundary: BezierRegularityOptions::v1(RootIsolationOptions::new(1e-12)),
        total_bisections,
    }
}
fn operands<'a>(
    first: &'a CapturedSpaceRepresentation,
    second: &'a CapturedSpaceRepresentation,
) -> [AnnularRegularityOperand<'a>; 2] {
    [
        (RegionSourceId(202), 81, first),
        (RegionSourceId(101), 79, second),
    ]
    .map(|(source, curve_id, space)| AnnularRegularityOperand {
        source,
        curve_id,
        space,
    })
}
fn regular() -> CapturedSpaceRepresentation {
    space(vec![[0.; 3], [1., 0., 0.]])
}
fn stationary() -> CapturedSpaceRepresentation {
    // x=(3t-1)^2 has one stationary parameter 1/3; its dyadic isolation needs subdivision.
    space(vec![[1., 0., 0.], [-2., 0., 0.], [4., 0., 0.]])
}
#[test]
fn exact_linear_derivatives_need_no_subdivisions_and_keep_source_keys() {
    let source = regular();
    let requested = options(0);
    let report = certify_boundaries(operands(&source, &source), requested, &mut || false)
        .unwrap_or_else(|_| panic!("constant nonzero derivatives require no subdivision"));
    assert_eq!(report.requested, requested);
    assert_eq!(report.total_bisections, 0);
    assert_eq!(
        report.boundaries.each_ref().map(|r| r.source),
        [RegionSourceId(202), RegionSourceId(101)]
    );
    assert_eq!(report.boundaries[0].report.source().curve_id(), 81);
    assert_eq!(report.boundaries[1].report.source().curve_id(), 79);
    assert_zero_work_regular(&report);
}
fn assert_zero_work_regular(report: &AnnularRegularityReport) {
    for boundary in &report.boundaries {
        assert_eq!(boundary.effective.roots.max_bisections, 0);
        assert_eq!(boundary.report.source().options(), boundary.effective);
        assert!(matches!(
            boundary.report.decision(),
            BezierRegularityDecision::Regular { .. }
        ));
    }
}
#[test]
fn aggregate_remainder_clamps_only_subdivision_cap() {
    let mut requested = options(17);
    requested.boundary.roots.max_bisections = 11;
    for (used, expected) in [(0, 11), (6, 11), (7, 10), (16, 1), (17, 0)] {
        let mut expected_policy = requested.boundary;
        expected_policy.roots.max_bisections = expected;
        assert_eq!(effective_options(requested, used), expected_policy);
    }
    let mut large = requested;
    large.total_bisections = u32::MAX;
    assert_eq!(effective_options(large, 0), requested.boundary);
}
#[test]
fn second_exhaustion_retains_first_proof_and_original_typed_cause() {
    let first = regular();
    let second = stationary();
    let requested = options(0);
    let failure = certify_boundaries(operands(&first, &second), requested, &mut || false)
        .err()
        .unwrap();
    assert_eq!(failure.requested, requested);
    assert_eq!(failure.source, RegionSourceId(101));
    assert_eq!(failure.completed.len(), 1);
    assert_eq!(failure.completed[0].source, RegionSourceId(202));
    assert_eq!(failure.effective.roots.max_bisections, 0);
    assert_eq!(failure.total_bisections, 0);
    let PrismEmbeddingCause::Regularity(cause) = &failure.cause else {
        panic!("typed exhausted isolation expected")
    };
    assert_eq!(cause.completion(), BezierRegularityCompletion::Incomplete);
    assert_eq!(cause.source().curve_id(), 79);
    assert_eq!(cause.source().options(), failure.effective);
}
#[test]
fn actual_failed_subdivisions_are_counted_without_retrying_next_boundary() {
    let first = stationary();
    let second = regular();
    let failure = certify_boundaries(operands(&first, &second), options(1), &mut || false)
        .err()
        .unwrap();
    assert!(failure.completed.is_empty());
    assert_eq!(failure.source, RegionSourceId(202));
    assert_eq!(failure.effective.roots.max_bisections, 1);
    assert_eq!(failure.total_bisections, 1);
    let PrismEmbeddingCause::Regularity(cause) = &failure.cause else {
        panic!("typed exhausted isolation expected")
    };
    assert_eq!(cause.completion(), BezierRegularityCompletion::Incomplete);
    assert_eq!(cause.bisections(), 1);
}
#[test]
fn stationary_classification_is_retained_as_a_refusal() {
    let first = regular();
    let second = space(vec![[0.; 3]]);
    let failure = certify_boundaries(operands(&first, &second), options(0), &mut || false)
        .err()
        .unwrap();
    assert_eq!(failure.completed.len(), 1);
    assert_eq!(failure.total_bisections, 0);
    let PrismEmbeddingCause::NonRegular(report) = &failure.cause else {
        panic!("constant derivative is stationary")
    };
    assert_eq!(
        report.decision(),
        &BezierRegularityDecision::EverywhereStationary
    );
}
#[test]
fn cancellation_retains_source_and_never_returns_partial_success() {
    let first = regular();
    let failure = certify_boundaries(operands(&first, &first), options(12), &mut || true)
        .err()
        .unwrap();
    assert!(failure.completed.is_empty());
    assert_eq!(failure.source, RegionSourceId(202));
    let PrismEmbeddingCause::Regularity(cause) = &failure.cause else {
        panic!("typed cancellation expected")
    };
    assert_eq!(cause.completion(), BezierRegularityCompletion::Cancelled);
}
#[test]
fn explicit_weights_keep_the_original_representation_without_root_work() {
    let first = regular();
    let mut second = regular();
    if let CapturedMetricCurve::Bezier { weights, .. } = &mut second.curve {
        *weights = Some(vec![1., 1.]);
    }
    let failure = certify_boundaries(operands(&first, &second), options(12), &mut || false)
        .err()
        .unwrap();
    assert_eq!(failure.completed.len(), 1);
    assert_eq!(failure.total_bisections, 0);
    assert!(
        matches!(&failure.cause, PrismEmbeddingCause::PolynomialRequired { curve_id:79, explicit_weights } if explicit_weights == &[1.,1.])
    );
}
#[test]
fn reports_are_owned_send_sync_without_local_geometry_borrows() {
    static_assertions::assert_impl_all!(AnnularRegularityReport: Send, Sync);
    static_assertions::assert_impl_all!(AnnularRegularityOptions: Send, Sync);
    static_assertions::assert_impl_all!(AnnularRegularityOperand<'static>: Send, Sync);
    let report = {
        let first = regular();
        certify_boundaries(operands(&first, &first), options(0), &mut || false)
            .unwrap_or_else(|_| panic!("regular fixture"))
    };
    assert_eq!(report.boundaries[0].report.source().curve_id(), 81);
}
