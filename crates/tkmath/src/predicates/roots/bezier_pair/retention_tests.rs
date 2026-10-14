// SPDX-License-Identifier: GPL-2.0-only
//! Retained predicates explain empty/contact results without rerunning or source mutation.
use super::super::RootIsolationOptions;
use super::{membership_certificate::MembershipRule, operation::*, source::OwnedBezierSource};
fn proof(second_interval: [f64; 2]) -> PairOperationProof {
    let a = OwnedBezierSource::capture(&[[0., 0.], [0.5, 0.], [1., 1.]], None, [0., 1.]).unwrap();
    let b = OwnedBezierSource::capture(&[[0., 0.5], [0.5, 0.5], [1., -0.5]], None, second_interval)
        .unwrap();
    compute(
        [a, b],
        false,
        RootIsolationOptions::new(1e-9),
        1e-6,
        &mut || false,
    )
    .unwrap()
}
#[test]
fn retained_candidate_source_and_exact_query_signs_are_bound_to_actual_roots() {
    let proof = proof([0., 1.]);
    let certificate = &proof.graph().unwrap().candidate_certificate;
    assert_eq!(
        certificate.polynomial.primitive(2).unwrap().0,
        vec![(-1).into(), 0.into(), 4.into()]
    );
    assert_eq!(certificate.isolation.roots().len(), 1);
    assert_eq!(certificate.membership.accepted, vec![true]);
    assert_eq!(certificate.membership.queries.len(), 4);
    for query in &certificate.membership.queries {
        assert_eq!(query.evaluation.isolation(), &certificate.isolation);
        assert!(query.query.rule.accepts(query.evaluation.signs()[0]));
    }
    assert_eq!(
        certificate.membership.queries[0].query.polynomial.0,
        proof.graph().unwrap().projection.alpha.0
    );
}
#[test]
fn completed_empty_retains_excluded_root_and_exact_outside_domain_cause() {
    let proof = proof([0.75, 1.]);
    assert!(proof.contacts.is_empty());
    let certificate = &proof.graph().unwrap().candidate_certificate;
    assert_eq!(certificate.isolation.roots().len(), 1);
    assert_eq!(certificate.membership.accepted, vec![false]);
    let lower = &certificate.membership.queries[2];
    assert_eq!(lower.query.rule, MembershipRule::Nonnegative);
    assert_eq!(lower.evaluation.signs(), &[std::cmp::Ordering::Less]);
}
