// SPDX-License-Identifier: GPL-2.0-only
//! Exact constant exclusions close actual admitted self domains without synthetic graph evidence.
use super::paired_tests::*;
use super::*;
use rustycad_tkmath::predicates::roots::bezier_pair::*;

#[test]
fn actual_polynomial_line_self_has_complete_source_bound_constant_exclusion() {
    let line = bezier_shape(&[[0., 0.], [1., 0.]], None);
    let census = paired(&[line]);
    assert_constant_self(&census);
    assert!(!has_obligation(
        &census,
        CertificationObligation::ExhaustivePairAndSelfDomains
    ));
    assert_remaining(&census);
}

#[test]
fn regular_rational_line_self_keeps_actual_denominator_and_owned_weights() {
    let line = bezier_shape(&[[0., 0.], [0.5, 0.]], Some(&[1., 2.]));
    let census = paired(&[line]);
    assert_constant_self(&census);
    let proof = super::paired_binding::domain_certificate(&census, &census.domains[0]).unwrap();
    assert_eq!(proof.captured_curves()[0].weights.unwrap(), [1., 2.]);
    assert!(
        proof
            .denominators()
            .iter()
            .all(|denominator| denominator.roots().is_empty())
    );
    assert_eq!(census.bisections, 0);
}

#[test]
fn reversed_line_alias_keeps_cross_occurrence_coincidence_incomplete() {
    let line = bezier_shape(&[[0., 0.], [1., 0.]], None);
    let census = paired(&[line.clone(), line.reversed()]);
    assert_eq!(census.domains.len(), 3);
    assert!(super::paired_binding::domain_certificate(&census, &census.domains[0]).is_some());
    assert!(super::paired_binding::domain_certificate(&census, &census.domains[2]).is_some());
    assert!(matches!(
        census.domains[1].evidence,
        DomainEvidence::Unsupported(CensusGap::SamePhysicalCurve)
    ));
    assert!(has_obligation(
        &census,
        CertificationObligation::ExhaustivePairAndSelfDomains
    ));
    assert_eq!(census.paired.as_ref().unwrap().reused_reports, 1);
}

fn assert_constant_self(census: &BoundaryCensus) {
    let proof = super::paired_binding::domain_certificate(census, &census.domains[0]).unwrap();
    assert_eq!(proof.contact_count(), 0);
    assert!(proof.graph().is_none());
    let exclusion = proof.constant_exclusion().unwrap();
    assert!(exclusion.divided_self_diagonal());
    assert_eq!(
        proof.sign_stage(),
        BezierPairSignStage::SkippedConstantExclusion
    );
    assert!(exclusion.constant().signed_coefficient(0).is_some());
}

#[test]
fn empty_source_admission_retains_original_typed_empty_cause() {
    let sources: [crate::sketch_region::ArrangementSource<'_>; 0] = [];
    let outcome = capture_boundary_inputs(&sources, rustycad_tkmath::gp::Plane3::default());
    assert!(matches!(
        outcome,
        Err(CensusCause::Input(
            crate::sketch_region::ArrangementCause::Empty
        ))
    ));
}

#[test]
fn missing_admitted_universe_is_not_vacuously_complete_pair_coverage() {
    let line = bezier_shape(&[[0., 0.], [1., 0.]], None);
    let mut inputs = super::tests::capture(&[line]);
    inputs.segments.clear();
    let census = retain_missing_universe(inputs);
    assert!(census.domains.is_empty());
    assert!(census.paired.as_ref().unwrap().reports.is_empty());
    assert!(has_obligation(
        &census,
        CertificationObligation::ExhaustivePairAndSelfDomains
    ));
    assert!(has_obligation(
        &census,
        CertificationObligation::SourceConversionAndPlane
    ));
}

fn retain_missing_universe(inputs: BoundaryInputs) -> BoundaryCensus {
    enumerate_source_bound_census(
        inputs,
        super::tests::policy(),
        super::source_binding_tests::source_policy(),
        || false,
    )
    .unwrap_or_else(|_| panic!("expected retained missing-universe receipt"))
}
