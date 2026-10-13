// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::RootIsolationOptions;
use super::super::BezierLineQuery;
use super::*;
use crate::predicates::roots::certificate_arithmetic::ExactRatio;

fn rational_query(controls: &[[f64; 2]], domain: BezierLineDomain) -> BezierIncidenceQuery<'_> {
    let curve = BezierLineQuery {
        controls,
        weights: Some(&[1., 2., 1.]),
        line: [[0., 0.], [0.5, 0.]],
        interval: [0., 1.],
        options: RootIsolationOptions::new(0.001),
    };
    BezierIncidenceQuery {
        curve_id: 17,
        line_id: 23,
        curve,
        domain,
        domain_sign: BezierDomainSignOptions::source_basis(),
        max_world_error: 1e-8,
    }
}
#[test]
fn complete_rational_support_and_segment_roots_have_independent_world_enclosures() {
    let controls = [[0., 1.], [0.5, -1.], [1., 1.]];
    let support = certify_bezier_line_incidences(
        rational_query(&controls, BezierLineDomain::Support),
        || false,
    )
    .unwrap();
    assert_eq!(support.contacts.len(), 2);
    assert!(support.bisections > support.support_isolation.bisections());
    for (index, contact) in support.contacts.iter().enumerate() {
        certify_world_root(index, contact);
    }
    let segment = certify_bezier_line_incidences(
        rational_query(&controls, BezierLineDomain::Segment),
        || false,
    )
    .unwrap();
    assert_eq!(segment.contacts, support.contacts[..1]);
    assert_eq!(segment.in_domain, [true, false]);
}
fn certify_world_root(index: usize, contact: &CertifiedBezierIncidence) {
    use std::cmp::Ordering::{Greater, Less};
    assert!(contact.achieved_world_error <= 1e-8);
    let lower = world_root_polynomial(contact.world_bounds[0][0]);
    let upper = world_root_polynomial(contact.world_bounds[0][1]);
    assert_eq!(lower.sign(), if index == 0 { Greater } else { Less });
    assert_eq!(upper.sign(), if index == 0 { Less } else { Greater });
    assert!(contact.world_bounds[1][0] <= 0. && contact.world_bounds[1][1] >= 0.);
}
fn world_root_polynomial(value: f64) -> ExactRatio {
    // Independently eliminate t from Nx/D and 6t²−6t+1: 64x²−64x+13.
    let x = ExactRatio::from_finite(value);
    let sixty_four = ExactRatio::from_finite(64.);
    x.multiply(&x)
        .multiply(&sixty_four)
        .subtract(&x.multiply(&sixty_four))
        .add(&ExactRatio::from_finite(13.))
}
#[test]
fn exhausted_world_refinement_cannot_be_a_completed_parameter_only_result() {
    let controls = [[0., 1.], [0.5, -1.], [1., 1.]];
    let mut query = rational_query(&controls, BezierLineDomain::Support);
    let initial =
        domain::isolate_domain(query.curve, query.domain_sign, query.domain, &mut || false)
            .unwrap();
    query.curve.options.max_bisections = initial.isolation.bisections();
    let error = certify_bezier_line_incidences(query, || false).unwrap_err();
    assert!(matches!(
        error.cause(),
        BezierIncidenceCause::RefinementExhausted { .. }
    ));
    assert_eq!(error.support_isolation().unwrap().roots().len(), 2);
    assert!(error.last_root().is_some());
    assert_eq!(error.source().curve_id(), 17);
}
