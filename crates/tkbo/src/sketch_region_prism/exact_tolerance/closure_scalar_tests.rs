// SPDX-License-Identifier: GPL-2.0-only
//! Original source maps, singleton contact bounds and root options are independent operands.
use super::{closure_scalar::*, scalar::*, test_support::*};
use rustycad_tkmath::predicates::roots::{DyadicBound, RootIsolationError, metric_bounds::*};

#[test]
fn original_native_map_and_raw_vertex_transport_prove_exact_closure() {
    let query = closure_source_query();
    assert_eq!(query.residual.spatial_limit, 0.125);
    let mut placement = identity();
    placement.origin = [2., 0., 0.];
    let proof = certify_exact_source_point(query, bounds(0., 0.), [1., 0., 0.], placement)
        .ok()
        .unwrap();
    assert_eq!(
        proof.proven_squared_upper,
        DyadicBound::from_f64(0.).unwrap()
    );
    assert_eq!(proof.requested_spatial, 0.);
    assert_eq!(proof.effective_spatial, 0.);
    assert_eq!(proof.bisections, 0);
    assert_eq!(proof.effective_roots, closure_roots());
}

#[test]
fn an_identity_map_substitution_cannot_preserve_the_original_closure_proof() {
    let mut query = closure_source_query();
    query.maps[0] = MetricParameterMap::Identity;
    let mut placement = identity();
    placement.origin = [2., 0., 0.];
    let result = certify_exact_source_point(query, bounds(0., 0.), [1., 0., 0.], placement);
    assert_source_exceeds(result);
}

#[test]
fn expanded_reversed_and_other_singleton_contact_bounds_stay_incomplete() {
    for actual in [
        bounds(0., 1.),
        bounds(1., 0.),
        bounds(1., 1.),
        bounds(-1., 0.),
    ] {
        let result = certify_exact_source_point(
            closure_source_query(),
            actual.clone(),
            [3., 0., 0.],
            identity(),
        );
        assert!(
            matches!(result,Err(ExactClosureFailure::SourceBounds { actual: retained }) if retained==actual)
        );
    }
}

#[test]
fn original_root_policy_error_is_retained_without_replacing_the_policy() {
    let mut query = closure_source_query();
    query.residual.options.version = 999;
    let result = certify_exact_source_point(query, bounds(0., 0.), [3., 0., 0.], identity());
    let report = source_metric_failure(result);
    assert!(matches!(
        report.decision,
        Err(ResidualError::Roots(RootIsolationError::OptionsVersion(
            999
        )))
    ));
}

#[test]
fn closure_query_raw_transport_refuses_the_rounded_source_point_trap() {
    let shift = 9_007_199_254_740_992.;
    let mut query = closure_source_query();
    query.residual.curve = MetricCurve::Line {
        origin: [shift, 0., 0.],
        direction: [0.; 3],
    };
    let mut placement = identity();
    placement.origin = [shift, 0., 0.];
    let result = certify_exact_source_point(query, bounds(0., 0.), [1., 0., 0.], placement);
    assert_source_exceeds(result);
}

fn source_metric_failure(
    result: Result<ExactEndpointCertificate, ExactClosureFailure>,
) -> TransportedResidualReport {
    let ExactClosureFailure::Endpoint(failure) = result.err().unwrap() else {
        panic!("endpoint cause")
    };
    let ExactEndpointFailure::Residual(report) = *failure else {
        panic!("metric cause")
    };
    report
}

fn assert_source_exceeds(result: Result<ExactEndpointCertificate, ExactClosureFailure>) {
    let report = source_metric_failure(result);
    assert!(matches!(
        report.decision,
        Ok(ResidualDecision::Exceeds { .. })
    ));
}
