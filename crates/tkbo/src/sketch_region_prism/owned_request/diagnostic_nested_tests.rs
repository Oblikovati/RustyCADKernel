// SPDX-License-Identifier: GPL-2.0-only
//! Named local failure fixtures prove nested ownership cannot disappear behind an empty outer census.
use super::{diagnostics::*, progress::*, test_fixture::*};
use crate::sketch_region_prism::diagnostic_envelope::*;
use rustycad_tkernel::standard::Failure;
struct LocalFailureFixture;
impl LocalFailureFixture {
    fn failed(cause: OwnedPreparationCause) -> Box<OwnedPrismPreparationFailure> {
        Box::new(OwnedPrismPreparationFailure {
            class: OwnedPreparationClass::Failed,
            cause,
            callback: None,
            retained: PreparationRetained::new(request()),
        })
    }
    fn native() -> Failure {
        let cause = Failure::with_type_bytes("Standard_DomainError", &[255, 17], &[254, 29]);
        cause.with_exception_parent("Standard_Failure")
    }
}
#[test]
fn top_level_native_constructor_failure_projects_exact_bytes() {
    let original =
        LocalFailureFixture::failed(OwnedPreparationCause::Native(LocalFailureFixture::native()));
    let projection = project_owned_preparation_refusal(original);
    assert_eq!(
        projection.diagnostic().cause(),
        &OwnedPreparationDiagnosticCause::Native(LocalFailureFixture::native())
    );
    assert_eq!(
        projection.diagnostic().coverage(),
        DiagnosticProjectionCoverage::Complete
    );
}
#[test]
fn nested_census_failure_is_incomplete_even_with_no_outer_owners() {
    use crate::{sketch_region::*, sketch_region_certification::CensusCause};
    let nested = CensusCause::Input(ArrangementCause::Failure {
        phase: ArrangementPhase::Copy,
        cause: LocalFailureFixture::native(),
    });
    let original = LocalFailureFixture::failed(OwnedPreparationCause::Census(Box::new(nested)));
    assert!(
        original
            .diagnostic_context()
            .outer_retained_stages()
            .is_empty()
    );
    let projection = project_owned_preparation_refusal(original);
    let expected_class = PrismDiagnosticClass::Failed;
    assert_eq!(projection.diagnostic().class(), expected_class);
    assert_census_gap(projection.diagnostic());
    assert_original_census(projection);
}
#[test]
fn missing_callback_payload_is_explicit_and_keeps_original() {
    let original = LocalFailureFixture::failed(OwnedPreparationCause::Callback);
    let projection = project_owned_preparation_refusal(original);
    assert_eq!(
        projection.diagnostic().gaps(),
        &[PrismDiagnosticGap::MissingCallbackEvidence]
    );
    assert_eq!(
        projection.diagnostic().coverage(),
        DiagnosticProjectionCoverage::Incomplete
    );
    assert!(projection.into_parts().1.is_some());
}
#[test]
fn unavailable_stage_keeps_class_and_unprojected_completion_gap() {
    let cause = OwnedPreparationCause::StageUnavailable {
        class: OwnedPreparationClass::Cancelled,
    };
    let projection = project_owned_preparation_refusal(LocalFailureFixture::failed(cause));
    let expected_class = PrismDiagnosticClass::Failed;
    assert_eq!(projection.diagnostic().class(), expected_class);
    assert_eq!(
        projection.diagnostic().cause(),
        &OwnedPreparationDiagnosticCause::StageUnavailable {
            class: OwnedPreparationClass::Cancelled
        }
    );
    assert_stage_gap(projection.diagnostic());
    assert!(projection.into_parts().1.is_some());
}
#[test]
fn missing_local_state_preserves_specific_expected_contract() {
    let cause = OwnedPreparationCause::MissingLocalState {
        expected: "constructed polynomial source",
    };
    let projection = project_owned_preparation_refusal(LocalFailureFixture::failed(cause));
    assert_eq!(
        projection.diagnostic().cause(),
        &OwnedPreparationDiagnosticCause::MissingLocalState {
            expected: "constructed polynomial source"
        }
    );
    assert_eq!(
        projection.diagnostic().coverage(),
        DiagnosticProjectionCoverage::Complete
    );
}
#[test]
fn exhausted_callback_remains_typed_with_exact_checkpoint() {
    let mut original = LocalFailureFixture::failed(OwnedPreparationCause::Callback);
    let checkpoint = OwnedPrismCheckpoint {
        phase: OwnedPrismPhase::Census,
        ordinal: usize::MAX,
    };
    original.callback = Some(OwnedPrismCallbackStop {
        checkpoint,
        cause: OwnedPrismCallbackCause::Exhausted,
    });
    let projection = project_owned_preparation_refusal(original);
    assert_exhausted(projection.diagnostic(), checkpoint);
}

