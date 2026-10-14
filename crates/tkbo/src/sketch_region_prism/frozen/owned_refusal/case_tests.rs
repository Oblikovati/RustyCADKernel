// SPDX-License-Identifier: GPL-2.0-only
//! Direct branch fixtures distinguish diagnostic projection from operation completion.
use super::*;
fn source_cases() -> Vec<(SourceConformanceCause, &'static str)> {
    use SourceConformanceCause as L;
    vec![
        source_ranges(),
        source_index(),
        (
            L::CurrentDescriptorMismatch { occurrence: 8 },
            "CurrentDescriptorMismatch { occurrence: 8 }",
        ),
        (
            L::MissingConstruction { occurrence: 9 },
            "MissingConstruction { occurrence: 9 }",
        ),
        (L::UnknownParameterBinding, "UnknownParameterBinding"),
    ]
}
fn source_ranges() -> (SourceConformanceCause, &'static str) {
    let cause = SourceConformanceCause::NativeRange {
        original: [0., 1.],
        copied: [2., 3.],
        stored: [4., 5.],
    };
    (
        cause,
        "NativeRange { original: [0.0, 1.0], copied: [2.0, 3.0], stored: [4.0, 5.0] }",
    )
}
fn source_index() -> (SourceConformanceCause, &'static str) {
    let cause = SourceConformanceCause::InvalidSegmentIndex {
        received: 7,
        expected_length: 3,
    };
    (
        cause,
        "InvalidSegmentIndex { received: 7, expected_length: 3 }",
    )
}
#[test]
fn source_scalar_branches_keep_every_observed_value() {
    for (cause, expected) in source_cases() {
        let result = NamedRefusalFixture::new()
            .projected(PrismProofJoinCause::ClosureQuery(Box::new(cause)));
        let OwnedPrismJoinCause::ClosureQuery(cause) = result.cause() else {
            panic!("expected source")
        };
        assert_eq!(format!("{cause:?}"), expected);
    }
}
#[test]
fn region_scalar_branches_keep_hypotheses_and_native_bytes() {
    let cases = region_cases();
    let expected = [
        "UnavailableCandidates",
        "MissingEvidence { hypothesis: \"closed domain\" }",
        "Hypothesis { name: \"all rims\", actual: 2, expected: 3 }",
    ];
    for (cause, expected) in cases.into_iter().zip(expected) {
        let result = NamedRefusalFixture::new()
            .projected(PrismProofJoinCause::ClosureRegion(Box::new(cause)));
        let OwnedPrismJoinCause::ClosureRegion(cause) = result.cause() else {
            panic!("expected region")
        };
        assert_eq!(format!("{cause:?}"), expected);
    }
}
fn pending_closure() -> PrismProofJoinCause {
    let point = ClosurePointEvidence {
        contexts: vec![],
        point: [-0.0, 2., 3.],
        original: original(99),
        requested_world_limit: 0.25,
        exact_certificate_limit: 0.,
        roots: RootIsolationOptions::new(1e-6),
        decision: Err(SourceConformanceCause::Exact(ResidualError::Domain([
            2., 3.,
        ]))),
        bisections: 17,
    };
    PrismProofJoinCause::ClosureRegion(Box::new(RegionCertificateCause::ClosurePoint(Box::new(
        point,
    ))))
}
#[test]
fn unprojected_closure_preserves_full_point_policy_work_and_typed_cause() {
    let error =
        retained(NamedRefusalFixture::new().run(pending_closure(), PrismRefusalClass::Incomplete));
    let point = retained_closure(error);
    assert_eq!(point.point[0].to_bits(), (-0.0f64).to_bits());
    assert_eq!(point.requested_world_limit, 0.25);
    assert_eq!(point.exact_certificate_limit, 0.);
    assert_eq!(point.bisections, 17);
    assert!(matches!(
        point.decision,
        Err(SourceConformanceCause::Exact(ResidualError::Domain([
            2., 3.
        ])))
    ));
}
#[test]
fn no_body_selection_is_not_misrepresented_as_completed_empty_projection() {
    let cause = PrismBindingFailure::Body(PrismCompletion::NoBody(
        crate::sketch_region_candidates::SeedSelection::Ambiguous {
            candidates: vec![2, 7],
        },
    ));
    let error = retained(NamedRefusalFixture::new().run(
        PrismProofJoinCause::Binding(Box::new(cause)),
        PrismRefusalClass::Incomplete,
    ));
    let local = error.into_original().1;
    let PrismProofJoinCause::Binding(cause) = local.cause else {
        panic!("expected binding")
    };
    assert!(
        matches!(*cause,PrismBindingFailure::Body(PrismCompletion::NoBody(crate::sketch_region_candidates::SeedSelection::Ambiguous {candidates})) if candidates == [2,7])
    );
}
fn freeze_failure() -> Box<PrismFreezeFailure> {
    Box::new(PrismFreezeFailure {
        operation: AuthoredPrismId(401),
        phase: FreezePhase::Commit,
        entity: None,
        cause: PrismFreezeCause::Native(native()),
    })
}
#[test]
fn frozen_capture_classes_and_phases_remain_distinct() {
    let fixture = NamedRefusalFixture::new();
    let failed = fixture.projected(PrismProofJoinCause::FailedCapture(freeze_failure()));
    let incomplete = fixture.projected(PrismProofJoinCause::IncompleteCapture(freeze_failure()));
    assert!(
        matches!(failed.cause(),OwnedPrismJoinCause::FailedCapture(value) if value.phase == FreezePhase::Commit)
    );
    assert!(
        matches!(incomplete.cause(),OwnedPrismJoinCause::IncompleteCapture(value) if matches!(&value.cause,PrismFreezeCause::Native(failure) if failure == &native()))
    );
}
#[test]
fn direct_join_and_binding_controls_preserve_exact_variants() {
    let fixture = NamedRefusalFixture::new();
    let cause = fixture.projected(PrismProofJoinCause::Version {
        actual: 7,
        expected: 1,
    });
    assert!(matches!(
        cause.cause(),
        OwnedPrismJoinCause::Version {
            actual: 7,
            expected: 1
        }
    ));
}
#[test]
fn direct_cancellation_controls_preserve_exact_variants() {
    let fixture = NamedRefusalFixture::new();
    let cause = fixture.projected(PrismProofJoinCause::Cancelled {
        freeze_phase: Some(FreezePhase::Geometry),
    });
    assert!(matches!(
        cause.cause(),
        OwnedPrismJoinCause::Cancelled {
            freeze_phase: Some(FreezePhase::Geometry)
        }
    ));
    let cause = fixture.projected(PrismProofJoinCause::Binding(Box::new(
        PrismBindingFailure::Cancelled,
    )));
    assert!(
        matches!(cause.cause(),OwnedPrismJoinCause::Binding(value) if matches!(**value,OwnedPrismBindingCause::Cancelled))
    );
}

