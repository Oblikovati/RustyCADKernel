// SPDX-License-Identifier: GPL-2.0-only
use super::{admission::*, diagnostics::*, names::*, pipeline::*, test_fixture::*};
#[test]
fn empty_request_is_declined_without_callbacks_or_geometry() {
    let mut request = request();
    request.sources.clear();
    let mut observer = RequestObserver::default();
    let failure = prepare_owned_polynomial_prism(request, &mut observer)
        .err()
        .unwrap();
    assert_eq!(failure.class, OwnedPreparationClass::Declined);
    assert!(matches!(
        failure.cause,
        OwnedPreparationCause::Admission(OwnedPrismAdmissionCause::EmptyInput)
    ));
    assert!(observer.seen.is_empty());
    assert!(failure.retained.source.is_none());
}
#[test]
fn multiple_sources_refuse_with_all_caller_ids_without_selecting_one() {
    let mut request = request();
    let mut second = request.sources[0].clone();
    second.source = CallerBoundaryId(900);
    request.sources.push(second);
    let failure = prepare_owned_polynomial_prism(request, &mut RequestObserver::default())
        .err()
        .unwrap();
    assert_eq!(failure.class, OwnedPreparationClass::Incomplete);
    assert!(
        matches!(failure.cause, OwnedPreparationCause::Admission(OwnedPrismAdmissionCause::MultipleBoundaries { actual, supported: 1 })
        if actual == vec![CallerBoundaryId(0), CallerBoundaryId(900)])
    );
    assert!(failure.retained.source.is_none());
}
#[test]
fn unsupported_version_and_open_boundary_remain_incomplete() {
    let mut version = request();
    version.version = 2;
    let mut open = request();
    open.sources[0].world_poles[3][0] = 1.;
    for request in [version, open] {
        let failure = prepare_owned_polynomial_prism(request, &mut RequestObserver::default())
            .err()
            .unwrap();
        assert_eq!(failure.class, OwnedPreparationClass::Incomplete);
        assert!(failure.retained.source.is_none());
    }
}
#[test]
fn invalid_numeric_input_and_endpoint_names_fail_without_native_construction() {
    for request in invalid_requests() {
        let failure = prepare_owned_polynomial_prism(request, &mut RequestObserver::default())
            .err()
            .unwrap();
        assert_eq!(failure.class, OwnedPreparationClass::Failed);
        assert!(failure.retained.source.is_none());
    }
}
fn invalid_requests() -> Vec<super::request::OwnedPolynomialPrismRequest> {
    let mut values = vec![request(); 6];
    values[0].extrusion = [0.; 3];
    values[1].extrusion[0] = f64::INFINITY;
    values[2].sources[0].world_poles[1][2] = f64::NAN;
    values[3].sources[0].world_poles.clear();
    values[4].sources[0].native_end = values[4].sources[0].native_start;
    values[5].selection = SemanticSelectionIntent::BoundedRegionContaining {
        seed: [f64::NAN, 0.],
    };
    values
}
#[test]
fn requested_revisions_and_scalar_policy_survive_refusal_unchanged() {
    let mut request = request();
    request.extrusion = [0.; 3];
    let original = request.clone();
    let failure = prepare_owned_polynomial_prism(request, &mut RequestObserver::default())
        .err()
        .unwrap();
    assert_eq!(
        failure.retained.request.sources[0].revision,
        original.sources[0].revision
    );
    assert_eq!(
        failure.retained.request.policy.publication,
        original.policy.publication
    );
    assert_eq!(
        failure.retained.request.sources[0].world_poles,
        original.sources[0].world_poles
    );
}

#[test]
fn nonfinite_declared_plane_refuses_before_constructor_with_exact_operand() {
    let mut request = request();
    request
        .declared_plane
        .set_location(rustycad_tkmath::gp::Point3::new(0., f64::NAN, 0.));
    let failure = prepare_owned_polynomial_prism(request, &mut RequestObserver::default())
        .err()
        .unwrap();
    assert_eq!(failure.class, OwnedPreparationClass::Failed);
    assert!(matches!(failure.cause, OwnedPreparationCause::Admission(
        OwnedPrismAdmissionCause::NonfinitePlane { component: DeclaredPlaneComponent::Origin, axis: 1, actual }) if actual.is_nan()));
    assert!(failure.retained.source.is_none());
}