fn assert_original_census(projection: PreparationRefusalProjection) {
    use crate::{sketch_region::*, sketch_region_certification::CensusCause};
    let (_, original) = projection.into_parts();
    let OwnedPreparationCause::Census(cause) = original.unwrap().cause else {
        panic!("expected original census cause")
    };
    let CensusCause::Input(ArrangementCause::Failure { phase, cause }) = *cause else {
        panic!("expected exact original nested failure")
    };
    assert_eq!(phase, ArrangementPhase::Copy);
    assert_eq!(cause, LocalFailureFixture::native());
}

fn assert_stage_gap(value: &OwnedPrismDiagnostic) {
    let expected =
        PrismDiagnosticGap::PreparationCause(UnprojectedPreparationPayload::StageCompletion);
    assert_eq!(value.gaps(), &[expected]);
}

fn assert_exhausted(value: &OwnedPrismDiagnostic, checkpoint: OwnedPrismCheckpoint) {
    let expected = OwnedPreparationCallback {
        checkpoint,
        cause: OwnedDiagnosticCallbackCause::Exhausted,
    };
    assert_eq!(value.callback(), Some(&expected));
    assert_eq!(value.coverage(), DiagnosticProjectionCoverage::Complete);
}

fn assert_census_gap(value: &OwnedPrismDiagnostic) {
    let gap = PrismDiagnosticGap::PreparationCause(UnprojectedPreparationPayload::Census);
    assert_eq!(value.gaps(), &[gap]);
}
#[test]
fn actual_contact_policy_failure_keeps_nested_transaction_and_exact_value() {
    let mut input = request();
    input.policy.contacts.world_endpoint_limit = -1.;
    let original = super::prepare_owned_polynomial_prism(input, &mut RequestObserver::default())
        .err()
        .unwrap();
    let projection = project_owned_preparation_refusal(original);
    let cause_gap =
        PrismDiagnosticGap::PreparationCause(UnprojectedPreparationPayload::ContactTransport);
    assert!(projection.diagnostic().gaps().contains(&cause_gap));
    let expected_class = PrismDiagnosticClass::Failed;
    assert_eq!(projection.diagnostic().class(), expected_class);
    let (_, original) = projection.into_parts();
    assert_original_contact(original.unwrap());
}
fn assert_original_contact(original: Box<OwnedPrismPreparationFailure>) {
    use crate::sketch_region_contact_transport::ContactTransportStop;
    let OwnedPreparationCause::Contact(ContactTransportStop::Policy {
        field,
        value,
        completed,
    }) = original.cause
    else {
        panic!("expected exact contact policy failure")
    };
    assert_eq!(field, "world endpoint limit");
    assert_eq!(value.to_bits(), (-1.0f64).to_bits());
    assert_eq!(
        completed.requested.world_endpoint_limit.to_bits(),
        value.to_bits()
    );
}
#[test]
fn unavailable_region_keeps_original_typed_request_and_missing_hypothesis() {
    let prepared = prepared(request());
    let requested = prepared.retained.prism.as_ref().unwrap().limits.requested;
    let local = unavailable_region(requested);
    let original =
        LocalFailureFixture::failed(OwnedPreparationCause::PrismUnavailable(Box::new(local)));
    let projection = project_owned_preparation_refusal(original);
    let gap = PrismDiagnosticGap::PreparationCause(UnprojectedPreparationPayload::RegionPrism);
    assert_eq!(projection.diagnostic().gaps(), &[gap]);
    let (_, original) = projection.into_parts();
    let OwnedPreparationCause::PrismUnavailable(local) = original.unwrap().cause else {
        panic!("expected original unavailable region")
    };
    assert_region_hypothesis(&local.cause);
    assert_eq!(local.requested.seed, requested.seed);
}

fn unavailable_region(
    requested: crate::sketch_region_prism::PrismRequest,
) -> crate::sketch_region_prism::region_certificate::RegionPrismUnavailable {
    use crate::sketch_region_prism::region_certificate::{
        RegionCertificateCause, RegionPrismUnavailable,
    };
    RegionPrismUnavailable {
        requested,
        cause: RegionCertificateCause::MissingEvidence {
            hypothesis: "independent fixture hypothesis",
        },
        missing: vec![],
    }
}

fn assert_region_hypothesis(
    cause: &crate::sketch_region_prism::region_certificate::RegionCertificateCause,
) {
    use crate::sketch_region_prism::region_certificate::RegionCertificateCause;
    assert!(matches!(
        cause,
        RegionCertificateCause::MissingEvidence {
            hypothesis: "independent fixture hypothesis"
        }
    ));
}
