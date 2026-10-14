// SPDX-License-Identifier: GPL-2.0-only
//! Preserve source parameter transport while replacing only the constant point comparator.
use super::with_captured_point_comparator;
use rustycad_tkmath::predicates::roots::{DyadicBound, RootIsolationOptions, metric_bounds::*};

#[test]
fn placed_point_adapter_preserves_source_maps_options_interval_and_placement() {
    let original = authored_query();
    let expected_maps = original.maps;
    let expected_options = original.residual.options;
    let mut placement = identity();
    placement.origin = [2., 0., 0.];
    let query = with_captured_point_comparator(original, [1., 0., 0.], placement, 0.);
    assert_eq!(query.maps, expected_maps);
    assert_eq!(query.residual.options, expected_options);
    assert_eq!(query.residual.interval, [0., 1.]);
    assert_eq!(query.residual.curve_embedding.linear, identity().linear);
    assert_eq!(query.residual.plane_embedding.origin, [1., 0., 0.]);
    assert_eq!(query.residual.plane_placement.origin, [2., 0., 0.]);
    assert_eq!(query.residual.spatial_limit, 0.);
    let report = certify_transported_residual_on_bounds(query, zero_bounds(), || false);
    assert_zero_report(report);
}

#[test]
fn placed_point_adapter_keeps_explicit_nonzero_request_separate_from_source_options() {
    let query = with_captured_point_comparator(authored_query(), [2., 0., 0.], identity(), 1.);
    let report = certify_transported_residual_on_bounds(query, zero_bounds(), || false);
    assert!(
        matches!(report.decision, Ok(ResidualDecision::Within { proven_squared_upper })
        if proven_squared_upper == DyadicBound::from_f64(1.).unwrap())
    );
}

fn authored_query() -> TransportedResidualQuery<'static> {
    TransportedResidualQuery {
        residual: authored_residual(),
        maps: [
            MetricParameterMap::Endpoints([3., 5.]),
            MetricParameterMap::Endpoints([7., 11.]),
        ],
    }
}

fn authored_residual() -> PlanarResidualQuery<'static> {
    let curve = MetricCurve::Line {
        origin: [0.; 3],
        direction: [1., 0., 0.],
    };
    let pcurve = MetricCurve::Line {
        origin: [99.; 2],
        direction: [3.; 2],
    };
    let plane_embedding = MetricEmbedding {
        linear: [[1.; 2]; 3],
        origin: [99.; 3],
        scale: 2.,
    };
    PlanarResidualQuery {
        curve,
        pcurve,
        curve_embedding: identity(),
        plane_embedding,
        plane_placement: identity(),
        interval: [0., 1.],
        spatial_limit: 7.,
        options: RootIsolationOptions::new(0.125),
    }
}

fn identity() -> MetricEmbedding<3> {
    MetricEmbedding {
        linear: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        origin: [0.; 3],
        scale: 1.,
    }
}

fn zero_bounds() -> [DyadicBound; 2] {
    [
        DyadicBound::from_f64(0.).unwrap(),
        DyadicBound::from_f64(0.).unwrap(),
    ]
}

fn assert_zero_report(report: TransportedResidualReport) {
    assert_eq!(report.bisections, 0);
    assert!(
        matches!(report.decision, Ok(ResidualDecision::Within { proven_squared_upper })
        if proven_squared_upper == DyadicBound::from_f64(0.).unwrap())
    );
}
