// SPDX-License-Identifier: GPL-2.0-only
use super::incidences::request;
use super::*;

#[test]
fn cubic_domain_query_has_explicit_admission_without_widening_linear_root_policy() {
    let controls = [[0., -3.], [0., -1.], [0., 1.], [4., 3.]];
    let mut query = request(&controls);
    query.curve.options.max_degree = 1;
    assert!(matches!(
        intersect_bezier_line(query.curve, || false),
        Ok(BezierLineContact::Isolated(_))
    ));
    assert!(classify_bezier_ray_contacts(query.curve, || false).is_err());
    let report = certify_bezier_line_incidences(query, || false).unwrap();
    assert_eq!(report.contacts()[0].point(), [0.5, 0.]);
    assert_eq!(report.source().root_options().max_degree, 1);
    assert_eq!(report.degrees().normal(), 1);
    assert_eq!(report.degrees().origin_conversion(), Some(3));
    assert_eq!(report.degrees().effective_domain_limit(), 3);
    verify_explicit_domain_limit(query);
}
fn verify_explicit_domain_limit(mut query: BezierIncidenceQuery<'_>) {
    query.domain_sign.degree_limit = BezierDomainDegreeLimit::Explicit(1);
    let error = certify_bezier_line_incidences(query, || false).unwrap_err();
    assert_eq!(error.completion(), BezierIncidenceCompletion::Incomplete);
    assert!(matches!(
        error.cause(),
        BezierIncidenceCause::DomainSignAdmission(_)
    ));
    query.domain_sign.degree_limit = BezierDomainDegreeLimit::Explicit(3);
    let report = certify_bezier_line_incidences(query, || false).unwrap();
    assert_eq!(
        report.source().domain_sign().degree_limit,
        BezierDomainDegreeLimit::Explicit(3)
    );
    assert_eq!(report.degrees().effective_domain_limit(), 3);
}
#[test]
fn unsupported_domain_policy_preserves_offending_version_and_original_root_limit() {
    let mut query = request(&[[0., -1.], [1., 1.]]);
    query.domain_sign.version = 2;
    let error = certify_bezier_line_incidences(query, || false).unwrap_err();
    assert_eq!(error.completion(), BezierIncidenceCompletion::Invalid);
    assert_eq!(error.cause(), &BezierIncidenceCause::DomainPolicyVersion(2));
    assert!(error.to_string().contains("domain policy version 2"));
}
