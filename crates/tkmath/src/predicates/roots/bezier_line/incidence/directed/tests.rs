// SPDX-License-Identifier: GPL-2.0-only
use super::super::{BezierDomainSignOptions, BezierIncidenceQuery};
use super::*;
use crate::predicates::roots::{RootIsolationOptions, bezier_line::BezierLineQuery};

#[test]
fn every_sign_pair_is_classified_or_typed_incomplete() {
    use Ordering::*;
    for origin in [Less, Equal, Greater] {
        for terminal in [Less, Equal, Greater] {
            check_sign_pair(origin, terminal);
        }
    }
}
fn check_sign_pair(origin: Ordering, terminal: Ordering) {
    let actual = endpoint_position(7, origin, terminal);
    assert_eq!(
        actual.is_ok(),
        origin == Ordering::Greater || terminal == Ordering::Less
    );
    if let Err(cause) = actual {
        assert_eq!(
            cause,
            DirectedBezierIncidenceCause::EndpointOrder {
                support_root_index: 7,
                origin,
                terminal,
            }
        );
        assert!(cause.to_string().contains("expected"));
    }
}
#[test]
fn mismatched_sign_cardinality_never_truncates_to_success() {
    let query = alignment_query();
    let (source, mut domain) =
        super::super::prepare_incidence(query, query.domain, &mut || false).unwrap();
    domain.end_signs.clear();
    let cause = endpoint_positions(&domain).unwrap_err();
    assert_eq!(
        cause,
        DirectedBezierIncidenceCause::Alignment {
            roots: 1,
            origins: 1,
            terminals: 0
        }
    );
    verify_alignment_failure(source, cause);
}
fn alignment_query() -> BezierIncidenceQuery<'static> {
    let curve = BezierLineQuery {
        controls: &[[0., -1.], [1., 1.]],
        weights: None,
        line: [[0., 0.], [1., 0.]],
        interval: [0., 1.],
        options: RootIsolationOptions::new(0.001),
    };
    BezierIncidenceQuery {
        curve_id: 1,
        line_id: 2,
        curve,
        domain: BezierLineDomain::Segment,
        domain_sign: BezierDomainSignOptions::source_basis(),
        max_world_error: 1e-8,
    }
}
fn verify_alignment_failure(source: BezierIncidenceSource, cause: DirectedBezierIncidenceCause) {
    let failure = DirectedBezierIncidenceFailure {
        version: 1,
        source: Box::new(source),
        cause,
    };
    assert_eq!(
        failure.completion(),
        super::super::BezierIncidenceCompletion::Incomplete
    );
    assert!(failure.to_string().contains("expected (1, 1)"));
}