fn region_cases() -> [RegionCertificateCause; 3] {
    [
        RegionCertificateCause::UnavailableCandidates,
        RegionCertificateCause::MissingEvidence {
            hypothesis: "closed domain",
        },
        RegionCertificateCause::Hypothesis {
            name: "all rims",
            actual: 2,
            expected: 3,
        },
    ]
}

fn retained_closure(error: LocalPrismProjectionFailure) -> Box<ClosurePointEvidence> {
    let local = error.into_original().1;
    let PrismProofJoinCause::ClosureRegion(cause) = local.cause else {
        panic!("expected region")
    };
    let RegionCertificateCause::ClosurePoint(point) = *cause else {
        panic!("expected point")
    };
    point
}

#[test]
fn zero_operation_token_refuses_without_losing_original_failure() {
    let mut fixture = NamedRefusalFixture::new();
    fixture.bindings.operation = AuthoredPrismId(0);
    let failure = zero_operation_refusal();
    let context = PrismRefusalIdentityContext {
        bindings: &fixture.bindings,
    };
    let error = retained(project_join_refusal(
        failure,
        PrismRefusalClass::Failed,
        &context,
    ));
    assert!(matches!(
        error.issue().cause(),
        PrismRefusalProjectionGap::InvalidOperation(AuthoredPrismId(0))
    ));
    assert_eq!(error.into_original().1.operation, AuthoredPrismId(0));
}

fn zero_operation_refusal() -> PrismProofJoinFailure {
    PrismProofJoinFailure {
        operation: AuthoredPrismId(0),
        phase: PrismProofJoinPhase::Admission,
        entity: None,
        cause: PrismProofJoinCause::MissingClosureQuery,
    }
}

#[test]
fn owned_refusal_process_receipt() {
    let fixture = NamedRefusalFixture::new();
    let named = fixture.projected(missing_original(0xdeadbeef));
    let numerical = fixture.projected(numerical_refusal());
    let native = fixture.projected(body_refusal(PrismCause::Mapped {
        phase: PrismPhase::Sweep,
        failure: native(),
    }));
    let bytes = format!("{named:?}\n{numerical:?}\n{native:?}\n").into_bytes();
    let hex: String = bytes.iter().map(|value| format!("{value:02x}")).collect();
    assert!(!hex.is_empty());
    println!("owned-prism-refusal-v1:{hex}");
}
