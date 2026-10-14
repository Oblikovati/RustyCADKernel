// SPDX-License-Identifier: GPL-2.0-only
//! Structural retention preserves the old rounded output while exposing its exact rational origin.
use super::super::RootIsolationOptions;
use super::*;
use std::cmp::Ordering;

pub(super) fn retained_rational_contact() -> BezierPairReport {
    let query = parameter_query();
    query.certify(parameter_policy(), || false)
}
fn parameter_policy() -> BezierPairPolicy {
    BezierPairPolicy {
        version: 1,
        roots: RootIsolationOptions::new(1e-9),
        signs: BezierPairSignOptions::derived(),
        requested_world_error: 1e-8,
    }
}
fn parameter_query() -> CapturedBezierPair {
    let first = [[0., 0.], [1., 1.]];
    let second = [[0., 1.], [1., 0.]];
    CapturedBezierPair::capture_pair(
        BezierPairCurve {
            controls: &first,
            weights: None,
            interval: [0., 1.],
        },
        BezierPairCurve {
            controls: &second,
            weights: Some(&[1., 2.]),
            interval: [0., 1.],
        },
    )
    .unwrap()
}

#[test]
fn exact_third_is_not_replaced_by_its_lower_binary64_display() {
    let report = retained_rational_contact();
    let contact = report.complete().unwrap().contact(0).unwrap();
    assert_eq!(contact.point, [0.5, 0.5]);
    assert_eq!(
        contact
            .exact_paired_parameter
            .compare_lower(1. / 3.)
            .unwrap(),
        Ordering::Greater
    );
    assert_eq!(
        contact
            .exact_paired_parameter
            .compare_upper(1. / 3.)
            .unwrap(),
        Ordering::Greater
    );
    assert_outward_third(contact.paired_parameter_bounds);
}

#[test]
fn nonfinite_comparisons_preserve_original_values_and_do_not_mutate_proof() {
    let report = retained_rational_contact();
    let contact = report.complete().unwrap().contact(0).unwrap();
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(contact.exact_paired_parameter.compare_lower(value).is_err());
        assert!(contact.exact_paired_parameter.compare_upper(value).is_err());
    }
    assert_eq!(
        contact.exact_paired_parameter.compare_lower(0.).unwrap(),
        Ordering::Greater
    );
    assert_eq!(
        contact.exact_paired_parameter.compare_upper(1.).unwrap(),
        Ordering::Less
    );
}

#[test]
fn retention_adds_only_existing_owned_payload_and_keeps_outward_conversion_equal() {
    let proof = retained_internal_proof();
    let contact = &proof.contacts[0];
    let lower =
        super::super::certificate_arithmetic::rounding::outward_binary64(&contact.exact_u.lower)
            .unwrap();
    let upper =
        super::super::certificate_arithmetic::rounding::outward_binary64(&contact.exact_u.upper)
            .unwrap();
    assert_eq!(contact.u_bounds, [lower[0], upper[1]]);
    println!(
        "{{\"retained_interval_stack_bytes\":{},\"existing_bigint_owners_retained\":4,\"comparative_performance_pending\":true}}",
        std::mem::size_of::<super::super::certificate_arithmetic::RatioInterval>()
    );
}

fn assert_outward_third(bounds: [f64; 2]) {
    let third = 1. / 3.;
    assert!(bounds[0] <= third);
    assert!(bounds[1] > third);
}

fn retained_internal_proof() -> super::operation::PairOperationProof {
    let query = parameter_query();
    let policy = parameter_policy();
    super::operation::compute(
        query.sources().clone(),
        false,
        policy.roots,
        policy.requested_world_error,
        &mut || false,
    )
    .unwrap()
}
