// SPDX-License-Identifier: GPL-2.0-only
use super::incidences::request;
use super::*;

fn rejected(query: BezierIncidenceQuery<'_>) -> DirectedBezierIncidenceFailure {
    certify_directed_bezier_incidences(DirectedBezierIncidenceQuery::v1(query), || false)
        .unwrap_err()
}
#[test]
fn new_support_sign_admission_is_explicit_and_does_not_widen_degree_policy() {
    let controls = [[0., -3.], [0., -1.], [0., 1.], [4., 3.]];
    let mut query = request(&controls);
    query.domain = BezierLineDomain::Support;
    query.curve.options.max_degree = 1;
    query.domain_sign.degree_limit = BezierDomainDegreeLimit::Explicit(1);
    assert!(certify_bezier_line_incidences(query, || false).is_ok());
    let failure = rejected(query);
    assert_eq!(failure.completion(), BezierIncidenceCompletion::Incomplete);
    assert_eq!(failure.source().root_options().max_degree, 1);
    assert!(
        matches!(failure.cause(), DirectedBezierIncidenceCause::Incidence(error)
        if matches!(error.cause(), BezierIncidenceCause::DomainSignAdmission(_)))
    );
    query.domain_sign.degree_limit = BezierDomainDegreeLimit::Explicit(3);
    let report = super::directed_incidences::directed(query);
    assert_eq!(
        report.incidence().degrees().effective_domain_limit(),
        Some(3)
    );
    assert!(report.incidence().degrees().domain_sign_applied());
}
#[test]
fn rejected_versions_and_invalid_world_limits_retain_original_request() {
    let mut query = DirectedBezierIncidenceQuery::v1(request(&[[0., -1.], [1., 1.]]));
    query.version = 9;
    let failure = certify_directed_bezier_incidences(query, || false).unwrap_err();
    assert_eq!(failure.version(), 9);
    assert_eq!(failure.cause(), &DirectedBezierIncidenceCause::Version(9));
    assert_eq!(failure.completion(), BezierIncidenceCompletion::Invalid);
    assert!(failure.to_string().contains("version 9; expected 1"));
    assert!(std::error::Error::source(&failure).is_some());
    query.incidence.max_world_error = -3.;
    let failure = rejected(query.incidence);
    assert_eq!(failure.source().requested_world_error(), -3.);
    assert_eq!(failure.completion(), BezierIncidenceCompletion::Invalid);
}
#[test]
fn construction_incomplete_preserves_same_support_root_and_requested_error() {
    let mut query = request(&[[0., -1.], [1., 2.]]);
    query.curve.weights = Some(&[2., 1.]);
    query.max_world_error = 1e-20;
    let failure = rejected(query);
    assert_eq!(failure.completion(), BezierIncidenceCompletion::Incomplete);
    let DirectedBezierIncidenceCause::Incidence(original) = failure.cause() else {
        panic!("{failure:?}")
    };
    assert_eq!(original.support_isolation().unwrap().roots().len(), 1);
    assert!(
        matches!(original.cause(), BezierIncidenceCause::ConstructionResolutionLimit { requested_error, effective_error_limit, .. }
        if *requested_error == 1e-20 && *effective_error_limit == 1e-20)
    );
}
#[test]
fn completed_empty_is_distinct_from_coincidence_and_cancellation() {
    let empty = super::directed_incidences::directed(request(&[[0., 1.], [1., 1.]]));
    assert!(empty.positions().is_empty());
    assert!(empty.incidence().contacts().is_empty());
    assert_eq!(
        empty.incidence().completion(),
        BezierIncidenceCompletion::Complete
    );
    assert_eq!(
        rejected(request(&[[0., 0.], [1., 0.]])).completion(),
        BezierIncidenceCompletion::Declined
    );
    let query = DirectedBezierIncidenceQuery::v1(request(&[[0., -1.], [1., 1.]]));
    let cancelled = certify_directed_bezier_incidences(query, || true).unwrap_err();
    assert_eq!(cancelled.completion(), BezierIncidenceCompletion::Cancelled);
}
#[test]
fn exhausted_root_budget_is_incomplete_without_a_partial_contact_list() {
    let mut query = request(&[[0., 1.], [1., -2.], [2., 1.]]);
    query.curve.options.max_bisections = 0;
    let failure = rejected(query);
    assert_eq!(failure.completion(), BezierIncidenceCompletion::Incomplete);
    assert_eq!(failure.source().root_options().max_bisections, 0);
    let DirectedBezierIncidenceCause::Incidence(original) = failure.cause() else {
        panic!("{failure:?}")
    };
    assert!(original.support_isolation().is_none());
    assert!(std::error::Error::source(failure.cause()).is_some());
}
#[test]
fn directed_version_does_not_replace_requested_domain_policy_version() {
    let mut query = request(&[[0., -1.], [1., 1.]]);
    query.domain_sign.version = 9;
    let failure = rejected(query);
    assert_eq!(failure.version(), 1);
    assert_eq!(failure.source().domain_sign().version, 9);
    assert_eq!(failure.completion(), BezierIncidenceCompletion::Invalid);
    assert!(
        matches!(failure.cause(), DirectedBezierIncidenceCause::Incidence(original)
        if original.cause() == &BezierIncidenceCause::DomainPolicyVersion(9))
    );
}
